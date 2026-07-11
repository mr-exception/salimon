import { WorldService } from '@repo/world';
import type { WorldSnapshot } from '../ticking.service/types';

export function getBodyPositions(world: WorldSnapshot, time: Date) {
  return WorldService.getBodyPositions(world, time);
}

