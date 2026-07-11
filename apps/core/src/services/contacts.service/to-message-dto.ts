import type { ContactMessageDocument } from '@models';
import type { ContactMessageDto } from '@repo/types';

export function toMessageDto(
  message: ContactMessageDocument,
): ContactMessageDto {
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
