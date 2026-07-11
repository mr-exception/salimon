import { ContactMessageModel } from '@models';
import { ContactsService } from '@services';
import { getRequiredSecurityCode, sendError } from '../../http';
import { queryString } from './query-string';
import type { Request, Response } from 'express';

export async function getMessages(request: Request, response: Response) {
  const securityCode = getRequiredSecurityCode(request, response);
  if (!securityCode) return;

  const contactId = queryString(request.query.contactId)?.trim();
  if (!contactId) {
    sendError(response, 400, 'contactId is required');
    return;
  }
  const afterValue = queryString(request.query.after);
  const after = afterValue
    ? ContactsService.decodeMessageCursor(afterValue)
    : undefined;
  if (afterValue && !after) {
    sendError(response, 400, 'after must be a valid cursor');
    return;
  }
  const requestedLimit = Number(queryString(request.query.limit) ?? 50);
  if (!Number.isInteger(requestedLimit) || requestedLimit < 1) {
    sendError(response, 400, 'limit must be a positive integer');
    return;
  }
  const limit = Math.min(requestedLimit, 100);

  try {
    if (!(await ContactsService.hasContact(securityCode, contactId))) {
      sendError(response, 404, 'Contact not found');
      return;
    }

    const filter: Record<string, unknown> = {
      spaceshipSecurityCode: securityCode,
      contactId,
      ...(after
        ? {
            $or: [
              { createdAt: { $gt: after.createdAt } },
              { createdAt: after.createdAt, _id: { $gt: after.id } },
            ],
          }
        : {}),
    };
    const messages = await ContactMessageModel.findMessages(filter, limit);

    const unreadMessageIds = messages
      .filter(
        (message) => message.sender === 'contact' && message.isRead !== true,
      )
      .map((message) => message._id);
    if (unreadMessageIds.length > 0) {
      await ContactMessageModel.markMessagesRead(
        securityCode,
        contactId,
        unreadMessageIds,
      );
      messages.forEach((message) => {
        if (unreadMessageIds.includes(message._id)) message.isRead = true;
      });
    }

    response.json({
      messages: messages.map(ContactsService.toMessageDto),
      cursor:
        messages.length > 0
          ? ContactsService.encodeMessageCursor(messages[messages.length - 1])
          : afterValue,
    });
  } catch (error) {
    console.error('Failed to load contact messages', error);
    sendError(response, 500, 'Failed to load contact messages');
  }
}
