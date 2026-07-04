import { useEffect, useState } from 'react';
import axios from 'axios';
import type { SpaceshipDto } from '@types';
import {
  getSpaceshipDto,
  hydrateSpaceship,
  spaceshipState,
  subscribeToWorld,
} from './world';

const STORAGE_KEY = 'salimon.spaceship';
const SECURITY_CODE_HEADER = 'x-spaceship-security-code';
const UPDATE_DELAY_MS = 1_000;
const DEFAULT_API_BASE_URL =
  'https://hjp81v6wyh.execute-api.us-east-1.amazonaws.com';

type BootstrapState = 'loading' | 'ready' | 'error';
type SpaceshipResponse = { spaceship: SpaceshipDto };

let bootstrapPromise: Promise<SpaceshipDto> | undefined;

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

async function registerSpaceship() {
  const { data } = await axios.post<SpaceshipResponse>(
    `${getApiBaseUrl()}/spaceship/register`,
  );
  return data.spaceship;
}

async function loadSpaceship() {
  const stored = readStoredSpaceship();
  if (!stored) return registerSpaceship();

  try {
    const { data } = await axios.get<SpaceshipResponse>(
      `${getApiBaseUrl()}/spaceship/info`,
      {
        headers: { [SECURITY_CODE_HEADER]: stored.securityCode },
      },
    );
    return data.spaceship;
  } catch (error) {
    if (axios.isAxiosError(error) && error.response?.status === 404) {
      return registerSpaceship();
    }
    throw error;
  }
}

function bootstrapSpaceship() {
  bootstrapPromise ??= loadSpaceship()
    .then((spaceship) => {
      storeSpaceship(spaceship);
      hydrateSpaceship(spaceship);
      return spaceship;
    })
    .catch((error: unknown) => {
      bootstrapPromise = undefined;
      throw error;
    });
  return bootstrapPromise;
}

async function updateSpaceship(securityCode: string) {
  const spaceship = getSpaceshipDto(securityCode);
  storeSpaceship(spaceship);
  const { data } = await axios.put<SpaceshipResponse>(
    `${getApiBaseUrl()}/spaceship/update`,
    spaceship,
    {
      headers: { [SECURITY_CODE_HEADER]: securityCode },
    },
  );
  storeSpaceship(data.spaceship);
}

export function useBootstrap(): BootstrapState {
  const [state, setState] = useState<BootstrapState>('loading');

  useEffect(() => {
    let disposed = false;
    let updateTimer: number | undefined;
    let securityCode: string | undefined;
    let unsubscribe: (() => void) | undefined;

    const flushUpdate = () => {
      if (!securityCode) return;
      window.clearTimeout(updateTimer);
      updateTimer = undefined;
      void updateSpaceship(securityCode).catch((error: unknown) => {
        console.error('Failed to persist spaceship', error);
      });
    };

    void bootstrapSpaceship()
      .then((spaceship) => {
        if (disposed) return;
        securityCode = spaceship.securityCode;
        unsubscribe = subscribeToWorld((_world, changedBodyNames) => {
          if (
            (changedBodyNames && !changedBodyNames.has(spaceshipState.name)) ||
            updateTimer
          ) {
            return;
          }
          updateTimer = window.setTimeout(flushUpdate, UPDATE_DELAY_MS);
        });
        window.addEventListener('pagehide', flushUpdate);
        setState('ready');
      })
      .catch((error: unknown) => {
        console.error('Failed to initialize spaceship', error);
        if (!disposed) setState('error');
      });

    return () => {
      disposed = true;
      unsubscribe?.();
      window.removeEventListener('pagehide', flushUpdate);
      window.clearTimeout(updateTimer);
    };
  }, []);

  return state;
}
