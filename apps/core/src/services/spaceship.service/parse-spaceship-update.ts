import type { SpaceshipMotionState } from '@models';
import {
  MAX_HULL_DURABILITY,
  MAX_THRUSTER_DURABILITY,
  SPACESHIP_THRUSTER_COUNT,
} from './constants';

const INTEGER_PATTERN = /^-?\d+$/;

export function parseSpaceshipUpdate(body: unknown) {
  if (!body || typeof body !== 'object') {
    throw new Error('Request body must be an object');
  }

  const candidate = body as Record<string, unknown>;
  const position = candidate.position;
  if (!position || typeof position !== 'object') {
    throw new Error('position is required');
  }

  const positionCandidate = position as Record<string, unknown>;
  const { x, y, relativeTo } = positionCandidate;
  if (
    typeof x !== 'string' ||
    !INTEGER_PATTERN.test(x) ||
    typeof y !== 'string' ||
    !INTEGER_PATTERN.test(y)
  ) {
    throw new Error('position.x and position.y must be integer strings');
  }
  if (
    relativeTo !== undefined &&
    (typeof relativeTo !== 'string' || relativeTo.length === 0)
  ) {
    throw new Error('position.relativeTo must be a non-empty string');
  }

  const { direction, speed, velocity, motionState, stats } = candidate;
  if (
    typeof direction !== 'number' ||
    !Number.isFinite(direction) ||
    direction < 0 ||
    direction >= 360
  ) {
    throw new Error('direction must be a number from 0 up to 360');
  }
  if (typeof speed !== 'string' || !/^\d+$/.test(speed)) {
    throw new Error('speed must be a non-negative integer string');
  }
  if (!velocity || typeof velocity !== 'object') {
    throw new Error('velocity is required');
  }
  const velocityCandidate = velocity as Record<string, unknown>;
  if (
    typeof velocityCandidate.x !== 'number' ||
    !Number.isFinite(velocityCandidate.x) ||
    typeof velocityCandidate.y !== 'number' ||
    !Number.isFinite(velocityCandidate.y)
  ) {
    throw new Error('velocity.x and velocity.y must be finite numbers');
  }
  if (
    motionState !== 'flying' &&
    motionState !== 'landed' &&
    motionState !== 'crashed'
  ) {
    throw new Error('motionState must be flying, landed, or crashed');
  }
  const parsedMotionState: SpaceshipMotionState = motionState;
  if (!stats || typeof stats !== 'object') {
    throw new Error('stats is required');
  }
  const statsCandidate = stats as Record<string, unknown>;
  if (
    typeof statsCandidate.fuelKns !== 'number' ||
    !Number.isFinite(statsCandidate.fuelKns) ||
    statsCandidate.fuelKns < 0
  ) {
    throw new Error('stats.fuelKns must be a non-negative finite number');
  }
  const hullDurability = statsCandidate.hullDurability;
  const thrusterDurability = statsCandidate.thrusterDurability;
  if (
    typeof hullDurability !== 'number' ||
    !Number.isFinite(hullDurability) ||
    hullDurability < 0 ||
    hullDurability > MAX_HULL_DURABILITY
  ) {
    throw new Error('stats.hullDurability must be between 0 and 200');
  }
  if (
    !Array.isArray(thrusterDurability) ||
    thrusterDurability.length !== SPACESHIP_THRUSTER_COUNT ||
    thrusterDurability.some(
      (value) =>
        typeof value !== 'number' ||
        !Number.isFinite(value) ||
        value < 0 ||
        value > MAX_THRUSTER_DURABILITY,
    )
  ) {
    throw new Error(
      'stats.thrusterDurability must contain four values from 0 to 100',
    );
  }

  return {
    position: {
      x,
      y,
      ...(relativeTo === undefined ? {} : { relativeTo }),
    },
    direction,
    speed,
    velocity: { x: velocityCandidate.x, y: velocityCandidate.y },
    motionState: parsedMotionState,
    stats: {
      fuelKns: statsCandidate.fuelKns,
      hullDurability,
      thrusterDurability,
    },
  };
}

