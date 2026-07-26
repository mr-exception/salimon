import { randomUUID } from 'node:crypto';
import { ContactMessageModel, type ContactMessageDocument } from '@models';
import { CONTACTS } from '../contacts.service';
import { buildContactInstructions, MAX_CONTEXT_MESSAGES } from './constants';
import { getOpenAI } from './openai';
import type { ReplyJob } from './types';

export async function generateContactReply(job: ReplyJob) {
  const profile = CONTACTS[job.contactId as keyof typeof CONTACTS];
  if (!profile) {
    throw new Error(`Unknown contact ${job.contactId}`);
  }

  const playerMessage = await ContactMessageModel.findPlayerMessage(
    job.spaceshipSecurityCode,
    job.contactId,
    job.playerMessageId,
  );
  if (!playerMessage) throw new Error('Player message not found');

  const existingReply = await ContactMessageModel.findByClientMessage(
    job.spaceshipSecurityCode,
    job.contactId,
    `reply:${job.playerMessageId}`,
  );
  if (existingReply) return;

  const history = await ContactMessageModel.findReplyContext(
    job.spaceshipSecurityCode,
    job.contactId,
    MAX_CONTEXT_MESSAGES,
  );

  const response = await getOpenAI().responses.create({
    model: process.env.OPENAI_MODEL ?? 'gpt-5.4-mini',
    instructions: buildContactInstructions(profile),
    input: history.reverse().map((message) => ({
      role:
        message.sender === 'player'
          ? ('user' as const)
          : ('assistant' as const),
      content: message.text,
    })),
    max_output_tokens: 400,
  });
  const text = response.output_text.trim();
  if (!text) throw new Error('OpenAI returned an empty reply');

  const reply: ContactMessageDocument = {
    _id: randomUUID(),
    spaceshipSecurityCode: job.spaceshipSecurityCode,
    contactId: job.contactId,
    sender: 'contact',
    text: text.slice(0, 2_000),
    status: 'sent',
    isRead: false,
    clientMessageId: `reply:${job.playerMessageId}`,
    createdAt: new Date(),
  };
  await ContactMessageModel.insert(reply);
  await ContactMessageModel.updateStatus(playerMessage._id, 'sent');
  return reply;
}
