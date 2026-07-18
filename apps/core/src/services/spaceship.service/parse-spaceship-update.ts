import type { SpaceshipActiveFeature, SpaceshipMotionState } from '@models';
import { parseSpaceshipInventory } from './parse-spaceship-inventory';
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

  const { direction, speed, velocity, motionState, stats, activeFeature } =
    candidate;
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
    activeFeature: parseActiveFeature(activeFeature),
    ...(candidate.inventory === undefined
      ? {}
      : { inventory: parseSpaceshipInventory(candidate.inventory) }),
  };
}

function parseActiveFeature(value: unknown): SpaceshipActiveFeature | undefined {
  if (value === undefined || value === null) return undefined;
  if (!value || typeof value !== 'object') {
    throw new Error('activeFeature must be an object');
  }

  const candidate = value as Record<string, unknown>;
  if (
    candidate.type === 'thrusters' ||
    candidate.type === 'manual-force'
  ) {
    if (
      !Array.isArray(candidate.thrusters) ||
      typeof candidate.elapsedSeconds !== 'number' ||
      !Number.isFinite(candidate.elapsedSeconds) ||
      candidate.elapsedSeconds < 0
    ) {
      throw new Error('activeFeature thrusters are invalid');
    }

    return {
      type: candidate.type,
      elapsedSeconds: candidate.elapsedSeconds,
      thrusters: candidate.thrusters.map(parseThruster),
    };
  }

  if (candidate.type !== 'target-speed') {
    throw new Error('activeFeature type is invalid');
  }
  if (
    typeof candidate.targetSpeedMetersPerSecond !== 'number' ||
    !Number.isFinite(candidate.targetSpeedMetersPerSecond) ||
    typeof candidate.maximumThrustPercent !== 'number' ||
    !Number.isFinite(candidate.maximumThrustPercent) ||
    typeof candidate.maximumAcceleration !== 'number' ||
    !Number.isFinite(candidate.maximumAcceleration) ||
    typeof candidate.durationSeconds !== 'number' ||
    !Number.isFinite(candidate.durationSeconds) ||
    typeof candidate.elapsedSeconds !== 'number' ||
    !Number.isFinite(candidate.elapsedSeconds) ||
    !candidate.targetVelocity ||
    typeof candidate.targetVelocity !== 'object'
  ) {
    throw new Error('activeFeature target speed values are invalid');
  }
  const velocity = candidate.targetVelocity as Record<string, unknown>;
  if (
    typeof velocity.x !== 'number' ||
    !Number.isFinite(velocity.x) ||
    typeof velocity.y !== 'number' ||
    !Number.isFinite(velocity.y)
  ) {
    throw new Error('activeFeature target velocity is invalid');
  }

  return {
    type: 'target-speed',
    targetSpeedMetersPerSecond: candidate.targetSpeedMetersPerSecond,
    maximumThrustPercent: candidate.maximumThrustPercent,
    targetDirection:
      typeof candidate.targetDirection === 'number' &&
      Number.isFinite(candidate.targetDirection)
        ? candidate.targetDirection
        : undefined,
    targetVelocity: { x: velocity.x, y: velocity.y },
    maximumAcceleration: candidate.maximumAcceleration,
    durationSeconds: candidate.durationSeconds,
    elapsedSeconds: candidate.elapsedSeconds,
  };
}

function parseThruster(value: unknown) {
  if (!value || typeof value !== 'object') {
    throw new Error('activeFeature thruster must be an object');
  }
  const candidate = value as Record<string, unknown>;
  if (
    typeof candidate.active !== 'boolean' ||
    typeof candidate.powerPercent !== 'number' ||
    !Number.isFinite(candidate.powerPercent) ||
    candidate.powerPercent < 0 ||
    candidate.powerPercent > 100
  ) {
    throw new Error('activeFeature thruster values are invalid');
  }

  return {
    active: candidate.active,
    powerPercent: candidate.powerPercent,
  };
}
