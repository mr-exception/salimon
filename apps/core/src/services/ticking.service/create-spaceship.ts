import { SpaceshipService } from '../spaceship.service';
import { getAbsoluteSpaceshipUpdate } from './get-absolute-spaceship-update';
import { loadWorldSnapshot } from './load-world-snapshot';

export async function createSpaceship() {
  const spaceship = SpaceshipService.createSpaceship();
  const simulatedAt = spaceship.simulatedAt ?? new Date();
  const world = await loadWorldSnapshot();
  const absoluteUpdate = getAbsoluteSpaceshipUpdate(
    spaceship,
    simulatedAt,
    world,
  );
  return {
    ...spaceship,
    ...absoluteUpdate,
    position: absoluteUpdate?.position ?? spaceship.position,
    velocity: absoluteUpdate?.velocity ?? spaceship.velocity,
  };
}

