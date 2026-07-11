import { tickingState } from './state';
import { startTicking } from './start-ticking';

export async function start() {
  tickingState.startPromise ??= startTicking();
  return tickingState.startPromise;
}

