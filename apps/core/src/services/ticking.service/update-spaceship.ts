import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import { getAbsoluteSpaceshipUpdate } from './get-absolute-spaceship-update';
import { getSpaceshipUpdate } from './get-spaceship-update';
import { loadWorldSnapshot } from './load-world-snapshot';
import type { WorldSnapshot } from './types';

export async function updateSpaceship(
  spaceship: SpaceshipDocument,
  simulatedAt = new Date(),
  suppliedWorld?: WorldSnapshot,
) {
  const world = suppliedWorld ?? (await loadWorldSnapshot());
  const absoluteUpdate = getAbsoluteSpaceshipUpdate(
    spaceship,
    simulatedAt,
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
  const update = getSpaceshipUpdate(currentSpaceship, simulatedAt, world);
  if (!absoluteUpdate && !update) return spaceship;
  return RepositoryService.updatePropagatedSpaceship(spaceship, {
    ...absoluteUpdate,
    ...update,
    position: update?.position ?? absoluteUpdate?.position,
    velocity: update?.velocity ?? absoluteUpdate?.velocity,
    stats: update?.stats ?? absoluteUpdate?.stats,
  });
}

