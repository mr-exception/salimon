import { useEffect, useState } from 'react';
import axios from 'axios';
import type {
  AsteroidDto,
  SerializedWorldSystems,
  SpaceshipDto,
  SpaceshipInventory,
} from '@repo/types';
import type { ContactMessageDto } from '@repo/types';
import {
  hydrateAsteroids,
  hydrateWorldSystems,
  hydrateSpaceship,
  setInventoryPersistHandler,
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
  asteroids?: AsteroidDto[];
};
type SpaceshipErrorMessage = {
  type: 'error';
  error: string;
};
type WorldViewportMessage = SerializedWorldSystems & {
  type: 'world:viewport';
  requestId?: string;
  asteroids?: AsteroidDto[];
};
export type ContactMessage = ContactMessageDto;
type ContactSocketMessage = {
  type: 'contact:message';
  requestId?: string;
  message: ContactMessage;
};
type ContactSocketErrorMessage = {
  type: 'contact:message:error';
  requestId?: string;
  error: string;
};
type SpaceshipSocketMessage =
  | SpaceshipInfoMessage
  | WorldInfoMessage
  | SpaceshipErrorMessage
  | WorldViewportMessage
  | ContactSocketMessage
  | ContactSocketErrorMessage;
const requestPromises = new WeakMap<BootstrapRequest, Promise<SpaceshipDto>>();
const worldViewportRequests = new Map<
  string,
  {
    resolve: (world: SerializedWorldSystems) => void;
    reject: (error: Error) => void;
  }
>();
const contactMessageRequests = new Map<
  string,
  {
    resolve: (message: ContactMessage) => void;
    reject: (error: Error) => void;
  }
>();
const contactMessageListeners = new Set<(message: ContactMessage) => void>();
let spaceshipSocket: WebSocket | undefined;
let nextWorldRequestId = 0;
let nextContactMessageRequestId = 0;
let pendingInventorySync: SpaceshipInventory | undefined;
let unconfirmedInventorySync: SpaceshipInventory | undefined;
let inventorySyncTimer: number | undefined;

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
      ...(Array.isArray(message.asteroids)
        ? { asteroids: message.asteroids }
        : {}),
    };
  }
  if (message.type === 'world:viewport' && Array.isArray(message.systems)) {
    return {
      type: 'world:viewport',
      requestId:
        typeof message.requestId === 'string' ? message.requestId : undefined,
      systems: message.systems,
      ...(Array.isArray(message.asteroids)
        ? { asteroids: message.asteroids }
        : {}),
    };
  }
  if (message.type === 'error' && typeof message.error === 'string') {
    return { type: 'error', error: message.error };
  }
  if (
    message.type === 'contact:message' &&
    message.message &&
    typeof message.message === 'object'
  ) {
    return {
      type: 'contact:message',
      requestId:
        typeof message.requestId === 'string' ? message.requestId : undefined,
      message: message.message as ContactMessage,
    };
  }
  if (
    message.type === 'contact:message:error' &&
    typeof message.error === 'string'
  ) {
    return {
      type: 'contact:message:error',
      requestId:
        typeof message.requestId === 'string' ? message.requestId : undefined,
      error: message.error,
    };
  }
  throw new Error('Unsupported spaceship socket message');
}

function inventoriesEqual(
  left?: Partial<SpaceshipInventory>,
  right?: Partial<SpaceshipInventory>,
) {
  if (!left || !right) return false;
  return Object.entries(right).every(
    ([material, amount]) =>
      left[material as keyof SpaceshipInventory] === amount,
  );
}

function preservePendingInventory(spaceship: SpaceshipDto) {
  const inventory = pendingInventorySync ?? unconfirmedInventorySync;
  if (!inventory) return spaceship;

  if (inventoriesEqual(spaceship.inventory, inventory)) {
    unconfirmedInventorySync = undefined;
    return spaceship;
  }

  return { ...spaceship, inventory };
}

async function applySpaceshipInfo(spaceship: SpaceshipDto) {
  const clientSpaceship = preservePendingInventory(spaceship);
  storeSpaceship(clientSpaceship);
  hydrateSpaceship(clientSpaceship);
}

async function applyWorldInfo(message: WorldInfoMessage) {
  const handledViewportRequest = handleWorldViewportInfo(message);
  const spaceship = preservePendingInventory(message.spaceship);
  storeSpaceship(spaceship);
  if (message.asteroids) {
    hydrateAsteroids(message.asteroids);
  }
  if (message.systems) {
    if (!handledViewportRequest) {
      hydrateWorldSystems({ systems: message.systems });
    }
    hydrateSpaceship(spaceship);
    return;
  }

  await applySpaceshipInfo(spaceship);
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

function flushInventorySync() {
  window.clearTimeout(inventorySyncTimer);
  inventorySyncTimer = undefined;
  if (!pendingInventorySync) return;
  if (!spaceshipSocket || spaceshipSocket.readyState !== WebSocket.OPEN) {
    return;
  }

  const inventory = pendingInventorySync;
  pendingInventorySync = undefined;
  unconfirmedInventorySync = inventory;
  spaceshipSocket.send(
    JSON.stringify({
      type: 'spaceship:inventory',
      inventory,
    }),
  );
}

function scheduleInventorySync(inventory: SpaceshipInventory) {
  pendingInventorySync = inventory;
  window.clearTimeout(inventorySyncTimer);
  inventorySyncTimer = window.setTimeout(flushInventorySync, 500);
}

function rejectPendingWorldViewportRequests(error: Error) {
  worldViewportRequests.forEach(({ reject }) => reject(error));
  worldViewportRequests.clear();
}

function rejectPendingContactMessageRequests(error: Error) {
  contactMessageRequests.forEach(({ reject }) => reject(error));
  contactMessageRequests.clear();
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
  if (message.asteroids) {
    hydrateAsteroids(message.asteroids);
  }

  const requestId = message.requestId;
  if (!requestId || !message.systems) return false;

  const request = worldViewportRequests.get(requestId);
  if (!request) return false;

  worldViewportRequests.delete(requestId);
  request.resolve({ systems: message.systems });
  return true;
}

function handleContactMessageInfo(message: ContactSocketMessage) {
  const requestId = message.requestId;
  if (requestId) {
    const request = contactMessageRequests.get(requestId);
    if (request) {
      contactMessageRequests.delete(requestId);
      request.resolve(message.message);
    }
  }

  contactMessageListeners.forEach((listener) => listener(message.message));
}

function handleContactMessageError(message: ContactSocketErrorMessage) {
  const requestId = message.requestId;
  if (!requestId) {
    console.error('Contact message socket error', message.error);
    return;
  }

  const request = contactMessageRequests.get(requestId);
  if (!request) return;
  contactMessageRequests.delete(requestId);
  request.reject(new Error(message.error));
}

export function subscribeToContactMessages(
  listener: (message: ContactMessage) => void,
) {
  contactMessageListeners.add(listener);
  return () => {
    contactMessageListeners.delete(listener);
  };
}

export function sendContactMessageOverSocket(request: {
  contactId: string;
  text: string;
  clientMessageId: string;
}) {
  if (!spaceshipSocket || spaceshipSocket.readyState !== WebSocket.OPEN) {
    return Promise.reject(new Error('Spaceship socket is not connected'));
  }

  const requestId = `contact:${++nextContactMessageRequestId}`;
  return new Promise<ContactMessage>((resolve, reject) => {
    contactMessageRequests.set(requestId, { resolve, reject });
    spaceshipSocket?.send(
      JSON.stringify({
        type: 'contact:message:send',
        requestId,
        ...request,
      }),
    );
  });
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

export function startSpaceshipManualForceFeature(
  thrusters: { powerPercent: number; durationSeconds: number }[],
) {
  sendSpaceshipSocketMessage({
    type: 'spaceship:start-manual-force',
    thrusters,
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
            if (message.type === 'contact:message') {
              handleContactMessageInfo(message);
              return;
            }
            if (message.type === 'contact:message:error') {
              handleContactMessageError(message);
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
          rejectPendingContactMessageRequests(
            new Error('Failed to connect to spaceship socket'),
          );
        });
        socket.addEventListener('open', () => {
          setWorldViewportLoader(requestWorldViewportOverSocket);
          setInventoryPersistHandler(scheduleInventorySync);
          setResult({ request, state: 'ready' });
        });
        socket.addEventListener('close', () => {
          flushInventorySync();
          if (spaceshipSocket === socket) {
            spaceshipSocket = undefined;
            setWorldViewportLoader(undefined);
            setInventoryPersistHandler(undefined);
          }
          rejectPendingWorldViewportRequests(
            new Error('Spaceship socket is closed'),
          );
          rejectPendingContactMessageRequests(
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
      flushInventorySync();
      if (spaceshipSocket === socket) spaceshipSocket = undefined;
      setWorldViewportLoader(undefined);
      setInventoryPersistHandler(undefined);
      socket?.close();
    };
  }, [request]);

  if (!request) return 'idle';
  return result.request === request ? result.state : 'loading';
}
