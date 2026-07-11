import { ContactMessageModel, type ContactMessageDocument } from '@models';
import { generateContactReply } from './generate-contact-reply';

export function generateContactReplyInBackground(
  message: ContactMessageDocument,
  options: {
    onReply?: (message: ContactMessageDocument) => void;
  } = {},
) {
  void generateContactReply({
    spaceshipSecurityCode: message.spaceshipSecurityCode,
    contactId: message.contactId,
    playerMessageId: message._id,
  })
    .then((reply) => {
      if (reply) options.onReply?.(reply);
    })
    .catch(async (error: unknown) => {
      console.error('Failed to generate contact reply', error);
      await ContactMessageModel.updateStatus(message._id, 'failed');
    });
}

