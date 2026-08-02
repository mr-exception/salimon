import type { PropsWithChildren } from 'react';
import style from '../style.module.css';

type Props = PropsWithChildren<{
  onClose: () => void;
}>;

export function CommunicationsShell({ children, onClose }: Props) {
  return (
    <div className={style.backdrop} role="presentation">
      <section
        className={style.dialog}
        data-tutorial-target="communications-dialog"
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

        <div className={style.content}>{children}</div>
      </section>
    </div>
  );
}
