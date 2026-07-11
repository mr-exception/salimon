import { useCallback, useEffect, useRef, useState } from 'react';
import axios from 'axios';
import {
  getApiBaseUrl,
  getStoredSpaceshipSecurityCode,
  SECURITY_CODE_HEADER,
  sendContactMessageOverSocket,
  subscribeToContactMessages,
} from '@store';
import { CommunicationsShell } from './components/communications-shell';
import { ContactsList } from './components/contacts-list';
import { ConversationPanel } from './components/conversation-panel';
import type { Contact, Message } from './types';
import { mergeMessages } from './utils';

type Props = {
  onClose: () => void;
  unreadMessages: Message[];
  onMessagesRead: (messageIds: string[]) => void;
};

export function Communications({
  onClose,
  unreadMessages,
  onMessagesRead,
}: Props) {
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [selectedContactId, setSelectedContactId] = useState<string>();
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isSending, setIsSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const cursorRef = useRef<string | undefined>(undefined);
  const selectedContactIdRef = useRef<string | undefined>(undefined);
  const securityCode = getStoredSpaceshipSecurityCode();

  useEffect(() => {
    selectedContactIdRef.current = selectedContactId;
  }, [selectedContactId]);

  const loadContacts = useCallback(async () => {
    if (!securityCode) return;
    const { data } = await axios.get<{ contacts: Contact[] }>(
      `${getApiBaseUrl()}/contacts/info`,
      { headers: { [SECURITY_CODE_HEADER]: securityCode } },
    );
    setContacts(data.contacts);
  }, [securityCode]);

  const loadMessages = useCallback(
    async (contactId: string, after?: string) => {
      if (!securityCode) return;
      const { data } = await axios.get<{
        messages: Message[];
        cursor?: string;
      }>(`${getApiBaseUrl()}/contacts/messages`, {
        headers: { [SECURITY_CODE_HEADER]: securityCode },
        params: { contactId, ...(after ? { after } : {}) },
      });
      setMessages((current) =>
        after ? mergeMessages(current, data.messages) : data.messages,
      );
      onMessagesRead(
        data.messages
          .filter((message) => message.sender === 'contact')
          .map((message) => message.id),
      );
      cursorRef.current = data.cursor;
      setError('');
    },
    [onMessagesRead, securityCode],
  );

  useEffect(() => {
    let disposed = false;
    void loadContacts()
      .catch(() => setError('Unable to establish communications.'))
      .finally(() => {
        if (!disposed) setIsLoading(false);
      });

    return () => {
      disposed = true;
    };
  }, [loadContacts]);

  useEffect(
    () =>
      subscribeToContactMessages((message) => {
        if (message.contactId === selectedContactIdRef.current) {
          setMessages((current) => mergeMessages(current, [message]));
          if (message.sender === 'contact') onMessagesRead([message.id]);
        }
        void loadContacts().catch(() => {
          setError('Unable to refresh contacts.');
        });
      }),
    [loadContacts, onMessagesRead],
  );

  useEffect(() => {
    if (!selectedContactId) return;
    const timer = window.setTimeout(() => {
      setMessages([]);
      cursorRef.current = undefined;
      setIsLoading(true);
      void loadMessages(selectedContactId)
        .catch(() => setError('Unable to load messages.'))
        .finally(() => setIsLoading(false));
    }, 0);
    return () => window.clearTimeout(timer);
  }, [loadMessages, selectedContactId]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const sendMessage = async () => {
    const text = draft.trim();
    if (!selectedContactId || !securityCode || !text || isSending) return;
    setIsSending(true);
    setError('');
    try {
      const message = await sendContactMessageOverSocket({
        contactId: selectedContactId,
        text,
        clientMessageId: crypto.randomUUID(),
      });
      setDraft('');
      setMessages((current) => mergeMessages(current, [message]));
    } catch {
      setError('Message failed to send. Try again.');
    } finally {
      setIsSending(false);
    }
  };

  const selectedContact = contacts.find(
    (contact) => contact.id === selectedContactId,
  );
  const unreadCountByContact = unreadMessages.reduce<Map<string, number>>(
    (counts, message) =>
      counts.set(message.contactId, (counts.get(message.contactId) ?? 0) + 1),
    new Map(),
  );

  return (
    <CommunicationsShell onClose={onClose}>
      <ContactsList
        contacts={contacts}
        selectedContactId={selectedContactId}
        unreadCountByContact={unreadCountByContact}
        onSelectContact={setSelectedContactId}
      />
      <ConversationPanel
        draft={draft}
        error={error}
        isLoading={isLoading}
        isSending={isSending}
        messages={messages}
        messagesEndRef={messagesEndRef}
        selectedContact={selectedContact}
        selectedContactId={selectedContactId}
        onDraftChange={setDraft}
        onSendMessage={() => {
          void sendMessage();
        }}
      />
    </CommunicationsShell>
  );
}
