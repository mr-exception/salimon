import { randomUUID } from 'node:crypto';
import OpenAI from 'openai';
import {
  ContactMessageModel,
  type ContactMessageDocument,
} from '@models';
import { CONTACTS, EASA_CHIEF_ID } from './contacts.service';

type ReplyJob = {
  spaceshipSecurityCode: string;
  contactId: string;
  playerMessageId: string;
};

const MAX_CONTEXT_MESSAGES = 30;

const CHIEF_INSTRUCTIONS = `You are the Chief of EASA (Earth Aeronautics and Space Administration) in Salimon: Echoes of Absenat.
You are speaking privately with a human spaceship pilot.
Known canon:
- An unknown being contacted humanity and gave it an energy cube.
- Humanity named the cube "the Core".
- The Core produces oxygen and electricity that spaceships can harvest.
- The being's exact message was: "Reach Absenat, where the world is going to start."
- The pilot's mission is to reach Absenat.
- The being, the Core's origin, Absenat, and the message's deeper meaning are unknown.
Never invent confirmed answers to those mysteries. Clearly distinguish speculation from known facts.
Stay in character, be concise, pragmatic, and mission-focused.
Do not claim to change ship resources, navigation, contacts, or any other game state.`;

let openai: OpenAI | undefined;

function getOpenAI() {
  openai ??= new OpenAI();
  return openai;
}

export class ContactRepliesService {
  static async generateContactReply(job: ReplyJob) {
    const profile = CONTACTS[job.contactId as keyof typeof CONTACTS];
    if (!profile || job.contactId !== EASA_CHIEF_ID) {
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

    const history = await (await ContactMessageModel.getCollection())
      .find({
        spaceshipSecurityCode: job.spaceshipSecurityCode,
        contactId: job.contactId,
        status: { $ne: 'failed' },
      })
      .sort({ createdAt: -1, _id: -1 })
      .limit(MAX_CONTEXT_MESSAGES)
      .toArray();

    const response = await getOpenAI().responses.create({
      model: process.env.OPENAI_MODEL ?? 'gpt-5.4-mini',
      instructions: CHIEF_INSTRUCTIONS,
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
  }

  static generateContactReplyInBackground(message: ContactMessageDocument) {
    void ContactRepliesService.generateContactReply({
      spaceshipSecurityCode: message.spaceshipSecurityCode,
      contactId: message.contactId,
      playerMessageId: message._id,
    }).catch(async (error: unknown) => {
      console.error('Failed to generate contact reply', error);
      await ContactMessageModel.updateStatus(message._id, 'failed');
    });
  }
}
