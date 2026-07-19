import type { WorldData } from './types';

export async function updateWorldBodies(
  updater: (worldData: WorldData) => number,
): Promise<number> {
  void updater;
  throw new Error('World bodies are stored in the static world asset.');
}
