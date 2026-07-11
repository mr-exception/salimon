import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';
import type { WorldSnapshot } from '../ticking.service/types';
import { calculateGravityAcceleration } from './calculate-gravity-acceleration';

export function calculateRequiredBurnAcceleration(
  targetVelocity: SpaceshipVelocity,
  remainingSeconds: number,
  currentVelocity: SpaceshipVelocity,
  position: SpaceshipVelocity,
  world: WorldSnapshot,
  time: Date,
): SpaceshipVelocity {
  return WorldService.calculateRequiredBurnAcceleration(
    targetVelocity,
    remainingSeconds,
    currentVelocity,
    position,
    (currentPosition) =>
      calculateGravityAcceleration(currentPosition, world, time),
  );
}

