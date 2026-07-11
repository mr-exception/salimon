import { advanceBodies } from './advance-bodies';
import { advanceSpaceships } from './advance-spaceships';
import { loadWorldSnapshot } from './load-world-snapshot';
import { tickingState } from './state';

export async function tick(invocationTime: Date) {
  if (tickingState.tickPromise) return tickingState.tickPromise;

  const startedAt = Date.now();
  tickingState.tickPromise = (async () => {
    const world = await loadWorldSnapshot();
    const bodies = await advanceBodies(invocationTime);
    const spaceships = await advanceSpaceships(invocationTime, world);
    return { bodies, spaceships };
  })().finally(() => {
    console.log(`tick passed: ${Date.now() - startedAt}ms`);
    tickingState.tickPromise = undefined;
  });

  return tickingState.tickPromise;
}

