import type { SerializedWorldBody } from '@repo/types';
import type { WorldBodyDocument } from '@models';
import { WorldAssetService } from '../world-asset.service';
import type { WorldData } from './types';

export async function getWorldData(): Promise<WorldData> {
  const { systems } = await WorldAssetService.getWorldSystems();
  const worldData: WorldData = {
    planets: [],
    moons: [],
    stars: [],
  };

  systems.flat().forEach((body) => {
    if (body.type === 'star') {
      worldData.stars.push(toWorldBodyDocument(body));
      return;
    }

    if (body.type === 'planet' || body.type === 'blackhole') {
      worldData.planets.push(toWorldBodyDocument(body));
      return;
    }

    if (body.type === 'moon') {
      worldData.moons.push(toWorldBodyDocument(body));
    }
  });

  return worldData;
}

function toWorldBodyDocument(body: SerializedWorldBody): WorldBodyDocument {
  return {
    id: body.id,
    name: body.name,
    isReal: body.isReal ?? true,
    position: { ...body.position },
    orbitalCenter: body.orbitalCenter,
    clockwise: body.clockwise,
    speed: body.speed,
    mass: body.mass,
    radius: body.radius,
    rotationPeriodSeconds: body.rotationPeriodSeconds,
    updatedAt: new Date(body.cTime ?? Date.now()),
  };
}
