import { useCallback, useEffect, useRef, useState } from 'react';
import axios from 'axios';
import {
  getApiBaseUrl,
  getStoredSpaceshipSecurityCode,
  getSpaceshipDto,
  getSpaceshipProximityTelemetry,
  SECURITY_CODE_HEADER,
  loadCachedContactMessages,
  markContactThreadRead,
  markCachedContactMessagesRead,
  sendContactMessage,
  subscribeToContactMessages,
  storeCachedContactMessage,
  storeCachedContactMessages,
} from '@store';
import { CommunicationsShell } from './components/communications-shell';
import { ContactsList } from './components/contacts-list';
import { ConversationPanel } from './components/conversation-panel';
import type { Contact, Message } from './types';
import { mergeMessages } from './utils';

type Props = {
  onClose: () => void;
  initialContactId?: string;
  unreadMessages: Message[];
  onMessagesRead: (messageIds: string[]) => void;
};

export function Communications({
  onClose,
  initialContactId,
  unreadMessages,
  onMessagesRead,
}: Props) {
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [selectedContactId, setSelectedContactId] = useState<
    string | undefined
  >(initialContactId);
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isSending, setIsSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const cursorRef = useRef<string | undefined>(undefined);
  const selectedContactIdRef = useRef<string | undefined>(undefined);
  const unreadMessagesRef = useRef<Message[]>(unreadMessages);
  const securityCode = getStoredSpaceshipSecurityCode();

  useEffect(() => {
    unreadMessagesRef.current = unreadMessages;
  }, [unreadMessages]);

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
      if (!after) {
        try {
          const cachedMessages = await loadCachedContactMessages(
            securityCode,
            contactId,
          );
          setMessages(cachedMessages);
        } catch (error) {
          console.error('Failed to load cached contact messages', error);
        }
      }
      const { data } = await axios.get<{
        messages: Message[];
        cursor?: string;
      }>(`${getApiBaseUrl()}/contacts/messages`, {
        headers: { [SECURITY_CODE_HEADER]: securityCode },
        params: { contactId, ...(after ? { after } : {}) },
      });
      await storeCachedContactMessages(securityCode, data.messages);
      setMessages((current) =>
        after ? mergeMessages(current, data.messages) : data.messages,
      );
      if (!after) {
        const readMessageIds = unreadMessagesRef.current
          .filter((message) => message.contactId === contactId)
          .map((message) => message.id);
        await markContactThreadRead(contactId);
        await markCachedContactMessagesRead(
          securityCode,
          contactId,
          readMessageIds,
        );
        onMessagesRead(readMessageIds);
        void loadContacts().catch(() => {
          setError('Unable to refresh contacts.');
        });
      }
      cursorRef.current = data.cursor;
      setError('');
    },
    [loadContacts, onMessagesRead, securityCode],
  );

  useEffect(() => {
    let disposed = false;
    const timer = window.setTimeout(() => {
      void loadContacts()
        .catch(() => setError('Unable to establish communications.'))
        .finally(() => {
          if (!disposed) setIsLoading(false);
        });
    }, 0);

    return () => {
      disposed = true;
      window.clearTimeout(timer);
    };
  }, [loadContacts]);

  useEffect(
    () =>
      subscribeToContactMessages((message) => {
        if (securityCode) {
          void storeCachedContactMessage(securityCode, message).catch(
            (error: unknown) => {
              console.error('Failed to cache contact message', error);
            },
          );
        }
        if (message.contactId === selectedContactIdRef.current) {
          setMessages((current) => mergeMessages(current, [message]));
          if (message.sender === 'contact') {
            void markContactThreadRead(message.contactId)
              .then(() => {
                if (securityCode) {
                  void markCachedContactMessagesRead(
                    securityCode,
                    message.contactId,
                    [message.id],
                  ).catch((error: unknown) => {
                    console.error('Failed to mark cached message read', error);
                  });
                }
                onMessagesRead([message.id]);
                void loadContacts().catch(() => {
                  setError('Unable to refresh contacts.');
                });
              })
              .catch(() => {
                setError('Unable to mark message read.');
              });
          }
        }
        void loadContacts().catch(() => {
          setError('Unable to refresh contacts.');
        });
      }),
    [loadContacts, onMessagesRead, securityCode],
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
    if (!selectedContactId) return;
    const timer = window.setInterval(() => {
      void loadMessages(selectedContactId, cursorRef.current).catch(() => {
        setError('Unable to refresh messages.');
      });
    }, 10_000);

    return () => window.clearInterval(timer);
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
      const message = await sendContactMessage({
        contactId: selectedContactId,
        text,
        clientMessageId: crypto.randomUUID(),
        shipContext: {
          ...getSpaceshipDto(securityCode),
          proximityTelemetry: getSpaceshipProximityTelemetry(),
        },
      });
      await storeCachedContactMessage(securityCode, message);
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
