import { requireWorldData } from './state';
import { start } from './start';
import type { WorldData } from './types';

export async function updateWorldBodies(
  updater: (worldData: WorldData) => number,
): Promise<number> {
  await start();
  return updater(requireWorldData());
}

