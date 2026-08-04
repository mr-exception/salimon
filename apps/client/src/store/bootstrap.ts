import { useEffect, useState } from 'react';
import axios from 'axios';
import { io, type Socket } from 'socket.io-client';
import type { SpaceshipDto } from '@repo/types';
import type { ContactMessageDto } from '@repo/types';
import {
  hydrateSpaceship,
  getSpaceshipDto,
  getSpaceshipProximityTelemetry,
  initializeSpaceshipInSimulation,
  setInventoryPersistHandler,
  startSpaceshipTargetSpeed,
  startSpaceshipThrusters,
  stopSpaceshipActiveFeatureLocally,
} from './world';
import { storeCachedContactMessage } from './contact-message-cache';

const STORAGE_KEY = 'salimon.spaceship';
const SHIP_SECRET_TOKEN_STORAGE_KEY = 'salimon.shipSecretToken';
export const SECURITY_CODE_HEADER = 'x-spaceship-security-code';
const DEFAULT_API_BASE_URL = 'http://localhost:3000';

export type BootstrapRequest =
  | { type: 'new' }
  | { type: 'continue' }
  | { type: 'claim'; securityCode: string };
export type BootstrapState = 'idle' | 'loading' | 'ready' | 'error';

type SpaceshipResponse = { spaceship: SpaceshipDto };
type ErrorResponse = { error?: string };
type ContactUnreadMessagesResponse = { messages: ContactMessage[] };
type ContactMessageSocketAck =
  | { ok: true; message: ContactMessage }
  | { ok: false; error: string };
type MarkThreadReadSocketAck =
  | { ok: true; contactId: string }
  | { ok: false; error: string };
export type ContactMessage = ContactMessageDto;
export type ContactShipContext = SpaceshipDto & {
  proximityTelemetry?: ReturnType<typeof getSpaceshipProximityTelemetry>;
};
const requestPromises = new WeakMap<BootstrapRequest, Promise<SpaceshipDto>>();
const contactMessageListeners = new Set<(message: ContactMessage) => void>();
const pendingContactMessages: ContactMessage[] = [];
let communicationSocket: Socket | undefined;
let communicationSocketSecurityCode: string | undefined;
let pendingSnapshotSync = false;
let inventorySyncTimer: number | undefined;
let currentSpaceship: SpaceshipDto | undefined;

export function getApiBaseUrl() {
  return (import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL).replace(
    /\/+$/,
    '',
  );
}

async function applySpaceshipInfo(spaceship: SpaceshipDto) {
  currentSpaceship = spaceship;
  storeSpaceship(spaceship);
  hydrateSpaceship(spaceship);
  connectCommunicationSocket(spaceship.securityCode);
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

function getSpaceshipSnapshotTime(spaceship: SpaceshipDto) {
  const snapshotTime = spaceship.positionCapturedAt ?? spaceship.simulatedAt;
  if (!snapshotTime) return 0;

  const time = Date.parse(snapshotTime);
  return Number.isFinite(time) ? time : 0;
}

function getNewestStoredSpaceship(spaceship: SpaceshipDto) {
  const storedSpaceship = readStoredSpaceship();
  if (
    !storedSpaceship ||
    storedSpaceship.securityCode !== spaceship.securityCode
  ) {
    return spaceship;
  }

  return getSpaceshipSnapshotTime(storedSpaceship) >
    getSpaceshipSnapshotTime(spaceship)
    ? storedSpaceship
    : spaceship;
}

function readStoredShipSecretToken() {
  const storedToken = localStorage.getItem(SHIP_SECRET_TOKEN_STORAGE_KEY);
  if (storedToken) return storedToken;

  return readStoredSpaceship()?.securityCode;
}

function storeShipSecretToken(securityCode: string) {
  localStorage.setItem(SHIP_SECRET_TOKEN_STORAGE_KEY, securityCode);
}

export function getStoredSpaceshipSecurityCode() {
  return readStoredShipSecretToken();
}

async function getSpaceshipFromRest(securityCode: string) {
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
    const workerRequest =
      request.type === 'continue'
        ? (() => {
            const securityCode = readStoredShipSecretToken();
            if (!securityCode) {
              throw new Error('No stored spaceship is available');
            }
            return {
              type: 'continue' as const,
              securityCode,
            };
          })()
        : request;
    const workerSpaceship = initializeSpaceshipInSimulation(workerRequest);
    if (workerSpaceship) return workerSpaceship;

    if (request.type === 'new') {
      const { data } = await axios.post<SpaceshipResponse>(
        `${getApiBaseUrl()}/spaceship/register`,
      );
      return data.spaceship;
    }
    if (request.type === 'claim') {
      return getSpaceshipFromRest(request.securityCode.trim());
    }
    const securityCode = readStoredShipSecretToken();
    if (!securityCode) throw new Error('No stored spaceship is available');
    return getSpaceshipFromRest(securityCode);
  })().then(async (spaceship) => {
    const resolvedSpaceship =
      request.type === 'continue'
        ? getNewestStoredSpaceship(spaceship)
        : spaceship;
    await applySpaceshipInfo(resolvedSpaceship);
    return resolvedSpaceship;
  });

  requestPromises.set(request, promise);
  return promise;
}

function flushInventorySync() {
  window.clearTimeout(inventorySyncTimer);
  inventorySyncTimer = undefined;
  if (!pendingSnapshotSync || !currentSpaceship) return;

  pendingSnapshotSync = false;
  void persistSpaceshipSnapshot().catch((error: unknown) => {
    console.error('Failed to persist spaceship snapshot', error);
    pendingSnapshotSync = true;
  });
}

function scheduleInventorySync() {
  pendingSnapshotSync = true;
  window.clearTimeout(inventorySyncTimer);
  inventorySyncTimer = window.setTimeout(flushInventorySync, 500);
}

async function persistSpaceshipSnapshot() {
  if (!currentSpaceship) return;
  const snapshot = getSpaceshipDto(currentSpaceship.securityCode);
  const { data } = await axios.put<SpaceshipResponse>(
    `${getApiBaseUrl()}/spaceship/info`,
    snapshot,
    {
      headers: {
        [SECURITY_CODE_HEADER]: currentSpaceship.securityCode,
      },
    },
  );
  currentSpaceship = data.spaceship;
  storeSpaceship(data.spaceship);
}

export async function saveSpaceship() {
  if (!currentSpaceship) throw new Error('Spaceship is not initialized');

  const snapshot = getSpaceshipDto(currentSpaceship.securityCode);
  try {
    const { data } = await axios.post<SpaceshipResponse>(
      `${getApiBaseUrl()}/spaceship/save`,
      snapshot,
      {
        headers: {
          [SECURITY_CODE_HEADER]: currentSpaceship.securityCode,
        },
      },
    );
    currentSpaceship = data.spaceship;
    storeShipSecretToken(data.spaceship.securityCode);
    storeSpaceship(data.spaceship);
    return data.spaceship;
  } catch (error) {
    if (axios.isAxiosError<ErrorResponse>(error)) {
      throw new Error(
        error.response?.data.error ?? 'Failed to save spaceship',
        { cause: error },
      );
    }
    throw error;
  }
}

export function subscribeToContactMessages(
  listener: (message: ContactMessage) => void,
) {
  contactMessageListeners.add(listener);
  pendingContactMessages.splice(0).forEach(listener);
  return () => {
    contactMessageListeners.delete(listener);
  };
}

function emitContactMessage(message: ContactMessage) {
  if (currentSpaceship) {
    void storeCachedContactMessage(
      currentSpaceship.securityCode,
      message,
    ).catch((error: unknown) => {
      console.error('Failed to cache contact message', error);
    });
  }

  if (contactMessageListeners.size === 0) {
    pendingContactMessages.push(message);
    if (pendingContactMessages.length > 100) pendingContactMessages.shift();
    return;
  }
  contactMessageListeners.forEach((listener) => listener(message));
}

function connectCommunicationSocket(securityCode: string) {
  if (communicationSocketSecurityCode === securityCode && communicationSocket) {
    if (!communicationSocket.connected) communicationSocket.connect();
    return communicationSocket;
  }

  communicationSocket?.disconnect();
  communicationSocketSecurityCode = securityCode;
  communicationSocket = io(getApiBaseUrl(), {
    auth: { securityCode },
    transports: ['websocket', 'polling'],
  });
  communicationSocket.on('contact:message', emitContactMessage);
  communicationSocket.on(
    'contact:unread-messages',
    ({ messages }: ContactUnreadMessagesResponse) => {
      messages.forEach(emitContactMessage);
    },
  );
  communicationSocket.on(
    'communications:error',
    ({ error }: { error: string }) => {
      console.error('Communications socket error', error);
    },
  );
  communicationSocket.on('connect_error', (error) => {
    console.error('Failed to connect communications socket', error);
  });
  return communicationSocket;
}

async function getCommunicationSocket() {
  const securityCode = currentSpaceship?.securityCode;
  if (!securityCode) throw new Error('Spaceship is not initialized');

  const socket = connectCommunicationSocket(securityCode);
  if (socket.connected) return socket;

  return new Promise<Socket>((resolve, reject) => {
    const cleanup = () => {
      socket.off('connect', handleConnect);
      socket.off('connect_error', handleError);
    };
    const handleConnect = () => {
      cleanup();
      resolve(socket);
    };
    const handleError = (error: Error) => {
      cleanup();
      reject(error);
    };
    socket.once('connect', handleConnect);
    socket.once('connect_error', handleError);
    socket.connect();
  });
}

export async function sendContactMessage(request: {
  contactId: string;
  text: string;
  clientMessageId: string;
  shipContext?: ContactShipContext;
}) {
  const socket = await getCommunicationSocket();
  return new Promise<ContactMessage>((resolve, reject) => {
    socket
      .timeout(10_000)
      .emit(
        'contact:send-message',
        request,
        (error: Error | null, response?: ContactMessageSocketAck) => {
          if (error) {
            reject(error);
            return;
          }
          if (!response?.ok) {
            reject(new Error(response?.error ?? 'Message failed to send'));
            return;
          }
          resolve(response.message);
        },
      );
  });
}

export async function markContactThreadRead(contactId: string) {
  const socket = await getCommunicationSocket();
  return new Promise<string>((resolve, reject) => {
    socket
      .timeout(10_000)
      .emit(
        'contact:mark-thread-read',
        { contactId },
        (error: Error | null, response?: MarkThreadReadSocketAck) => {
          if (error) {
            reject(error);
            return;
          }
          if (!response?.ok) {
            reject(new Error(response?.error ?? 'Failed to mark thread read'));
            return;
          }
          resolve(response.contactId);
        },
      );
  });
}

export function startSpaceshipTargetSpeedFeature(
  targetSpeedMetersPerSecond: number,
  maximumThrustPercent: number,
  targetDirection?: number,
) {
  const started = startSpaceshipTargetSpeed(
    targetSpeedMetersPerSecond,
    maximumThrustPercent,
    targetDirection,
  );
  if (started) pendingSnapshotSync = true;
  return started;
}

export function startSpaceshipThrustersFeature(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  const started = startSpaceshipThrusters(thrusters);
  if (started) pendingSnapshotSync = true;
  return started;
}

export function stopSpaceshipActiveFeature() {
  stopSpaceshipActiveFeatureLocally();
  pendingSnapshotSync = true;
}

export function useBootstrap(request: BootstrapRequest | null): BootstrapState {
  const [result, setResult] = useState<{
    request: BootstrapRequest | null;
    state: BootstrapState;
  }>({ request: null, state: 'idle' });

  useEffect(() => {
    if (!request) return;

    let disposed = false;

    void initializeSpaceship(request)
      .then(() => {
        if (disposed) return;
        setInventoryPersistHandler(scheduleInventorySync);
        setResult({ request, state: 'ready' });
      })
      .catch((error: unknown) => {
        console.error('Failed to initialize spaceship', error);
        if (!disposed) setResult({ request, state: 'error' });
      });

    return () => {
      disposed = true;
      flushInventorySync();
      setInventoryPersistHandler(undefined);
    };
  }, [request]);

  if (!request) return 'idle';
  return result.request === request ? result.state : 'loading';
}
