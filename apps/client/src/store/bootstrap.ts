import { useEffect, useState } from 'react';
import axios from 'axios';
import type { SpaceshipDto } from '@types';
import {
  getSpaceshipDto,
  hydrateSpaceship,
  isSpaceshipEngineRunning,
  spaceshipState,
  subscribeToWorld,
} from './world';

const STORAGE_KEY = 'salimon.spaceship';
const SECURITY_CODE_HEADER = 'x-spaceship-security-code';
const COASTING_UPDATE_DELAY_MS = 5 * 60 * 1_000;
const THRUSTING_UPDATE_DELAY_MS = 5_000;
const DEFAULT_API_BASE_URL =
  'https://hjp81v6wyh.execute-api.us-east-1.amazonaws.com';

export type BootstrapRequest =
  | { type: 'new' }
  | { type: 'continue' }
  | { type: 'claim'; securityCode: string };
export type BootstrapState = 'idle' | 'loading' | 'ready' | 'error';

type SpaceshipResponse = { spaceship: SpaceshipDto };
const requestPromises = new WeakMap<BootstrapRequest, Promise<SpaceshipDto>>();

function getApiBaseUrl() {
  return (import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL).replace(
    /\/+$/,
    '',
  );
}

function readStoredSpaceship() {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return undefined;
  try {
    const spaceship = JSON.parse(stored) as SpaceshipDto;
    return typeof spaceship.securityCode === 'string' ? spaceship : undefined;
  } catch {
    localStorage.removeItem(STORAGE_KEY);
    return undefined;
  }
}

function storeSpaceship(spaceship: SpaceshipDto) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(spaceship));
}

export function getStoredSpaceshipSecurityCode() {
  return readStoredSpaceship()?.securityCode;
}

async function getSpaceship(securityCode: string) {
  const { data } = await axios.get<SpaceshipResponse>(
    `${getApiBaseUrl()}/spaceship/info`,
    { headers: { [SECURITY_CODE_HEADER]: securityCode } },
  );
  return data.spaceship;
}

function initializeSpaceship(request: BootstrapRequest) {
  const existingPromise = requestPromises.get(request);
  if (existingPromise) return existingPromise;

  const promise = (async () => {
    if (request.type === 'new') {
      const { data } = await axios.post<SpaceshipResponse>(
        `${getApiBaseUrl()}/spaceship/register`,
      );
      return data.spaceship;
    }
    if (request.type === 'claim') {
      return getSpaceship(request.securityCode.trim());
    }
    const stored = readStoredSpaceship();
    if (!stored) throw new Error('No stored spaceship is available');
    return getSpaceship(stored.securityCode);
  })().then((spaceship) => {
    storeSpaceship(spaceship);
    hydrateSpaceship(spaceship);
    return spaceship;
  });

  requestPromises.set(request, promise);
  return promise;
}

async function updateSpaceship(securityCode: string) {
  const spaceship = getSpaceshipDto(securityCode);
  storeSpaceship(spaceship);
  const { data } = await axios.put<SpaceshipResponse>(
    `${getApiBaseUrl()}/spaceship/update`,
    spaceship,
    { headers: { [SECURITY_CODE_HEADER]: securityCode } },
  );
  storeSpaceship(data.spaceship);
}

export function useBootstrap(request: BootstrapRequest | null): BootstrapState {
  const [result, setResult] = useState<{
    request: BootstrapRequest | null;
    state: BootstrapState;
  }>({ request: null, state: 'idle' });

  useEffect(() => {
    if (!request) return;

    let disposed = false;
    let updateTimer: number | undefined;
    let updateDelay: number | undefined;
    let securityCode: string | undefined;
    let unsubscribe: (() => void) | undefined;

    const flushUpdate = () => {
      if (!securityCode) return;
      window.clearTimeout(updateTimer);
      updateTimer = undefined;
      updateDelay = undefined;
      void updateSpaceship(securityCode).catch((error: unknown) => {
        console.error('Failed to persist spaceship', error);
      });
    };

    void initializeSpaceship(request)
      .then((spaceship) => {
        if (disposed) return;
        securityCode = spaceship.securityCode;
        unsubscribe = subscribeToWorld((_world, changedBodyNames) => {
          if (changedBodyNames && !changedBodyNames.has(spaceshipState.name)) {
            return;
          }
          const nextUpdateDelay = isSpaceshipEngineRunning()
            ? THRUSTING_UPDATE_DELAY_MS
            : COASTING_UPDATE_DELAY_MS;
          if (updateTimer && updateDelay === nextUpdateDelay) return;
          window.clearTimeout(updateTimer);
          updateDelay = nextUpdateDelay;
          updateTimer = window.setTimeout(flushUpdate, nextUpdateDelay);
        });
        window.addEventListener('pagehide', flushUpdate);
        setResult({ request, state: 'ready' });
      })
      .catch((error: unknown) => {
        console.error('Failed to initialize spaceship', error);
        if (!disposed) setResult({ request, state: 'error' });
      });

    return () => {
      disposed = true;
      unsubscribe?.();
      window.removeEventListener('pagehide', flushUpdate);
      window.clearTimeout(updateTimer);
    };
  }, [request]);

  if (!request) return 'idle';
  return result.request === request ? result.state : 'loading';
}
