import { useEffect } from 'react';
import type { BodyContextMenuRequest } from './game/scene';
import style from './style.module.css';

type BodyContextMenuProps = {
  request: BodyContextMenuRequest;
  onDismiss: () => void;
  onToggleAlwaysVisible: () => void;
};

export function BodyContextMenu({
  request,
  onDismiss,
  onToggleAlwaysVisible,
}: BodyContextMenuProps) {
  useEffect(() => {
    const dismissOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onDismiss();
    };

    window.addEventListener('keydown', dismissOnEscape);
    window.addEventListener('resize', onDismiss);

    return () => {
      window.removeEventListener('keydown', dismissOnEscape);
      window.removeEventListener('resize', onDismiss);
    };
  }, [onDismiss]);

  return (
    <>
      <button
        type="button"
        className={style.contextMenuBackdrop}
        aria-label="Close context menu"
        onClick={onDismiss}
        onContextMenu={(event) => {
          event.preventDefault();
          onDismiss();
        }}
      />
      <div
        className={style.bodyContextMenu}
        style={{ left: request.x, top: request.y }}
        role="menu"
        aria-label={`${request.name} options`}
        onContextMenu={(event) => event.preventDefault()}
      >
        <header>
          <span>{request.name}</span>
          <small>{request.kind}</small>
        </header>
        <button
          type="button"
          role="menuitemcheckbox"
          aria-checked={request.alwaysVisible}
          onClick={onToggleAlwaysVisible}
        >
          <span className={style.contextMenuCheck} aria-hidden="true">
            {request.alwaysVisible ? '✓' : ''}
          </span>
          Always show name
        </button>
      </div>
    </>
  );
}
