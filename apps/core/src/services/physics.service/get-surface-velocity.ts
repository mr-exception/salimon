import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';

export function getSurfaceVelocity(
  position: SpaceshipVelocity,
  rotationPeriodSeconds: number | undefined,
): SpaceshipVelocity {
  return WorldService.getSurfaceVelocity(position, rotationPeriodSeconds);
}

