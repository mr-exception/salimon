import { parseInvocationTime } from './parse-invocation-time';
import { start } from './start';
import { tick } from './tick';

export async function updateWorld(time: string | Date = new Date()) {
  await start();
  const invocationTime = parseInvocationTime(time);
  const result = await tick(invocationTime);
  return {
    selected: result.bodies + result.spaceships,
    updated: result.bodies + result.spaceships,
    bodies: result.bodies,
    spaceships: result.spaceships,
  };
}

