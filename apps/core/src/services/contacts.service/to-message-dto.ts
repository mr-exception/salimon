import type { ContactMessageDocument } from '@models';

export function toMessageDto(message: ContactMessageDocument) {
  return {
    id: message._id,
    contactId: message.contactId,
    sender: message.sender,
    text: message.text,
    status: message.status,
    isRead: message.isRead,
    createdAt: message.createdAt.toISOString(),
  };
}

