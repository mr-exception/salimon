import type { SpaceshipMotionState } from '@models';
import { WorldService } from '@repo/world';

export function getImpactMotionState(
  impactSpeed: number,
): SpaceshipMotionState {
  return WorldService.getImpactMotionState(impactSpeed);
}

