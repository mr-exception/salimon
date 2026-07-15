import { tickingState } from './state';

export async function tick(invocationTime: Date) {
  if (tickingState.tickPromise) return tickingState.tickPromise;

  const startedAt = Date.now();
  tickingState.tickPromise = (async () => {
    const tickedObjects =
      tickingState.sandbox?.tick(invocationTime.getTime()) ?? [];
    const bodies = tickedObjects.filter((object) =>
      tickingState.sandbox?.getBodyKind(object),
    ).length;
    const spaceships = tickedObjects.filter((object) =>
      tickingState.sandbox?.getSpaceshipSecurityCode(object),
    ).length;
    return { bodies, spaceships };
  })().finally(() => {
    console.log(`tick passed: ${Date.now() - startedAt}ms`);
    tickingState.tickPromise = undefined;
  });

  return tickingState.tickPromise;
}
