import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import { randomUUID } from 'node:crypto';
import { SendMessageCommand, SQSClient } from '@aws-sdk/client-sqs';
import {
  getContactMessagesCollection,
  hasContact,
  parseJsonBody,
  toMessageDto,
  type ContactMessageDocument,
} from '../../contacts';
import { getSecurityCode, jsonResponse } from '../../spaceship';

const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const MAX_MESSAGE_LENGTH = 1_000;
const sqs = new SQSClient({});

async function enqueueReply(message: ContactMessageDocument) {
  const queueUrl = process.env.CONTACT_REPLY_QUEUE_URL;
  if (!queueUrl) throw new Error('CONTACT_REPLY_QUEUE_URL is not configured');
  await sqs.send(
    new SendMessageCommand({
      QueueUrl: queueUrl,
      MessageBody: JSON.stringify({
        spaceshipSecurityCode: message.spaceshipSecurityCode,
        contactId: message.contactId,
        playerMessageId: message._id,
      }),
      MessageGroupId: `${message.spaceshipSecurityCode}:${message.contactId}`,
      MessageDeduplicationId: message._id,
    }),
  );
}

export async function handler(
  event: APIGatewayProxyEventV2,
): Promise<APIGatewayProxyResultV2> {
  const securityCode = getSecurityCode(event);
  if (!securityCode) {
    return jsonResponse(400, {
      error: 'A valid x-spaceship-security-code header is required',
    });
  }

  let contactId: string;
  let text: string;
  let clientMessageId: string;
  try {
    const body = parseJsonBody(event);
    contactId = typeof body.contactId === 'string' ? body.contactId.trim() : '';
    text = typeof body.text === 'string' ? body.text.trim() : '';
    clientMessageId =
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
  } catch (error) {
    return jsonResponse(400, {
      error: error instanceof Error ? error.message : 'Invalid request body',
    });
  }

  try {
    if (!(await hasContact(securityCode, contactId))) {
      return jsonResponse(404, { error: 'Contact not found' });
    }

    const messages = await getContactMessagesCollection();
    const existing = await messages.findOne({
      spaceshipSecurityCode: securityCode,
      contactId,
      clientMessageId,
    });
    if (existing) {
      if (existing.status === 'failed') {
        await enqueueReply(existing);
        existing.status = 'sent';
        await messages.updateOne(
          { _id: existing._id },
          { $set: { status: 'sent' } },
        );
      }
      return jsonResponse(202, { message: toMessageDto(existing) });
    }

    const recentMessageCount = await messages.countDocuments({
      spaceshipSecurityCode: securityCode,
      sender: 'player',
      createdAt: { $gt: new Date(Date.now() - 60_000) },
    });
    if (recentMessageCount >= 10) {
      return jsonResponse(429, {
        error: 'Message rate limit exceeded. Try again shortly.',
      });
    }

    const message: ContactMessageDocument = {
      _id: randomUUID(),
      spaceshipSecurityCode: securityCode,
      contactId,
      sender: 'player',
      text,
      status: 'queued',
      clientMessageId,
      createdAt: new Date(),
    };
    await messages.insertOne(message);

    try {
      await enqueueReply(message);
      message.status = 'sent';
      await messages.updateOne(
        { _id: message._id },
        { $set: { status: 'sent' } },
      );
    } catch (error) {
      await messages.updateOne(
        { _id: message._id },
        { $set: { status: 'failed' } },
      );
      throw error;
    }

    return jsonResponse(202, { message: toMessageDto(message) });
  } catch (error) {
    console.error('Failed to send contact message', error);
    return jsonResponse(500, { error: 'Failed to send contact message' });
  }
}
