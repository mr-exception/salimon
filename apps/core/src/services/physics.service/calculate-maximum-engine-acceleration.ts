import { WorldService } from '@repo/world';

export function calculateMaximumEngineAcceleration(maximumThrustPercent = 100) {
  return WorldService.calculateMaximumEngineAcceleration(maximumThrustPercent);
}

