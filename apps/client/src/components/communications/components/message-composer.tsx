import style from '../style.module.css';

type Props = {
  draft: string;
  isDisabled: boolean;
  isSending: boolean;
  onDraftChange: (draft: string) => void;
  onSendMessage: () => void;
};

export function MessageComposer({
  draft,
  isDisabled,
  isSending,
  onDraftChange,
  onSendMessage,
}: Props) {
  return (
    <form
      className={style.composer}
      onSubmit={(event) => {
        event.preventDefault();
        onSendMessage();
      }}
    >
      <label htmlFor="communications-message">Message</label>
      <textarea
        id="communications-message"
        value={draft}
        maxLength={1_000}
        placeholder="Message the Chief…"
        onChange={(event) => onDraftChange(event.currentTarget.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            onSendMessage();
          }
        }}
      />
      <button type="submit" disabled={isDisabled}>
        {isSending ? 'Sending…' : 'Send'}
      </button>
    </form>
  );
}
