export type ContactProfile = {
  id: string;
  name: string;
  description: string;
  position: string;
  organization: string;
  role: string;
};

export type ContactInfo = ContactProfile & {
  unreadCount: number;
  lastMessageAt?: string;
};

export type ContactMessageSender = 'player' | 'contact';

export type ContactMessageStatus = 'sent' | 'queued' | 'failed';

export type ContactMessageDto = {
  id: string;
  contactId: string;
  sender: ContactMessageSender;
  text: string;
  status: ContactMessageStatus;
  isRead: boolean;
  createdAt: string;
};
