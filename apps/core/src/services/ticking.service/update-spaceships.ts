import { parseInvocationTime } from './parse-invocation-time';
import { start } from './start';
import { tickingState } from './state';

export async function updateSpaceships(time: string | Date = new Date()) {
  await start();
  const processed =
    tickingState.sandbox
      ?.tick(parseInvocationTime(time).getTime())
      .filter((object) =>
        tickingState.sandbox?.getSpaceshipSecurityCode(object),
      ).length ?? 0;
  return {
    selected: processed,
    processed,
  };
}
