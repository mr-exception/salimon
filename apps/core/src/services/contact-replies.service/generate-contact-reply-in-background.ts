import { ContactMessageModel, type ContactMessageDocument } from '@models';
import type { ContactReplyOptions } from '../contacts.service';
import { generateContactReply } from './generate-contact-reply';

export function generateContactReplyInBackground(
  message: ContactMessageDocument,
  options: ContactReplyOptions = {},
) {
  void generateContactReply({
    spaceshipSecurityCode: message.spaceshipSecurityCode,
    contactId: message.contactId,
    playerMessageId: message._id,
    shipContext: options.shipContext,
  })
    .then((reply) => {
      if (reply) options.onReply?.(reply);
    })
    .catch(async (error: unknown) => {
      console.error('Failed to generate contact reply', error);
      await ContactMessageModel.updateStatus(message._id, 'failed');
    });
}
