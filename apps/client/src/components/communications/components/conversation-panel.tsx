import type { RefObject } from 'react';
import { ContactHeading } from './contact-heading';
import { MessageComposer } from './message-composer';
import { MessageList } from './message-list';
import type { Contact, Message } from '../types';
import style from '../style.module.css';

type Props = {
  draft: string;
  error: string;
  isLoading: boolean;
  isSending: boolean;
  messages: Message[];
  messagesEndRef: RefObject<HTMLDivElement | null>;
  selectedContact?: Contact;
  selectedContactId?: string;
  onDraftChange: (draft: string) => void;
  onSendMessage: () => void;
};

export function ConversationPanel({
  draft,
  error,
  isLoading,
  isSending,
  messages,
  messagesEndRef,
  selectedContact,
  selectedContactId,
  onDraftChange,
  onSendMessage,
}: Props) {
  return (
    <div className={style.conversation}>
      <ContactHeading contact={selectedContact} />
      <MessageList
        isLoading={isLoading}
        messages={messages}
        messagesEndRef={messagesEndRef}
      />
      <MessageComposer
        draft={draft}
        isDisabled={!draft.trim() || isSending || !selectedContactId}
        isSending={isSending}
        onDraftChange={onDraftChange}
        onSendMessage={onSendMessage}
      />
      <p className={style.error} role="status">
        {!navigator.onLine ? 'Communications offline.' : error}
      </p>
    </div>
  );
}
