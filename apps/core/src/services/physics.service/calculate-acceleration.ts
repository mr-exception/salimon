import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';
import type { WorldSnapshot } from '../ticking.service/types';
import { calculateGravityAcceleration } from './calculate-gravity-acceleration';

export function calculateAcceleration(
  position: SpaceshipVelocity,
  world: WorldSnapshot,
  time: Date,
  thrustAcceleration?: SpaceshipVelocity,
): SpaceshipVelocity {
  return WorldService.calculateAcceleration(
    position,
    (currentPosition) =>
      calculateGravityAcceleration(currentPosition, world, time),
    thrustAcceleration,
  );
}

