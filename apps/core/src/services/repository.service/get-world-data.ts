import type { SerializedWorldBody } from '@repo/types';
import { WorldSystemModel, type WorldBodyDocument } from '@models';
import type { WorldData } from './types';

export async function getWorldData(): Promise<WorldData> {
  const systems = await WorldSystemModel.findAllSystems();
  const worldData: WorldData = {
    planets: [],
    moons: [],
    stars: [],
  };

  systems
    .flatMap((system) => system.bodies)
    .forEach((body) => {
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
    minZoomRenderShape: body.minZoomRenderShape,
    minZoomRenderName: body.minZoomRenderName,
    rotationPeriodSeconds: body.rotationPeriodSeconds,
    updatedAt: new Date(body.cTime ?? Date.now()),
  };
}
