import { randomUUID } from 'node:crypto';
import { Router } from 'express';
import {
  ContactModel,
  ContactMessageModel,
  type ContactMessageDocument,
} from '@models';
import {
  CONTACTS,
  ContactRepliesService,
  ContactsService,
  RepositoryService,
} from '@services';
import { asyncHandler, sendError } from '../http';

const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const MAX_MESSAGE_LENGTH = 1_000;

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

    let contactId: string;
    let text: string;
    let clientMessageId: string;
    try {
      const body = ContactsService.parseJsonBody(request.body);
      contactId =
        typeof body.contactId === 'string' ? body.contactId.trim() : '';
      text = typeof body.text === 'string' ? body.text.trim() : '';
      clientMessageId =
        typeof body.clientMessageId === 'string'
          ? body.clientMessageId.trim()
          : '';
      if (!contactId) throw new Error('contactId is required');
      if (!text) throw new Error('text is required');
      if (text.length > MAX_MESSAGE_LENGTH) {
        throw new Error(
          `text must be at most ${MAX_MESSAGE_LENGTH} characters`,
        );
      }
      if (!UUID_PATTERN.test(clientMessageId)) {
        throw new Error('clientMessageId must be a UUID');
      }
    } catch (error) {
      sendError(
        response,
        400,
        error instanceof Error ? error.message : 'Invalid request body',
      );
      return;
    }

    try {
      if (!(await ContactsService.hasContact(securityCode, contactId))) {
        sendError(response, 404, 'Contact not found');
        return;
      }

      const existing = await ContactMessageModel.findByClientMessage(
        securityCode,
        contactId,
        clientMessageId,
      );
      if (existing) {
        if (existing.status === 'failed') {
          ContactRepliesService.generateContactReplyInBackground(existing);
          existing.status = 'sent';
          await ContactMessageModel.updateStatus(existing._id, 'sent');
        }
        response
          .status(202)
          .json({ message: ContactsService.toMessageDto(existing) });
        return;
      }

      const recentMessageCount =
        await ContactMessageModel.countRecentPlayerMessages(
          securityCode,
          new Date(Date.now() - 60_000),
        );
      if (recentMessageCount >= 10) {
        sendError(
          response,
          429,
          'Message rate limit exceeded. Try again shortly.',
        );
        return;
      }

      const message: ContactMessageDocument = {
        _id: randomUUID(),
        spaceshipSecurityCode: securityCode,
        contactId,
        sender: 'player',
        text,
        status: 'queued',
        isRead: true,
        clientMessageId,
        createdAt: new Date(),
      };
      await ContactMessageModel.insert(message);
      ContactRepliesService.generateContactReplyInBackground(message);
      message.status = 'sent';
      await ContactMessageModel.updateStatus(message._id, 'sent');

      response
        .status(202)
        .json({ message: ContactsService.toMessageDto(message) });
    } catch (error) {
      console.error('Failed to send contact message', error);
      sendError(response, 500, 'Failed to send contact message');
    }
  }),
);
