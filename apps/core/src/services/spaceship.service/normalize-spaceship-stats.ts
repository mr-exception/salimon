import type { SpaceshipStats } from '@models';
import {
  INITIAL_SPACESHIP_FUEL_KNS,
  MAX_HULL_DURABILITY,
  MAX_THRUSTER_DURABILITY,
  SPACESHIP_THRUSTER_COUNT,
} from './constants';

export function normalizeSpaceshipStats(
  stats: Partial<SpaceshipStats> | undefined,
): SpaceshipStats {
  return {
    fuelKns: stats?.fuelKns ?? INITIAL_SPACESHIP_FUEL_KNS,
    hullDurability: stats?.hullDurability ?? MAX_HULL_DURABILITY,
    thrusterDurability: Array.from(
      { length: SPACESHIP_THRUSTER_COUNT },
      (_, index) =>
        stats?.thrusterDurability?.[index] ?? MAX_THRUSTER_DURABILITY,
    ),
  };
}

