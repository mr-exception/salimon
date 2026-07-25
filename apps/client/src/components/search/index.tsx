import { useEffect, useRef, useState } from 'react';
import axios from 'axios';
import {
  getApiBaseUrl,
  getStoredSpaceshipSecurityCode,
  SECURITY_CODE_HEADER,
} from '@store';
import style from './style.module.css';

export type SearchResult = {
  name: string;
  kind: 'planet' | 'moon' | 'star';
  navigationZoom: number;
};

type SearchResponse = {
  results: SearchResult[];
};

type Props = {
  onClose: () => void;
  onSelect: (result: SearchResult) => void;
};

const KIND_LABELS: Record<SearchResult['kind'], string> = {
  planet: 'Planet',
  moon: 'Moon',
  star: 'Star',
};

export function SearchDialog({ onClose, onSelect }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const normalizedQuery = query.trim();

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    const dismissOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };

    window.addEventListener('keydown', dismissOnEscape);
    return () => window.removeEventListener('keydown', dismissOnEscape);
  }, [onClose]);

  useEffect(() => {
    if (!normalizedQuery) return;

    const securityCode = getStoredSpaceshipSecurityCode();
    const controller = new AbortController();
    const timer = window.setTimeout(() => {
      setIsLoading(true);
      setError('');
      void axios
        .get<SearchResponse>(`${getApiBaseUrl()}/search`, {
          params: { q: normalizedQuery },
          signal: controller.signal,
          headers: securityCode
            ? { [SECURITY_CODE_HEADER]: securityCode }
            : undefined,
        })
        .then(({ data }) => setResults(data.results))
        .catch((requestError: unknown) => {
          if (axios.isCancel(requestError)) return;
          setError('Search failed.');
          setResults([]);
        })
        .finally(() => {
          if (!controller.signal.aborted) setIsLoading(false);
        });
    }, 180);

    return () => {
      window.clearTimeout(timer);
      controller.abort();
    };
  }, [normalizedQuery]);

  const chooseResult = (result: SearchResult) => {
    onSelect(result);
    onClose();
  };

  const updateQuery = (value: string) => {
    setQuery(value);
    setError('');

    if (!value.trim()) {
      setResults([]);
      setIsLoading(false);
    }
  };

  return (
    <div className={style.backdrop}>
      <button
        className={style.overlay}
        type="button"
        aria-label="Close search"
        onClick={onClose}
      />
      <section
        className={style.dialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby="search-dialog-title"
      >
        <header className={style.header}>
          <div>
            <small>Navigator</small>
            <h2 id="search-dialog-title">Search</h2>
          </div>
          <button type="button" aria-label="Close search" onClick={onClose}>
            ×
          </button>
        </header>
        <div className={style.content}>
          <label className={style.searchLabel} htmlFor="navigator-search">
            Find by name
          </label>
          <div className={style.searchField}>
            <span className={style.searchIcon} aria-hidden="true" />
            <input
              ref={inputRef}
              id="navigator-search"
              type="search"
              value={query}
              placeholder="Search planets, moons, and stars"
              autoComplete="off"
              onChange={(event) => updateQuery(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' && results[0]) {
                  chooseResult(results[0]);
                }
              }}
              aria-controls="navigator-search-results"
            />
          </div>
          <ul
            id="navigator-search-results"
            className={style.results}
            aria-label="Search results"
          >
            {normalizedQuery && results.length > 0
              ? results.map((result) => (
                  <li key={`${result.kind}-${result.name}`}>
                    <button type="button" onClick={() => chooseResult(result)}>
                      <span className={style.resultMarker} aria-hidden="true" />
                      <span>{result.name}</span>
                      <small>{KIND_LABELS[result.kind]}</small>
                    </button>
                  </li>
                ))
              : null}
            {normalizedQuery &&
              !isLoading &&
              results.length === 0 &&
              !error && (
                <li className={style.notice}>No celestial bodies found</li>
              )}
            {isLoading && <li className={style.notice}>Searching...</li>}
            {error && (
              <li className={style.error} role="alert">
                {error}
              </li>
            )}
          </ul>
        </div>
      </section>
    </div>
  );
}
