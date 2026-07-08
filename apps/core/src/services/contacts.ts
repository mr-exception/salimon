import { randomUUID } from 'node:crypto';
import type { Collection } from 'mongodb';
import { getDatabase } from './spaceship';

export const EASA_CHIEF_ID = 'easa-chief';
export const INITIAL_CHIEF_MESSAGE =
  'Pilot, this is the Chief of EASA. An unknown being contacted humanity and gave us an energy cube with one message: “Reach Absenat, where the world is going to start.” We named the cube the Core. It produces the oxygen and electricity your ship needs, and both resources can be harvested aboard. Your mission is to leave Earth and reach Absenat. We do not yet know who sent the Core or what awaits you there. Stay in contact.';

export type ContactDocument = {
  spaceshipSecurityCode: string;
  contactId: string;
  unlockedAt: Date;
  lastReadAt?: Date;
};

export type ContactMessageSender = 'player' | 'contact';
export type ContactMessageStatus = 'sent' | 'queued' | 'failed';

export type ContactMessageDocument = {
  _id: string;
  spaceshipSecurityCode: string;
  contactId: string;
  sender: ContactMessageSender;
  text: string;
  status: ContactMessageStatus;
  isRead: boolean;
  clientMessageId?: string;
  createdAt: Date;
};

export const CONTACTS = {
  [EASA_CHIEF_ID]: {
    id: EASA_CHIEF_ID,
    name: 'Chief of EASA',
    organization: 'Earth Aeronautics and Space Administration',
  },
} as const;

let contactIndexesPromise: Promise<unknown> | undefined;
let messageIndexesPromise: Promise<unknown> | undefined;

export async function getContactsCollection() {
  const collection = (await getDatabase()).collection<ContactDocument>(
    'contacts',
  );
  contactIndexesPromise ??= collection.createIndex(
    { spaceshipSecurityCode: 1, contactId: 1 },
    { unique: true },
  );
  await contactIndexesPromise;
  return collection;
}

export async function getContactMessagesCollection() {
  const collection = (await getDatabase()).collection<ContactMessageDocument>(
    'contactMessages',
  );
  messageIndexesPromise ??= collection.createIndexes([
    {
      key: { spaceshipSecurityCode: 1, contactId: 1, createdAt: 1, _id: 1 },
    },
    {
      key: {
        spaceshipSecurityCode: 1,
        sender: 1,
        isRead: 1,
        createdAt: 1,
      },
    },
    {
      key: { spaceshipSecurityCode: 1, contactId: 1, clientMessageId: 1 },
      name: 'unique_client_message',
      unique: true,
      partialFilterExpression: { clientMessageId: { $type: 'string' } },
    },
  ]);
  await messageIndexesPromise;
  return collection;
}

export async function initializeSpaceshipContacts(
  spaceshipSecurityCode: string,
) {
  const now = new Date();
  const contacts = await getContactsCollection();
  const messages = await getContactMessagesCollection();

  await contacts.updateOne(
    { spaceshipSecurityCode, contactId: EASA_CHIEF_ID },
    {
      $setOnInsert: {
        spaceshipSecurityCode,
        contactId: EASA_CHIEF_ID,
        unlockedAt: now,
      },
    },
    { upsert: true },
  );

  await messages.updateOne(
    {
      spaceshipSecurityCode,
      contactId: EASA_CHIEF_ID,
      sender: 'contact',
      clientMessageId: 'initial-briefing',
    },
    {
      $setOnInsert: {
        _id: randomUUID(),
        spaceshipSecurityCode,
        contactId: EASA_CHIEF_ID,
        sender: 'contact',
        text: INITIAL_CHIEF_MESSAGE,
        status: 'sent',
        isRead: false,
        clientMessageId: 'initial-briefing',
        createdAt: now,
      },
    },
    { upsert: true },
  );
}

export async function hasContact(
  spaceshipSecurityCode: string,
  contactId: string,
) {
  return Boolean(
    await (
      await getContactsCollection()
    ).findOne({ spaceshipSecurityCode, contactId }),
  );
}

export function parseJsonBody(body: unknown) {
  if (!body || typeof body !== 'object' || Array.isArray(body)) {
    throw new Error('Request body must be an object');
  }
  return body as Record<string, unknown>;
}

export function toMessageDto(message: ContactMessageDocument) {
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

export function encodeMessageCursor(message: ContactMessageDocument) {
  return Buffer.from(
    JSON.stringify([message.createdAt.toISOString(), message._id]),
  ).toString('base64url');
}

export function decodeMessageCursor(cursor: string) {
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

export async function findLatestMessage(
  messages: Collection<ContactMessageDocument>,
  spaceshipSecurityCode: string,
  contactId: string,
) {
  return messages.findOne(
    { spaceshipSecurityCode, contactId },
    { sort: { createdAt: -1, _id: -1 } },
  );
}
