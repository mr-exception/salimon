import type { RefObject } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
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
            <MessageText message={message} />
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

function MessageText({ message }: { message: Message }) {
  if (message.sender === 'player') {
    return <p>{message.text}</p>;
  }

  return (
    <div className={style.markdown}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a: ({ children, ...props }) => (
            <a {...props} rel="noreferrer" target="_blank">
              {children}
            </a>
          ),
        }}
      >
        {message.text}
      </ReactMarkdown>
    </div>
  );
}
