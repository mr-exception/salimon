import type { RefObject } from 'react';
import type { Message } from '../types';
import style from '../style.module.css';

type Props = {
  isLoading: boolean;
  messages: Message[];
  messagesEndRef: RefObject<HTMLDivElement | null>;
};

export function MessageList({ isLoading, messages, messagesEndRef }: Props) {
  return (
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
  );
}
