import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';
import type { WorldSnapshot } from '../ticking.service/types';
import { calculateGravityAcceleration } from './calculate-gravity-acceleration';

export function calculateTargetSpeedBurnDuration(
  targetVelocity: SpaceshipVelocity,
  currentVelocity: SpaceshipVelocity,
  position: SpaceshipVelocity,
  maximumAcceleration: number,
  world: WorldSnapshot,
  time: Date,
) {
  return WorldService.calculateTargetSpeedBurnDuration(
    targetVelocity,
    currentVelocity,
    position,
    maximumAcceleration,
    (currentPosition) =>
      calculateGravityAcceleration(currentPosition, world, time),
  );
}
