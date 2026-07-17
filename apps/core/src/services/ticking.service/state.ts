import type { Timer } from './types';
import type { WorldSandbox } from '@repo/sandbox';

export type TickResult = {
  bodies: number;
  spaceships: number;
};

export const tickingState: {
  timer: Timer | undefined;
  startPromise: Promise<void> | undefined;
  tickPromise: Promise<TickResult> | undefined;
  sandbox: WorldSandbox | undefined;
  unsubscribeFromSandboxTicks: (() => void) | undefined;
  unsubscribeFromSandboxCrashes: (() => void) | undefined;
} = {
  timer: undefined,
  startPromise: undefined,
  tickPromise: undefined,
  sandbox: undefined,
  unsubscribeFromSandboxTicks: undefined,
  unsubscribeFromSandboxCrashes: undefined,
};
