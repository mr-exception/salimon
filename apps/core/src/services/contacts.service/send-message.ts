import { randomUUID } from 'node:crypto';
import { ContactMessageModel, type ContactMessageDocument } from '@models';
import type { ContactShipContext } from './constants';
import { hasContact } from './has-contact';
import { initializeSpaceshipContacts } from './initialize-spaceship-contacts';

export async function sendMessage(
  spaceshipSecurityCode: string,
  request: {
    contactId: string;
    text: string;
    clientMessageId: string;
    shipContext?: ContactShipContext;
  },
  options: {
    onReply?: (message: ContactMessageDocument) => void;
    shipContext?: ContactShipContext;
  } = {},
) {
  const { ContactRepliesService } =
    await import('../contact-replies.service/index.js');

  await initializeSpaceshipContacts(spaceshipSecurityCode);

  if (!(await hasContact(spaceshipSecurityCode, request.contactId))) {
    throw new Error('Contact not found');
  }

  const existing = await ContactMessageModel.findByClientMessage(
    spaceshipSecurityCode,
    request.contactId,
    request.clientMessageId,
  );
  const replyOptions = {
    ...options,
    shipContext: request.shipContext ?? options.shipContext,
  };
  if (existing) {
    if (existing.status === 'failed') {
      ContactRepliesService.generateContactReplyInBackground(
        existing,
        replyOptions,
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
  ContactRepliesService.generateContactReplyInBackground(message, replyOptions);
  message.status = 'sent';
  await ContactMessageModel.updateStatus(message._id, 'sent');

  return message;
}
