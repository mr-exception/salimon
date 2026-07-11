import { advanceSpaceships } from './advance-spaceships';
import { loadWorldSnapshot } from './load-world-snapshot';
import { parseInvocationTime } from './parse-invocation-time';
import { start } from './start';

export async function updateSpaceships(time: string | Date = new Date()) {
  await start();
  const invocationTime = parseInvocationTime(time);
  const processed = await advanceSpaceships(
    invocationTime,
    await loadWorldSnapshot(),
  );
  return {
    selected: processed,
    processed,
  };
}

