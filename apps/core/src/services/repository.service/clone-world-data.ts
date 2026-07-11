import type { WorldData } from './types';
import { cloneBody } from './clone-body';

export function cloneWorldData(worldData: WorldData): WorldData {
  return {
    planets: worldData.planets.map(cloneBody),
    moons: worldData.moons.map(cloneBody),
    stars: worldData.stars.map(cloneBody),
  };
}

