import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';

export function add(
  value: SpaceshipVelocity,
  change: SpaceshipVelocity,
  scale = 1,
): SpaceshipVelocity {
  return WorldService.add(value, change, scale);
}

