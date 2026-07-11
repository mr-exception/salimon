import type { SpaceshipDocument, SpaceshipVelocity } from '@models';
import { SPACESHIP_MASS_KG } from '@repo/world';
import {
  SPACESHIP_THRUSTER_COUNT,
  SpaceshipService,
} from '../spaceship.service';
import { THRUSTER_DURABILITY_DRAIN_RATE } from '../ticking.service/constants';
import type { ActiveThrusters } from './types';

export function getActiveThrusters(
  accelerationValue: SpaceshipVelocity | undefined,
  stats: SpaceshipDocument['stats'],
): ActiveThrusters | undefined {
  if (!accelerationValue) return undefined;

  const normalizedStats = SpaceshipService.normalizeSpaceshipStats(stats);
  const thrustByIndex = Array<number>(SPACESHIP_THRUSTER_COUNT).fill(0);
  const effectiveAcceleration = { x: 0, y: 0 };
  const xIndex = accelerationValue.x < 0 ? 1 : 3;
  const yIndex = accelerationValue.y < 0 ? 2 : 0;

  if (
    Math.abs(accelerationValue.x) > 1e-8 &&
    normalizedStats.thrusterDurability[xIndex] > 0
  ) {
    effectiveAcceleration.x = accelerationValue.x;
    thrustByIndex[xIndex] =
      (Math.abs(accelerationValue.x) * SPACESHIP_MASS_KG) / 1_000;
  }
  if (
    Math.abs(accelerationValue.y) > 1e-8 &&
    normalizedStats.thrusterDurability[yIndex] > 0
  ) {
    effectiveAcceleration.y = accelerationValue.y;
    thrustByIndex[yIndex] =
      (Math.abs(accelerationValue.y) * SPACESHIP_MASS_KG) / 1_000;
  }

  const activeIndexes = thrustByIndex
    .map((thrust, index) => ({ index, thrust }))
    .filter(({ thrust }) => thrust > 0);
  if (activeIndexes.length === 0) return undefined;

  return {
    effectiveAcceleration,
    thrustByIndex,
    totalKilonewtons: activeIndexes.reduce(
      (total, { thrust }) => total + thrust,
      0,
    ),
    availableSeconds: Math.min(
      ...activeIndexes.map(
        ({ index, thrust }) =>
          normalizedStats.thrusterDurability[index] /
          ((thrust / 100) * THRUSTER_DURABILITY_DRAIN_RATE),
      ),
    ),
  };
}

