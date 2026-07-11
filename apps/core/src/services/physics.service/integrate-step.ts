import type { SpaceshipVelocity } from '@models';
import { WorldService } from '@repo/world';
import type { Motion, WorldSnapshot } from '../ticking.service/types';
import { calculateAcceleration } from './calculate-acceleration';

export function integrateStep(
  motion: Motion,
  startedAt: Date,
  seconds: number,
  world: WorldSnapshot,
  thrustAcceleration?: SpaceshipVelocity,
): Motion {
  return WorldService.integrateStep(motion, seconds, (position, offset) =>
    calculateAcceleration(
      position,
      world,
      new Date(startedAt.getTime() + offset * 1_000),
      thrustAcceleration,
    ),
  );
}

