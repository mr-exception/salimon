import type { ContactMessageDocument } from '@models';

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

