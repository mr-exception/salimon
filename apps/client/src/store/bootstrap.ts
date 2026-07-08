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
export const SECURITY_CODE_HEADER = 'x-spaceship-security-code';
const COASTING_UPDATE_DELAY_MS = 5 * 60 * 1_000;
const THRUSTING_UPDATE_DELAY_MS = 5_000;
const DEFAULT_API_BASE_URL = 'http://localhost:3000';

export type BootstrapRequest =
  | { type: 'new' }
  | { type: 'continue' }
  | { type: 'claim'; securityCode: string };
export type BootstrapState = 'idle' | 'loading' | 'ready' | 'error';

type SpaceshipResponse = { spaceship: SpaceshipDto };
type SpaceshipInfoMessage = {
  type: 'spaceship:info';
  spaceship: SpaceshipDto;
};
type SpaceshipErrorMessage = {
  type: 'error';
  error: string;
};
type SpaceshipSocketMessage = SpaceshipInfoMessage | SpaceshipErrorMessage;
const requestPromises = new WeakMap<BootstrapRequest, Promise<SpaceshipDto>>();

export function getApiBaseUrl() {
  return (import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL).replace(
    /\/+$/,
    '',
  );
}

function getSpaceshipSocketUrl(securityCode: string) {
  const url = new URL(`${getApiBaseUrl()}/spaceship/socket`);
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  url.searchParams.set('shipSecret', securityCode);
  return url.toString();
}

function parseSpaceshipSocketMessage(data: string): SpaceshipSocketMessage {
  const message = JSON.parse(data) as Partial<SpaceshipSocketMessage>;
  if (message.type === 'spaceship:info' && message.spaceship) {
    return {
      type: 'spaceship:info',
      spaceship: message.spaceship,
    };
  }
  if (message.type === 'error' && typeof message.error === 'string') {
    return { type: 'error', error: message.error };
  }
  throw new Error('Unsupported spaceship socket message');
}

function applySpaceshipInfo(spaceship: SpaceshipDto) {
  storeSpaceship(spaceship);
  hydrateSpaceship(spaceship);
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

async function getSpaceshipFromSocket(securityCode: string) {
  return new Promise<SpaceshipDto>((resolve, reject) => {
    const socket = new WebSocket(getSpaceshipSocketUrl(securityCode));

    socket.addEventListener('message', (event) => {
      try {
        const message = parseSpaceshipSocketMessage(String(event.data));
        if (message.type === 'error') {
          reject(new Error(message.error));
          socket.close();
          return;
        }
        resolve(message.spaceship);
        socket.close();
      } catch (error) {
        reject(error);
        socket.close();
      }
    });

    socket.addEventListener('error', () => {
      reject(new Error('Failed to connect to spaceship socket'));
    });
  });
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
      return getSpaceshipFromSocket(request.securityCode.trim());
    }
    const stored = readStoredSpaceship();
    if (!stored) throw new Error('No stored spaceship is available');
    return getSpaceshipFromSocket(stored.securityCode);
  })().then((spaceship) => {
    applySpaceshipInfo(spaceship);
    return spaceship;
  });

  requestPromises.set(request, promise);
  return promise;
}

function updateSpaceship(socket: WebSocket, securityCode: string) {
  const spaceship = getSpaceshipDto(securityCode);
  storeSpaceship(spaceship);
  if (socket.readyState !== WebSocket.OPEN) {
    throw new Error('Spaceship socket is not connected');
  }
  socket.send(JSON.stringify({ type: 'spaceship:update', spaceship }));
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
    let socket: WebSocket | undefined;
    let engineWasRunning = false;
    let unsubscribe: (() => void) | undefined;

    const flushUpdate = () => {
      if (!securityCode) return;
      window.clearTimeout(updateTimer);
      updateTimer = undefined;
      updateDelay = undefined;
      if (!socket) return;
      try {
        updateSpaceship(socket, securityCode);
      } catch (error: unknown) {
        console.error('Failed to persist spaceship', error);
      }
    };

    void initializeSpaceship(request)
      .then((spaceship) => {
        if (disposed) return;
        securityCode = spaceship.securityCode;
        socket = new WebSocket(getSpaceshipSocketUrl(securityCode));
        socket.addEventListener('message', (event) => {
          try {
            const message = parseSpaceshipSocketMessage(String(event.data));
            if (message.type === 'error') {
              console.error('Spaceship socket error', message.error);
              return;
            }
            applySpaceshipInfo(message.spaceship);
          } catch (error) {
            console.error('Failed to process spaceship socket message', error);
          }
        });
        socket.addEventListener('error', () => {
          console.error('Failed to connect to spaceship socket');
        });
        unsubscribe = subscribeToWorld((_world, changedBodyNames) => {
          if (changedBodyNames && !changedBodyNames.has(spaceshipState.name)) {
            return;
          }
          const engineIsRunning = isSpaceshipEngineRunning();
          if (engineWasRunning && !engineIsRunning) {
            engineWasRunning = false;
            flushUpdate();
            return;
          }

          engineWasRunning = engineIsRunning;
          const nextUpdateDelay = engineIsRunning
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
      socket?.close();
    };
  }, [request]);

  if (!request) return 'idle';
  return result.request === request ? result.state : 'loading';
}
