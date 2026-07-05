import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import type { Filter } from 'mongodb';
import {
  decodeMessageCursor,
  encodeMessageCursor,
  getContactMessagesCollection,
  getContactsCollection,
  hasContact,
  toMessageDto,
  type ContactMessageDocument,
} from '../contacts';
import { getSecurityCode, jsonResponse } from '../spaceship';

export async function handler(
  event: APIGatewayProxyEventV2,
): Promise<APIGatewayProxyResultV2> {
  const securityCode = getSecurityCode(event);
  if (!securityCode) {
    return jsonResponse(400, {
      error: 'A valid x-spaceship-security-code header is required',
    });
  }

  const contactId = event.queryStringParameters?.contactId?.trim();
  if (!contactId) {
    return jsonResponse(400, { error: 'contactId is required' });
  }
  const afterValue = event.queryStringParameters?.after;
  const after = afterValue ? decodeMessageCursor(afterValue) : undefined;
  if (afterValue && !after) {
    return jsonResponse(400, { error: 'after must be a valid cursor' });
  }
  const requestedLimit = Number(event.queryStringParameters?.limit ?? 50);
  if (!Number.isInteger(requestedLimit) || requestedLimit < 1) {
    return jsonResponse(400, { error: 'limit must be a positive integer' });
  }
  const limit = Math.min(requestedLimit, 100);

  try {
    if (!(await hasContact(securityCode, contactId))) {
      return jsonResponse(404, { error: 'Contact not found' });
    }

    const filter: Filter<ContactMessageDocument> = {
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
    const messages = await (await getContactMessagesCollection())
      .find(filter)
      .sort({ createdAt: 1, _id: 1 })
      .limit(limit)
      .toArray();

    if (messages.some((message) => message.sender === 'contact')) {
      await (
        await getContactsCollection()
      ).updateOne(
        { spaceshipSecurityCode: securityCode, contactId },
        { $set: { lastReadAt: new Date() } },
      );
    }

    return jsonResponse(200, {
      messages: messages.map(toMessageDto),
      cursor:
        messages.length > 0
          ? encodeMessageCursor(messages[messages.length - 1])
          : afterValue,
    });
  } catch (error) {
    console.error('Failed to load contact messages', error);
    return jsonResponse(500, { error: 'Failed to load contact messages' });
  }
}
