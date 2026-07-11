import { randomUUID } from 'node:crypto';
import { ContactModel, ContactMessageModel } from '@models';
import { EASA_CHIEF_ID, INITIAL_CHIEF_MESSAGE } from './constants';

export async function initializeSpaceshipContacts(
  spaceshipSecurityCode: string,
) {
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

