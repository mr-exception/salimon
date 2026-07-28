import type { SerializedWorldBody } from '@repo/types';
import { WorldSystemModel, type WorldBodyDocument } from '@models';
import { cloneWorldData } from './clone-world-data';
import { repositoryState } from './state';
import type { WorldData } from './types';

export async function getWorldData(): Promise<WorldData> {
  if (repositoryState.worldData) {
    return cloneWorldData(repositoryState.worldData);
  }

  repositoryState.worldDataPromise ??= loadWorldDataFromDatabase()
    .then((worldData) => {
      repositoryState.worldData = cloneWorldData(worldData);
      return worldData;
    })
    .finally(() => {
      repositoryState.worldDataPromise = undefined;
    });

  return cloneWorldData(await repositoryState.worldDataPromise);
}

async function loadWorldDataFromDatabase(): Promise<WorldData> {
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
