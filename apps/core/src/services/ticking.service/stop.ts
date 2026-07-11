import { tickingState } from './state';

export function stop() {
  if (tickingState.timer) {
    clearInterval(tickingState.timer);
    tickingState.timer = undefined;
  }

  tickingState.startPromise = undefined;
  tickingState.tickPromise = undefined;
}

