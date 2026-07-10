import { useEffect, useState } from 'react';
import axios from 'axios';
import type { SpaceshipDto } from '@repo/types';
import {
  hydrateSpaceship,
} from './world';

const STORAGE_KEY = 'salimon.spaceship';
export const SECURITY_CODE_HEADER = 'x-spaceship-security-code';
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
let spaceshipSocket: WebSocket | undefined;

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

function sendSpaceshipSocketMessage(message: unknown) {
  if (!spaceshipSocket || spaceshipSocket.readyState !== WebSocket.OPEN) {
    throw new Error('Spaceship socket is not connected');
  }

  spaceshipSocket.send(JSON.stringify(message));
}

export function startSpaceshipTargetSpeedFeature(
  targetSpeedMetersPerSecond: number,
  maximumThrustPercent: number,
  targetDirection?: number,
) {
  sendSpaceshipSocketMessage({
    type: 'spaceship:start-target-speed',
    targetSpeedMetersPerSecond,
    maximumThrustPercent,
    targetDirection,
  });
}

export function stopSpaceshipActiveFeature() {
  sendSpaceshipSocketMessage({ type: 'spaceship:stop-active-feature' });
}

export function useBootstrap(request: BootstrapRequest | null): BootstrapState {
  const [result, setResult] = useState<{
    request: BootstrapRequest | null;
    state: BootstrapState;
  }>({ request: null, state: 'idle' });

  useEffect(() => {
    if (!request) return;

    let disposed = false;
    let socket: WebSocket | undefined;

    void initializeSpaceship(request)
      .then((spaceship) => {
        if (disposed) return;
        socket = new WebSocket(getSpaceshipSocketUrl(spaceship.securityCode));
        spaceshipSocket = socket;
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
        setResult({ request, state: 'ready' });
      })
      .catch((error: unknown) => {
        console.error('Failed to initialize spaceship', error);
        if (!disposed) setResult({ request, state: 'error' });
      });

    return () => {
      disposed = true;
      if (spaceshipSocket === socket) spaceshipSocket = undefined;
      socket?.close();
    };
  }, [request]);

  if (!request) return 'idle';
  return result.request === request ? result.state : 'loading';
}
