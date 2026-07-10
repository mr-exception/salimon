import { randomUUID } from 'node:crypto';
import {
  ContactModel,
  ContactMessageModel,
  type ContactMessageDocument,
} from '@models';

export const EASA_CHIEF_ID = 'easa-chief';
export const INITIAL_CHIEF_MESSAGE =
  'Pilot, this is the Chief of EASA. An unknown being contacted humanity and gave us an energy cube with one message: “Reach Absenat, where the world is going to start.” We named the cube the Core. It produces the oxygen and electricity your ship needs, and both resources can be harvested aboard. Your mission is to leave Earth and reach Absenat. We do not yet know who sent the Core or what awaits you there. Stay in contact.';

export type { ContactDocument } from '@models';
export type {
  ContactMessageDocument,
  ContactMessageSender,
  ContactMessageStatus,
} from '@models';

export const CONTACTS = {
  [EASA_CHIEF_ID]: {
    id: EASA_CHIEF_ID,
    name: 'Chief of EASA',
    organization: 'Earth Aeronautics and Space Administration',
  },
} as const;

const MAX_MESSAGE_LENGTH = 1_000;
const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export class ContactsService {
  static async initializeSpaceshipContacts(spaceshipSecurityCode: string) {
    const now = new Date();

    await ContactModel.upsertSpaceshipContact({
      spaceshipSecurityCode,
      contactId: EASA_CHIEF_ID,
      unlockedAt: now,
    });

    await ContactMessageModel.upsertInitialMessage({
      _id: randomUUID(),
      spaceshipSecurityCode,
      contactId: EASA_CHIEF_ID,
      sender: 'contact',
      text: INITIAL_CHIEF_MESSAGE,
      status: 'sent',
      isRead: false,
      clientMessageId: 'initial-briefing',
      createdAt: now,
    });
  }

  static async hasContact(spaceshipSecurityCode: string, contactId: string) {
    return Boolean(
      await ContactModel.findBySpaceshipAndContact(
        spaceshipSecurityCode,
        contactId,
      ),
    );
  }

  static parseJsonBody(body: unknown) {
    if (!body || typeof body !== 'object' || Array.isArray(body)) {
      throw new Error('Request body must be an object');
    }
    return body as Record<string, unknown>;
  }

  static toMessageDto(message: ContactMessageDocument) {
    return {
      id: message._id,
      contactId: message.contactId,
      sender: message.sender,
      text: message.text,
      status: message.status,
      isRead: message.isRead,
      createdAt: message.createdAt.toISOString(),
    };
  }

  static encodeMessageCursor(message: ContactMessageDocument) {
    return Buffer.from(
      JSON.stringify([message.createdAt.toISOString(), message._id]),
    ).toString('base64url');
  }

  static decodeMessageCursor(cursor: string) {
    try {
      const value: unknown = JSON.parse(
        Buffer.from(cursor, 'base64url').toString('utf8'),
      );
      if (
        !Array.isArray(value) ||
        value.length !== 2 ||
        typeof value[0] !== 'string' ||
        Number.isNaN(Date.parse(value[0])) ||
        typeof value[1] !== 'string'
      ) {
        return undefined;
      }
      return { createdAt: new Date(value[0]), id: value[1] };
    } catch {
      return undefined;
    }
  }

  static findLatestMessage(spaceshipSecurityCode: string, contactId: string) {
    return ContactMessageModel.findLatest(spaceshipSecurityCode, contactId);
  }

  static parseSendMessageRequest(body: Record<string, unknown>) {
    const contactId =
      typeof body.contactId === 'string' ? body.contactId.trim() : '';
    const text = typeof body.text === 'string' ? body.text.trim() : '';
    const clientMessageId =
      typeof body.clientMessageId === 'string'
        ? body.clientMessageId.trim()
        : '';

    if (!contactId) throw new Error('contactId is required');
    if (!text) throw new Error('text is required');
    if (text.length > MAX_MESSAGE_LENGTH) {
      throw new Error(`text must be at most ${MAX_MESSAGE_LENGTH} characters`);
    }
    if (!UUID_PATTERN.test(clientMessageId)) {
      throw new Error('clientMessageId must be a UUID');
    }

    return { contactId, text, clientMessageId };
  }

  static async sendMessage(
    spaceshipSecurityCode: string,
    request: {
      contactId: string;
      text: string;
      clientMessageId: string;
    },
    options: {
      onReply?: (message: ContactMessageDocument) => void;
    } = {},
  ) {
    const { ContactRepliesService } = await import(
      './contact-replies.service.js'
    );

    if (
      !(await ContactsService.hasContact(
        spaceshipSecurityCode,
        request.contactId,
      ))
    ) {
      throw new Error('Contact not found');
    }

    const existing = await ContactMessageModel.findByClientMessage(
      spaceshipSecurityCode,
      request.contactId,
      request.clientMessageId,
    );
    if (existing) {
      if (existing.status === 'failed') {
        ContactRepliesService.generateContactReplyInBackground(
          existing,
          options,
        );
        existing.status = 'sent';
        await ContactMessageModel.updateStatus(existing._id, 'sent');
      }
      return existing;
    }

    const recentMessageCount =
      await ContactMessageModel.countRecentPlayerMessages(
        spaceshipSecurityCode,
        new Date(Date.now() - 60_000),
      );
    if (recentMessageCount >= 10) {
      throw new Error('Message rate limit exceeded. Try again shortly.');
    }

    const message: ContactMessageDocument = {
      _id: randomUUID(),
      spaceshipSecurityCode,
      contactId: request.contactId,
      sender: 'player',
      text: request.text,
      status: 'queued',
      isRead: true,
      clientMessageId: request.clientMessageId,
      createdAt: new Date(),
    };
    await ContactMessageModel.insert(message);
    ContactRepliesService.generateContactReplyInBackground(message, options);
    message.status = 'sent';
    await ContactMessageModel.updateStatus(message._id, 'sent');

    return message;
  }
}
