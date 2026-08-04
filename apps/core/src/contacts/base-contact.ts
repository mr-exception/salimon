import { randomUUID } from 'node:crypto';
import { ContactMessageModel, ContactModel } from '@models';
import type { ContactMessageDocument } from '@models';
import type {
  ContactDocumentCollection,
  ContactMessageRequest,
  ContactProfile,
  ContactReplyOptions,
} from './types';

export abstract class BaseContact implements ContactProfile {
  abstract readonly id: string;
  abstract readonly name: string;
  abstract readonly description: string;
  abstract readonly position: string;
  abstract readonly organization: string;
  abstract readonly role: string;
  abstract readonly documents: ContactDocumentCollection;
  abstract readonly background: readonly string[];
  abstract readonly worldGoal: readonly string[];
  abstract readonly personality: readonly string[];
  abstract readonly speakingStyle: readonly string[];
  abstract readonly knownCanon: readonly string[];
  abstract readonly unknowns: readonly string[];
  abstract readonly boundaries: readonly string[];
  abstract readonly initialMessage: string;

  async sendMessage(
    spaceshipSecurityCode: string,
    request: Omit<ContactMessageRequest, 'contactId'>,
    options: ContactReplyOptions = {},
  ) {
    const { sendMessage } =
      await import('../services/contacts.service/send-message.js');
    return sendMessage(
      spaceshipSecurityCode,
      {
        contactId: this.id,
        text: request.text,
        clientMessageId: request.clientMessageId,
        shipContext: request.shipContext,
      },
      options,
    );
  }

  async getResponse(spaceshipSecurityCode: string) {
    return ContactMessageModel.findLatestContactMessage(
      spaceshipSecurityCode,
      this.id,
    );
  }

  async triggerFirstMessage(spaceshipSecurityCode: string) {
    const now = new Date();

    await ContactModel.upsertSpaceshipContact({
      spaceshipSecurityCode,
      contactId: this.id,
      unlockedAt: now,
    });

    const message: ContactMessageDocument = {
      _id: randomUUID(),
      spaceshipSecurityCode,
      contactId: this.id,
      sender: 'contact',
      text: this.initialMessage,
      status: 'sent',
      isRead: false,
      clientMessageId: 'initial-briefing',
      createdAt: now,
    };

    await ContactMessageModel.upsertInitialMessage(message);
    return message;
  }

  toProfile(): ContactProfile {
    return {
      id: this.id,
      name: this.name,
      description: this.description,
      position: this.position,
      organization: this.organization,
      role: this.role,
      documents: this.documents,
      background: this.background,
      worldGoal: this.worldGoal,
      personality: this.personality,
      speakingStyle: this.speakingStyle,
      knownCanon: this.knownCanon,
      unknowns: this.unknowns,
      boundaries: this.boundaries,
      initialMessage: this.initialMessage,
    };
  }
}
