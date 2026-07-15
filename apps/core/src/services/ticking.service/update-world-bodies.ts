import { parseInvocationTime } from './parse-invocation-time';
import { start } from './start';
import { tickingState } from './state';

export async function updateWorldBodies(time: string | Date = new Date()) {
  await start();
  const updated =
    tickingState.sandbox
      ?.tick(parseInvocationTime(time).getTime())
      .filter((object) => tickingState.sandbox?.getBodyKind(object)).length ??
    0;
  return {
    selected: updated,
    updated,
  };
}
