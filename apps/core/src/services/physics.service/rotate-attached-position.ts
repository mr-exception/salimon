import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';

export function rotateAttachedPosition(
  position: SpaceshipVelocity,
  elapsedSeconds: number,
  rotationPeriodSeconds: number | undefined,
  collisionRadius: number,
): SpaceshipVelocity {
  return WorldService.rotateAttachedPosition(
    position,
    elapsedSeconds,
    rotationPeriodSeconds,
    collisionRadius,
  );
}

