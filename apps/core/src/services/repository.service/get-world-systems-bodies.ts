import { getWorldData } from './get-world-data';
import { toPublicBody } from './to-public-body';

export async function getWorldSystemsBodies() {
  const worldData = await getWorldData();
  return {
    planets: worldData.planets.map(toPublicBody),
    moons: worldData.moons.map(toPublicBody),
    stars: worldData.stars.map(toPublicBody),
  };
}

