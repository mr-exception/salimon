import { useMemo, useState } from 'react';
import type { Planet, Star } from '@repo/types';
import style from './style.module.css';

export type SearchResult =
  | { body: Planet; kind: 'Planet' }
  | { body: Star; kind: 'Star' };

type SearchPanelProps = {
  planets: Planet[];
  stars: Star[];
  onSelect: (result: SearchResult) => void;
};

export function SearchPanel({ planets, stars, onSelect }: SearchPanelProps) {
  const [query, setQuery] = useState('');
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const searchIndex = useMemo(
    () => [
      ...planets.map((body) => ({
        body,
        kind: 'Planet' as const,
        nameLower: body.name.toLocaleLowerCase(),
      })),
      ...stars.map((body) => ({
        body,
        kind: 'Star' as const,
        nameLower: body.name.toLocaleLowerCase(),
      })),
    ],
    [planets, stars],
  );
  const results = useMemo<SearchResult[]>(() => {
    if (!normalizedQuery) return [];

    return searchIndex
      .filter(({ nameLower }) => nameLower.includes(normalizedQuery))
      .sort((a, b) => {
        const aStartsWith = a.nameLower.startsWith(normalizedQuery);
        const bStartsWith = b.nameLower.startsWith(normalizedQuery);

        return (
          Number(bStartsWith) - Number(aStartsWith) ||
          a.body.name.localeCompare(b.body.name)
        );
      })
      .map(({ body, kind }) => ({ body, kind }));
  }, [normalizedQuery, searchIndex]);

  const chooseResult = (result: SearchResult) => {
    onSelect(result);
    setQuery('');
  };

  return (
    <div className={style.searchPanel}>
      <label className={style.searchLabel} htmlFor="navigator-search">
        Find a celestial body
      </label>
      <div className={style.searchField}>
        <span className={style.searchIcon} aria-hidden="true" />
        <input
          id="navigator-search"
          type="search"
          value={query}
          placeholder="Search planets and stars"
          autoComplete="off"
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && results[0]) {
              chooseResult(results[0]);
            }
          }}
          aria-controls="navigator-search-results"
        />
      </div>
      {normalizedQuery && (
        <ul
          id="navigator-search-results"
          className={style.searchResults}
          aria-label="Search results"
        >
          {results.length ? (
            results.map((result) => (
              <li key={`${result.kind}-${result.body.name}`}>
                <button type="button" onClick={() => chooseResult(result)}>
                  <span className={style.resultMarker} aria-hidden="true" />
                  <span>{result.body.name}</span>
                  <small>{result.kind}</small>
                </button>
              </li>
            ))
          ) : (
            <li className={style.noResults}>No celestial bodies found</li>
          )}
        </ul>
      )}
    </div>
  );
}
