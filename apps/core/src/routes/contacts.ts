import { Router } from 'express';
import { ContactModel, ContactMessageModel } from '@models';
import { CONTACTS, ContactsService, RepositoryService } from '@services';
import { asyncHandler, sendError } from '../http';

function queryString(value: unknown) {
  return typeof value === 'string' ? value : undefined;
}

export const contactsRouter = Router();

contactsRouter.get(
  '/info',
  asyncHandler(async (request, response) => {
    const securityCode = request.securityCode;
    if (!securityCode) {
      sendError(
        response,
        400,
        'A valid x-spaceship-security-code header is required',
      );
      return;
    }

    try {
      const spaceship =
        await RepositoryService.findSpaceshipBySecurityCode(securityCode);
      if (!spaceship) {
        sendError(response, 404, 'Spaceship not found');
        return;
      }

      let contacts =
        await ContactModel.findBySpaceshipSecurityCode(securityCode);
      if (contacts.length === 0) {
        await ContactsService.initializeSpaceshipContacts(securityCode);
        contacts = await ContactModel.findBySpaceshipSecurityCode(securityCode);
      }

      response.json({
        contacts: await Promise.all(
          contacts.flatMap(async (contact) => {
            const profile =
              CONTACTS[contact.contactId as keyof typeof CONTACTS];
            if (!profile) return [];
            const [latestMessage, unreadCount] = await Promise.all([
              ContactsService.findLatestMessage(
                securityCode,
                contact.contactId,
              ),
              ContactMessageModel.countUnreadContactMessages(
                securityCode,
                contact.contactId,
              ),
            ]);
            return [
              {
                ...profile,
                unreadCount,
                lastMessageAt: latestMessage?.createdAt.toISOString(),
              },
            ];
          }),
        ).then((groups) => groups.flat()),
      });
    } catch (error) {
      console.error('Failed to load contacts', error);
      sendError(response, 500, 'Failed to load contacts');
    }
  }),
);

contactsRouter.get(
  '/messages',
  asyncHandler(async (request, response) => {
    const securityCode = request.securityCode;
    if (!securityCode) {
      sendError(
        response,
        400,
        'A valid x-spaceship-security-code header is required',
      );
      return;
    }

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
  }),
);

contactsRouter.get(
  '/messages/unread',
  asyncHandler(async (request, response) => {
    const securityCode = request.securityCode;
    if (!securityCode) {
      sendError(
        response,
        400,
        'A valid x-spaceship-security-code header is required',
      );
      return;
    }

    try {
      const spaceship =
        await RepositoryService.findSpaceshipBySecurityCode(securityCode);
      if (!spaceship) {
        sendError(response, 404, 'Spaceship not found');
        return;
      }
      await ContactsService.initializeSpaceshipContacts(securityCode);

      const messages =
        await ContactMessageModel.findUnreadContactMessages(securityCode);

      response.json({ messages: messages.map(ContactsService.toMessageDto) });
    } catch (error) {
      console.error('Failed to load unread contact messages', error);
      sendError(response, 500, 'Failed to load unread contact messages');
    }
  }),
);

contactsRouter.post(
  '/messages/send',
  asyncHandler(async (request, response) => {
    const securityCode = request.securityCode;
    if (!securityCode) {
      sendError(
        response,
        400,
        'A valid x-spaceship-security-code header is required',
      );
      return;
    }

    let messageRequest: {
      contactId: string;
      text: string;
      clientMessageId: string;
    };
    try {
      const body = ContactsService.parseJsonBody(request.body);
      messageRequest = ContactsService.parseSendMessageRequest(body);
    } catch (error) {
      sendError(
        response,
        400,
        error instanceof Error ? error.message : 'Invalid request body',
      );
      return;
    }

    try {
      const message = await ContactsService.sendMessage(
        securityCode,
        messageRequest,
      );

      response
        .status(202)
        .json({ message: ContactsService.toMessageDto(message) });
    } catch (error) {
      if (error instanceof Error && error.message === 'Contact not found') {
        sendError(response, 404, error.message);
        return;
      }
      if (
        error instanceof Error &&
        error.message === 'Message rate limit exceeded. Try again shortly.'
      ) {
        sendError(response, 429, error.message);
        return;
      }
      console.error('Failed to send contact message', error);
      sendError(response, 500, 'Failed to send contact message');
    }
  }),
);
