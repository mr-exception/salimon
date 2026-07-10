import { useCallback, useEffect, useRef, useState } from 'react';
import axios from 'axios';
import {
  getApiBaseUrl,
  getStoredSpaceshipSecurityCode,
  SECURITY_CODE_HEADER,
  sendContactMessageOverSocket,
  subscribeToContactMessages,
  type ContactMessage,
} from '@store';
import style from './style.module.css';

type Contact = {
  id: string;
  name: string;
  organization: string;
  unreadCount: number;
  lastMessageAt?: string;
};

type Message = ContactMessage;

type Props = {
  onClose: () => void;
  unreadMessages: Message[];
  onMessagesRead: (messageIds: string[]) => void;
};

const CONTACT_POLL_MS = 30_000;
const MAX_RETRY_MS = 60_000;

function mergeMessages(current: Message[], incoming: Message[]) {
  const byId = new Map(current.map((message) => [message.id, message]));
  incoming.forEach((message) => byId.set(message.id, message));
  return [...byId.values()].sort(
    (left, right) => Date.parse(left.createdAt) - Date.parse(right.createdAt),
  );
}

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
    let retryDelay = CONTACT_POLL_MS;
    let contactTimer: number | undefined;

    const canPoll = () => !document.hidden && navigator.onLine;
    const pollContacts = async () => {
      if (disposed) return;
      if (canPoll()) {
        try {
          await loadContacts();
          retryDelay = CONTACT_POLL_MS;
        } catch {
          retryDelay = Math.min(retryDelay * 2, MAX_RETRY_MS);
        }
      }
      contactTimer = window.setTimeout(
        pollContacts,
        canPoll() ? CONTACT_POLL_MS : MAX_RETRY_MS,
      );
    };
    const pollNow = () => {
      window.clearTimeout(contactTimer);
      void pollContacts();
    };

    const initialTimer = window.setTimeout(() => {
      void loadContacts()
        .catch(() => setError('Unable to establish communications.'))
        .finally(() => {
          if (!disposed) setIsLoading(false);
        });
      contactTimer = window.setTimeout(pollContacts, CONTACT_POLL_MS);
    }, 0);
    document.addEventListener('visibilitychange', pollNow);
    window.addEventListener('online', pollNow);
    return () => {
      disposed = true;
      window.clearTimeout(initialTimer);
      window.clearTimeout(contactTimer);
      document.removeEventListener('visibilitychange', pollNow);
      window.removeEventListener('online', pollNow);
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
    <div className={style.backdrop} role="presentation">
      <section
        className={style.dialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby="communications-title"
      >
        <header className={style.header}>
          <div>
            <small>Ship communications</small>
            <h2 id="communications-title">Contacts</h2>
          </div>
          <button type="button" onClick={onClose} aria-label="Close">
            ×
          </button>
        </header>

        <div className={style.content}>
          <nav className={style.contacts} aria-label="Known contacts">
            {contacts.map((contact) => {
              const unreadCount = unreadCountByContact.get(contact.id) ?? 0;
              return (
                <button
                  type="button"
                  key={contact.id}
                  data-active={contact.id === selectedContactId}
                  data-unread={unreadCount > 0}
                  onClick={() => setSelectedContactId(contact.id)}
                >
                  <span>{contact.name}</span>
                  <small>{contact.organization}</small>
                  {unreadCount > 0 && (
                    <strong aria-label={`${unreadCount} unread`}>
                      {unreadCount}
                    </strong>
                  )}
                </button>
              );
            })}
          </nav>

          <div className={style.conversation}>
            <div className={style.contactHeading}>
              <strong>{selectedContact?.name ?? 'No contact selected'}</strong>
              <small>{selectedContact?.organization}</small>
            </div>
            <div className={style.messages} aria-live="polite">
              {isLoading && <p className={style.notice}>Loading messages…</p>}
              {!isLoading &&
                messages.map((message) => (
                  <article
                    className={style.message}
                    data-sender={message.sender}
                    key={message.id}
                  >
                    <p>{message.text}</p>
                    <small>
                      {new Date(message.createdAt).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                      {message.status === 'queued' ? ' · awaiting reply' : ''}
                    </small>
                  </article>
                ))}
              <div ref={messagesEndRef} />
            </div>
            <form
              className={style.composer}
              onSubmit={(event) => {
                event.preventDefault();
                void sendMessage();
              }}
            >
              <label htmlFor="communications-message">Message</label>
              <textarea
                id="communications-message"
                value={draft}
                maxLength={1_000}
                placeholder="Message the Chief…"
                onChange={(event) => setDraft(event.currentTarget.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter' && !event.shiftKey) {
                    event.preventDefault();
                    void sendMessage();
                  }
                }}
              />
              <button
                type="submit"
                disabled={!draft.trim() || isSending || !selectedContactId}
              >
                {isSending ? 'Sending…' : 'Send'}
              </button>
            </form>
            <p className={style.error} role="status">
              {!navigator.onLine ? 'Communications offline.' : error}
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}
