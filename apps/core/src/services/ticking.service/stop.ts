import { tickingState } from './state';

export function stop() {
  tickingState.unsubscribeFromSandboxTicks?.();
  tickingState.unsubscribeFromSandboxTicks = undefined;
  tickingState.sandbox?.stop();
  tickingState.sandbox = undefined;

  if (tickingState.timer) {
    clearInterval(tickingState.timer);
    tickingState.timer = undefined;
  }

  tickingState.startPromise = undefined;
  tickingState.tickPromise = undefined;
}
