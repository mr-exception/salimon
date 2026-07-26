import { useEffect, useRef, useState } from 'react';
import {
  getStoredSpaceshipSecurityCode,
  type BootstrapRequest,
  type BootstrapState,
} from '@store';
import style from './style.module.css';

type Props = {
  bootstrapState: BootstrapState;
  onStart: (request: BootstrapRequest) => void;
};

const KNOWN_INDEXED_DB_NAMES = [
  'salimon-world',
  'salimon-world-fetched-sectors',
];

function maskCode(code: string) {
  return `${code.slice(0, 4)}${'•'.repeat(Math.max(8, code.length - 8))}${code.slice(-4)}`;
}

async function deleteDatabase(name: string) {
  return new Promise<void>((resolve, reject) => {
    const request = indexedDB.deleteDatabase(name);

    request.onsuccess = () => resolve();
    request.onerror = () => reject(request.error);
    request.onblocked = () => resolve();
  });
}

async function deleteIndexedDatabases() {
  const discoveredNames =
    typeof indexedDB.databases === 'function'
      ? (await indexedDB.databases()).flatMap((database) =>
          database.name ? [database.name] : [],
        )
      : [];
  const databaseNames = new Set([
    ...KNOWN_INDEXED_DB_NAMES,
    ...discoveredNames,
  ]);

  await Promise.all([...databaseNames].map(deleteDatabase));
}

async function clearLocalBrowserData() {
  localStorage.clear();
  sessionStorage.clear();

  if ('caches' in window) {
    const cacheNames = await caches.keys();
    await Promise.all(cacheNames.map((cacheName) => caches.delete(cacheName)));
  }

  await deleteIndexedDatabases();
}

export default function StartMenu({ bootstrapState, onStart }: Props) {
  const [storedCode] = useState(getStoredSpaceshipSecurityCode);
  const [isCodeVisible, setIsCodeVisible] = useState(false);
  const [isClaimOpen, setIsClaimOpen] = useState(false);
  const [claimCode, setClaimCode] = useState('');
  const [copyLabel, setCopyLabel] = useState('Reveal & copy');
  const [clearLabel, setClearLabel] = useState('Clear local data');
  const dialogRef = useRef<HTMLDialogElement>(null);
  const isLoading = bootstrapState === 'loading';

  useEffect(() => {
    if (isClaimOpen) dialogRef.current?.showModal();
    else dialogRef.current?.close();
  }, [isClaimOpen]);

  const revealAndCopy = async () => {
    if (!storedCode) return;
    setIsCodeVisible(true);
    try {
      await navigator.clipboard.writeText(storedCode);
      setCopyLabel('Copied');
    } catch {
      setCopyLabel('Code revealed');
    }
  };

  const claimShip = () => {
    const securityCode = claimCode.trim();
    if (securityCode) onStart({ type: 'claim', securityCode });
  };

  const clearData = async () => {
    if (
      !window.confirm(
        'Clear local storage, session storage, IndexedDB, and browser caches for this app?',
      )
    ) {
      return;
    }

    setClearLabel('Clearing');
    try {
      await clearLocalBrowserData();
      window.location.reload();
    } catch (error) {
      console.error('Failed to clear local data', error);
      setClearLabel('Clear failed');
    }
  };

  return (
    <main className={style.menu}>
      <div className={style.stars} aria-hidden="true" />
      <section className={style.content} aria-labelledby="game-title">
        <p className={style.eyebrow}>Deep-space navigation protocol</p>
        <h1 id="game-title">
          <span>Salimon</span>
          <small>Echoes of Absenat</small>
        </h1>
        <div className={style.rule} />
        <nav className={style.actions} aria-label="Game menu">
          <button
            type="button"
            onClick={() => onStart({ type: 'new' })}
            disabled={isLoading}
          >
            <span>01</span> Start a new game
          </button>
          <button
            type="button"
            onClick={() => onStart({ type: 'continue' })}
            disabled={!storedCode || isLoading}
          >
            <span>02</span> Continue
          </button>
          <button
            type="button"
            onClick={() => setIsClaimOpen(true)}
            disabled={isLoading}
          >
            <span>03</span> Claim ship
          </button>
          <button
            type="button"
            className={style.dangerAction}
            onClick={() => void clearData()}
            disabled={isLoading}
          >
            <span>04</span> {clearLabel}
          </button>
        </nav>

        {storedCode && (
          <div className={style.identity}>
            <span>Current ship security code</span>
            <button type="button" onClick={() => void revealAndCopy()}>
              <code>{isCodeVisible ? storedCode : maskCode(storedCode)}</code>
              <small>{copyLabel}</small>
            </button>
          </div>
        )}

        <p className={style.status} role="status">
          {isLoading && 'Establishing telemetry link…'}
          {bootstrapState === 'error' &&
            'Connection failed. Verify the security code and try again.'}
        </p>
      </section>
      <p className={style.coordinates}>SOL // 03 · SIGNAL STABLE</p>

      <dialog
        ref={dialogRef}
        className={style.dialog}
        onCancel={() => setIsClaimOpen(false)}
        onClose={() => setIsClaimOpen(false)}
      >
        <form
          onSubmit={(event) => {
            event.preventDefault();
            claimShip();
          }}
        >
          <p className={style.eyebrow}>Ship recovery channel</p>
          <h2>Claim a vessel</h2>
          <p>Enter the security code issued when the ship was registered.</p>
          <label htmlFor="ship-security-code">Security code</label>
          <input
            id="ship-security-code"
            value={claimCode}
            onChange={(event) => setClaimCode(event.target.value)}
            autoComplete="off"
            spellCheck={false}
            autoFocus
          />
          {bootstrapState === 'error' && (
            <p className={style.dialogError} role="alert">
              Ship not found or the recovery channel is unavailable.
            </p>
          )}
          <div className={style.dialogActions}>
            <button type="button" onClick={() => setIsClaimOpen(false)}>
              Cancel
            </button>
            <button type="submit" disabled={!claimCode.trim() || isLoading}>
              Claim & continue
            </button>
          </div>
        </form>
      </dialog>
    </main>
  );
}
