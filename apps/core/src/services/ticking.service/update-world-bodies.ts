import { advanceBodies } from './advance-bodies';
import { parseInvocationTime } from './parse-invocation-time';
import { start } from './start';

export async function updateWorldBodies(time: string | Date = new Date()) {
  await start();
  const updated = await advanceBodies(parseInvocationTime(time));
  return {
    selected: updated,
    updated,
  };
}

