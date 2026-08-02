import type { ContactMessageDto } from '@repo/types';

const DATABASE_NAME = 'salimon-communications';
const DATABASE_VERSION = 1;
const MESSAGE_STORE_NAME = 'contactMessages';

type CachedContactMessage = ContactMessageDto & {
  cacheKey: string;
  spaceshipSecurityCode: string;
  shipContactKey: string;
};

let databasePromise: Promise<IDBDatabase> | undefined;

function getMessageCacheKey(
  spaceshipSecurityCode: string,
  messageId: string,
) {
  return `${spaceshipSecurityCode}:${messageId}`;
}

function getShipContactKey(spaceshipSecurityCode: string, contactId: string) {
  return `${spaceshipSecurityCode}:${contactId}`;
}

function openMessageDatabase() {
  if (databasePromise) return databasePromise;

  databasePromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DATABASE_NAME, DATABASE_VERSION);

    request.onupgradeneeded = () => {
      const database = request.result;
      const store = database.createObjectStore(MESSAGE_STORE_NAME, {
        keyPath: 'cacheKey',
      });
      store.createIndex('shipContactKey', 'shipContactKey', { unique: false });
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });

  return databasePromise;
}

function completeTransaction(transaction: IDBTransaction) {
  return new Promise<void>((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}

function toCachedMessage(
  spaceshipSecurityCode: string,
  message: ContactMessageDto,
): CachedContactMessage {
  return {
    ...message,
    cacheKey: getMessageCacheKey(spaceshipSecurityCode, message.id),
    spaceshipSecurityCode,
    shipContactKey: getShipContactKey(spaceshipSecurityCode, message.contactId),
  };
}

function fromCachedMessage(message: CachedContactMessage): ContactMessageDto {
  return {
    id: message.id,
    contactId: message.contactId,
    sender: message.sender,
    text: message.text,
    status: message.status,
    isRead: message.isRead,
    createdAt: message.createdAt,
  };
}

export async function loadCachedContactMessages(
  spaceshipSecurityCode: string,
  contactId: string,
) {
  const database = await openMessageDatabase();
  return new Promise<ContactMessageDto[]>((resolve, reject) => {
    const transaction = database.transaction(MESSAGE_STORE_NAME, 'readonly');
    const store = transaction.objectStore(MESSAGE_STORE_NAME);
    const index = store.index('shipContactKey');
    const request = index.getAll(
      getShipContactKey(spaceshipSecurityCode, contactId),
    );

    request.onsuccess = () => {
      resolve(
        (request.result as CachedContactMessage[])
          .map(fromCachedMessage)
          .sort(
            (left, right) =>
              Date.parse(left.createdAt) - Date.parse(right.createdAt),
          ),
      );
    };
    request.onerror = () => reject(request.error);
  });
}

export async function storeCachedContactMessages(
  spaceshipSecurityCode: string,
  messages: ContactMessageDto[],
) {
  if (messages.length === 0) return;

  const database = await openMessageDatabase();
  const transaction = database.transaction(MESSAGE_STORE_NAME, 'readwrite');
  const transactionDone = completeTransaction(transaction);
  const store = transaction.objectStore(MESSAGE_STORE_NAME);

  messages.forEach((message) => {
    store.put(toCachedMessage(spaceshipSecurityCode, message));
  });

  await transactionDone;
}

export async function storeCachedContactMessage(
  spaceshipSecurityCode: string,
  message: ContactMessageDto,
) {
  await storeCachedContactMessages(spaceshipSecurityCode, [message]);
}

export async function markCachedContactMessagesRead(
  spaceshipSecurityCode: string,
  contactId: string,
  messageIds: string[],
) {
  if (messageIds.length === 0) return;

  const database = await openMessageDatabase();
  const transaction = database.transaction(MESSAGE_STORE_NAME, 'readwrite');
  const transactionDone = completeTransaction(transaction);
  const store = transaction.objectStore(MESSAGE_STORE_NAME);
  const readIds = new Set(messageIds);

  await Promise.all(
    messageIds.map(
      (messageId) =>
        new Promise<void>((resolve, reject) => {
          const request = store.get(
            getMessageCacheKey(spaceshipSecurityCode, messageId),
          );
          request.onsuccess = () => {
            const message = request.result as CachedContactMessage | undefined;
            if (
              message &&
              message.contactId === contactId &&
              readIds.has(message.id)
            ) {
              store.put({ ...message, isRead: true });
            }
            resolve();
          };
          request.onerror = () => reject(request.error);
        }),
    ),
  );

  await transactionDone;
}
