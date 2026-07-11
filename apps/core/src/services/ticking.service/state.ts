import type { Timer } from './types';

export type TickResult = {
  bodies: number;
  spaceships: number;
};

export const tickingState: {
  timer: Timer | undefined;
  startPromise: Promise<void> | undefined;
  tickPromise: Promise<TickResult> | undefined;
} = {
  timer: undefined,
  startPromise: undefined,
  tickPromise: undefined,
};

