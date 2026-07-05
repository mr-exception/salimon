import { useCallback, useEffect, useRef, useState } from 'react';
import axios from 'axios';
import {
  getApiBaseUrl,
  getStoredSpaceshipSecurityCode,
  SECURITY_CODE_HEADER,
} from '@store';
import style from './style.module.css';

type Contact = {
  id: string;
  name: string;
  organization: string;
  unreadCount: number;
  lastMessageAt?: string;
};

type Message = {
  id: string;
  contactId: string;
  sender: 'player' | 'contact';
  text: string;
  status: 'sent' | 'queued' | 'failed';
  createdAt: string;
};

type Props = {
  onClose: () => void;
};

const MESSAGE_POLL_MS = 5_000;
const CONTACT_POLL_MS = 30_000;
const MAX_RETRY_MS = 60_000;

function mergeMessages(current: Message[], incoming: Message[]) {
  const byId = new Map(current.map((message) => [message.id, message]));
  incoming.forEach((message) => byId.set(message.id, message));
  return [...byId.values()].sort(
    (left, right) => Date.parse(left.createdAt) - Date.parse(right.createdAt),
  );
}

export function Communications({ onClose }: Props) {
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [selectedContactId, setSelectedContactId] = useState<string>();
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isSending, setIsSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const cursorRef = useRef<string | undefined>(undefined);
  const securityCode = getStoredSpaceshipSecurityCode();

  const loadContacts = useCallback(async () => {
    if (!securityCode) return;
    const { data } = await axios.get<{ contacts: Contact[] }>(
      `${getApiBaseUrl()}/contacts/info`,
      { headers: { [SECURITY_CODE_HEADER]: securityCode } },
    );
    setContacts(data.contacts);
    setSelectedContactId((current) => current ?? data.contacts[0]?.id);
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
      cursorRef.current = data.cursor;
      setError('');
    },
    [securityCode],
  );

  useEffect(() => {
    let disposed = false;
    let retryDelay = MESSAGE_POLL_MS;
    let contactTimer: number | undefined;
    let messageTimer: number | undefined;

    const canPoll = () => !document.hidden && navigator.onLine;
    const pollContacts = async () => {
      if (disposed) return;
      if (canPoll()) {
        try {
          await loadContacts();
          retryDelay = MESSAGE_POLL_MS;
        } catch {
          retryDelay = Math.min(retryDelay * 2, MAX_RETRY_MS);
        }
      }
      contactTimer = window.setTimeout(
        pollContacts,
        canPoll() ? CONTACT_POLL_MS : MAX_RETRY_MS,
      );
    };
    const pollMessages = async () => {
      if (disposed) return;
      if (selectedContactId && canPoll()) {
        try {
          await loadMessages(selectedContactId, cursorRef.current);
          retryDelay = MESSAGE_POLL_MS;
        } catch {
          retryDelay = Math.min(retryDelay * 2, MAX_RETRY_MS);
        }
      }
      messageTimer = window.setTimeout(
        pollMessages,
        retryDelay + Math.random() * 500,
      );
    };
    const pollNow = () => {
      window.clearTimeout(contactTimer);
      window.clearTimeout(messageTimer);
      void pollContacts();
      void pollMessages();
    };

    const initialTimer = window.setTimeout(() => {
      void loadContacts()
        .catch(() => setError('Unable to establish communications.'))
        .finally(() => {
          if (!disposed) setIsLoading(false);
        });
      void pollMessages();
      contactTimer = window.setTimeout(pollContacts, CONTACT_POLL_MS);
    }, 0);
    document.addEventListener('visibilitychange', pollNow);
    window.addEventListener('online', pollNow);
    return () => {
      disposed = true;
      window.clearTimeout(initialTimer);
      window.clearTimeout(contactTimer);
      window.clearTimeout(messageTimer);
      document.removeEventListener('visibilitychange', pollNow);
      window.removeEventListener('online', pollNow);
    };
  }, [loadContacts, loadMessages, selectedContactId]);

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
      const { data } = await axios.post<{ message: Message }>(
        `${getApiBaseUrl()}/contacts/messages/send`,
        {
          contactId: selectedContactId,
          text,
          clientMessageId: crypto.randomUUID(),
        },
        { headers: { [SECURITY_CODE_HEADER]: securityCode } },
      );
      setDraft('');
      setMessages((current) => mergeMessages(current, [data.message]));
      await loadMessages(selectedContactId, cursorRef.current);
    } catch {
      setError('Message failed to send. Try again.');
    } finally {
      setIsSending(false);
    }
  };

  const selectedContact = contacts.find(
    (contact) => contact.id === selectedContactId,
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
            {contacts.map((contact) => (
              <button
                type="button"
                key={contact.id}
                data-active={contact.id === selectedContactId}
                onClick={() => setSelectedContactId(contact.id)}
              >
                <span>{contact.name}</span>
                <small>{contact.organization}</small>
                {contact.unreadCount > 0 && (
                  <strong aria-label={`${contact.unreadCount} unread`}>
                    {contact.unreadCount}
                  </strong>
                )}
              </button>
            ))}
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
