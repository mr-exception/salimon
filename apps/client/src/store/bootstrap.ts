import { useEffect, useState } from 'react';
import axios from 'axios';
import type { SpaceshipDto } from '@repo/types';
import type { ContactMessageDto } from '@repo/types';
import {
  hydrateSpaceship,
  getSpaceshipDto,
  setInventoryPersistHandler,
  startSpaceshipTargetSpeed,
  startSpaceshipThrusters,
  stopSpaceshipActiveFeatureLocally,
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
type ContactMessageResponse = { message: ContactMessage };
export type ContactMessage = ContactMessageDto;
const requestPromises = new WeakMap<BootstrapRequest, Promise<SpaceshipDto>>();
const contactMessageListeners = new Set<(message: ContactMessage) => void>();
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
    if (request.type === 'new') {
      const { data } = await axios.post<SpaceshipResponse>(
        `${getApiBaseUrl()}/spaceship/register`,
      );
      return data.spaceship;
    }
    if (request.type === 'claim') {
      return getSpaceshipFromRest(request.securityCode.trim());
    }
    const stored = readStoredSpaceship();
    if (!stored) throw new Error('No stored spaceship is available');
    return getSpaceshipFromRest(stored.securityCode);
  })().then(async (spaceship) => {
    await applySpaceshipInfo(spaceship);
    return spaceship;
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
  storeSpaceship(snapshot);
}

export function subscribeToContactMessages(
  listener: (message: ContactMessage) => void,
) {
  contactMessageListeners.add(listener);
  return () => {
    contactMessageListeners.delete(listener);
  };
}

export async function sendContactMessage(request: {
  contactId: string;
  text: string;
  clientMessageId: string;
}) {
  const securityCode = currentSpaceship?.securityCode;
  if (!securityCode) throw new Error('Spaceship is not initialized');

  const { data } = await axios.post<ContactMessageResponse>(
    `${getApiBaseUrl()}/contacts/messages`,
    request,
    { headers: { [SECURITY_CODE_HEADER]: securityCode } },
  );
  contactMessageListeners.forEach((listener) => listener(data.message));
  return data.message;
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
    let snapshotTimer: number | undefined;

    void initializeSpaceship(request)
      .then(() => {
        if (disposed) return;
        setInventoryPersistHandler(scheduleInventorySync);
        snapshotTimer = window.setInterval(() => {
          pendingSnapshotSync = true;
          flushInventorySync();
        }, 5_000);
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
      window.clearInterval(snapshotTimer);
    };
  }, [request]);

  if (!request) return 'idle';
  return result.request === request ? result.state : 'loading';
}
