import { RepositoryService } from '../repository.service';
import { cloneSpaceship } from './clone-spaceship';
import { getAbsoluteSpaceshipUpdate } from './get-absolute-spaceship-update';
import { getSpaceshipUpdate } from './get-spaceship-update';
import type { WorldSnapshot } from './types';

export async function advanceSpaceships(
  invocationTime: Date,
  world: WorldSnapshot,
) {
  let processed = 0;

  await RepositoryService.updateSpaceships((spaceshipsBySecurityCode) => {
    const spaceships = [...spaceshipsBySecurityCode.values()]
      .sort((left, right) => left.securityCode.localeCompare(right.securityCode))
      .map(cloneSpaceship);

    for (const spaceship of spaceships) {
      const absoluteUpdate = getAbsoluteSpaceshipUpdate(
        spaceship,
        invocationTime,
        world,
      );
      const currentSpaceship = absoluteUpdate
        ? {
            ...spaceship,
            ...absoluteUpdate,
            position: absoluteUpdate.position ?? spaceship.position,
            velocity: absoluteUpdate.velocity ?? spaceship.velocity,
          }
        : spaceship;
      const update = getSpaceshipUpdate(currentSpaceship, invocationTime, world);
      if (!absoluteUpdate && !update) continue;

      const current = spaceshipsBySecurityCode.get(spaceship.securityCode);
      if (
        !current ||
        current.updatedAt.getTime() !== spaceship.updatedAt.getTime()
      ) {
        continue;
      }

      spaceshipsBySecurityCode.set(
        spaceship.securityCode,
        cloneSpaceship({
          ...current,
          ...absoluteUpdate,
          ...update,
          position:
            update?.position ?? absoluteUpdate?.position ?? current.position,
          velocity:
            update?.velocity ?? absoluteUpdate?.velocity ?? current.velocity,
          stats: update?.stats ?? current.stats,
        }),
      );
      processed += 1;
    }

    return processed;
  });

  return processed;
}

