import { useEffect, useState } from 'react';
import axios from 'axios';
import type { SerializedWorldSystems, SpaceshipDto } from '@repo/types';
import {
  hydrateWorldSystems,
  hydrateSpaceship,
  refreshWorldViewport,
  setWorldViewportLoader,
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
type WorldInfoMessage = Partial<SerializedWorldSystems> & {
  type: 'world:info';
  spaceship: SpaceshipDto;
  requestId?: string;
};
type SpaceshipErrorMessage = {
  type: 'error';
  error: string;
};
type WorldViewportMessage = SerializedWorldSystems & {
  type: 'world:viewport';
  requestId?: string;
};
type SpaceshipSocketMessage =
  | SpaceshipInfoMessage
  | WorldInfoMessage
  | SpaceshipErrorMessage
  | WorldViewportMessage;
const requestPromises = new WeakMap<BootstrapRequest, Promise<SpaceshipDto>>();
const worldViewportRequests = new Map<
  string,
  {
    resolve: (world: SerializedWorldSystems) => void;
    reject: (error: Error) => void;
  }
>();
let spaceshipSocket: WebSocket | undefined;
let worldRefreshPromise: Promise<unknown> | undefined;
let nextWorldRequestId = 0;

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
  if (message.type === 'world:info' && message.spaceship) {
    return {
      type: 'world:info',
      spaceship: message.spaceship,
      requestId:
        typeof message.requestId === 'string' ? message.requestId : undefined,
      ...(Array.isArray(message.systems) ? { systems: message.systems } : {}),
    };
  }
  if (message.type === 'world:viewport' && Array.isArray(message.systems)) {
    return {
      type: 'world:viewport',
      requestId:
        typeof message.requestId === 'string' ? message.requestId : undefined,
      systems: message.systems,
    };
  }
  if (message.type === 'error' && typeof message.error === 'string') {
    return { type: 'error', error: message.error };
  }
  throw new Error('Unsupported spaceship socket message');
}

function shouldRefreshWorldForSpaceship(spaceship: SpaceshipDto) {
  return spaceship.motionState === 'flying' && !spaceship.position.relativeTo;
}

async function refreshWorldForSpaceship() {
  worldRefreshPromise ??= refreshWorldViewport().finally(() => {
    worldRefreshPromise = undefined;
  });
  await worldRefreshPromise;
}

async function applySpaceshipInfo(spaceship: SpaceshipDto) {
  storeSpaceship(spaceship);
  if (shouldRefreshWorldForSpaceship(spaceship)) {
    await refreshWorldForSpaceship();
  }
  hydrateSpaceship(spaceship);
}

async function applyWorldInfo(message: WorldInfoMessage) {
  const handledViewportRequest = handleWorldViewportInfo(message);
  storeSpaceship(message.spaceship);
  if (message.systems) {
    if (!handledViewportRequest) {
      hydrateWorldSystems({ systems: message.systems });
    }
    hydrateSpaceship(message.spaceship);
    return;
  }

  await applySpaceshipInfo(message.spaceship);
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
        if (
          message.type !== 'spaceship:info' &&
          message.type !== 'world:info'
        ) {
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
  })().then(async (spaceship) => {
    await applySpaceshipInfo(spaceship);
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

function rejectPendingWorldViewportRequests(error: Error) {
  worldViewportRequests.forEach(({ reject }) => reject(error));
  worldViewportRequests.clear();
}

function requestWorldViewportOverSocket(request: {
  x: string;
  y: string;
  radius: string;
}) {
  if (!spaceshipSocket || spaceshipSocket.readyState !== WebSocket.OPEN) {
    return Promise.reject(new Error('Spaceship socket is not connected'));
  }

  const requestId = `world:${++nextWorldRequestId}`;
  return new Promise<SerializedWorldSystems>((resolve, reject) => {
    worldViewportRequests.set(requestId, { resolve, reject });
    spaceshipSocket?.send(
      JSON.stringify({
        type: 'world:viewport',
        requestId,
        ...request,
      }),
    );
  });
}

function handleWorldViewportInfo(
  message: WorldViewportMessage | WorldInfoMessage,
) {
  const requestId = message.requestId;
  if (!requestId || !message.systems) return false;

  const request = worldViewportRequests.get(requestId);
  if (!request) return false;

  worldViewportRequests.delete(requestId);
  request.resolve({ systems: message.systems });
  return true;
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
            if (message.type === 'world:viewport') {
              handleWorldViewportInfo(message);
              return;
            }
            if (message.type === 'world:info') {
              void applyWorldInfo(message).catch((error: unknown) => {
                console.error('Failed to apply world info', error);
              });
              return;
            }
            void applySpaceshipInfo(message.spaceship).catch(
              (error: unknown) => {
                console.error('Failed to apply spaceship info', error);
              },
            );
          } catch (error) {
            console.error('Failed to process spaceship socket message', error);
          }
        });
        socket.addEventListener('error', () => {
          console.error('Failed to connect to spaceship socket');
          rejectPendingWorldViewportRequests(
            new Error('Failed to connect to spaceship socket'),
          );
        });
        socket.addEventListener('open', () => {
          setWorldViewportLoader(requestWorldViewportOverSocket);
          setResult({ request, state: 'ready' });
        });
        socket.addEventListener('close', () => {
          if (spaceshipSocket === socket) {
            spaceshipSocket = undefined;
            setWorldViewportLoader(undefined);
          }
          rejectPendingWorldViewportRequests(
            new Error('Spaceship socket is closed'),
          );
        });
      })
      .catch((error: unknown) => {
        console.error('Failed to initialize spaceship', error);
        if (!disposed) setResult({ request, state: 'error' });
      });

    return () => {
      disposed = true;
      if (spaceshipSocket === socket) spaceshipSocket = undefined;
      setWorldViewportLoader(undefined);
      socket?.close();
    };
  }, [request]);

  if (!request) return 'idle';
  return result.request === request ? result.state : 'loading';
}
