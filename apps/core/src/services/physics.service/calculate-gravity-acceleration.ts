import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';
import type { WorldSnapshot } from '../ticking.service/types';
import { getBodyPositions } from './get-body-positions';

export function calculateGravityAcceleration(
  position: SpaceshipVelocity,
  world: WorldSnapshot,
  time: Date,
): SpaceshipVelocity {
  const bodyPositions = getBodyPositions(world, time);
  return WorldService.calculateGravityAcceleration(
    position,
    world.bodies,
    (body) => bodyPositions.get(body.name),
  );
}

