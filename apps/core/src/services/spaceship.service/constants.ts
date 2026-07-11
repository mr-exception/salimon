import type { SpaceshipDto } from '@repo/types';

export const SECURITY_CODE_HEADER = 'x-spaceship-security-code';
export const INITIAL_SPACESHIP_FUEL_KNS = 1_000_000;
export const MAX_HULL_DURABILITY = 200;
export const MAX_THRUSTER_DURABILITY = 100;
export const SPACESHIP_THRUSTER_COUNT = 4;
export const DEFAULT_SPACESHIP = {
  position: {
    x: '6371200',
    y: '0',
    relativeTo: 'Earth',
  },
  direction: 0,
  speed: '0',
  velocity: { x: 0, y: 0 },
  motionState: 'landed',
  stats: {
    fuelKns: INITIAL_SPACESHIP_FUEL_KNS,
    hullDurability: MAX_HULL_DURABILITY,
    thrusterDurability: Array(SPACESHIP_THRUSTER_COUNT).fill(
      MAX_THRUSTER_DURABILITY,
    ),
  },
} satisfies Omit<SpaceshipDto, 'securityCode' | 'simulatedAt'>;

