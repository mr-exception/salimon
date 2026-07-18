import type { SpaceshipDocument } from '@models';
import { WorldSandbox } from '@repo/sandbox';
import { MAX_ENGINE_THRUST_KN } from '@repo/world';
import { RepositoryService } from '../repository.service';
import { tickingState } from './state';

const THRUSTER_COUNT = 4;

export async function startSpaceshipThrustersFeature(
  spaceship: SpaceshipDocument,
  params: {
    thrusters: { powerPercent: number; active: boolean }[];
  },
) {
  if (spaceship.motionState === 'crashed') return undefined;

  const thrusters = normalizeThrusters(params.thrusters);
  const force = getThrusterForce(thrusters);
  if (!force) return undefined;

  const simulatedAt = new Date();
  const sandbox = tickingState.sandbox;
  const object =
    sandbox?.getObject(
      WorldSandbox.getSpaceshipObjectId(spaceship.securityCode),
    ) ?? sandbox?.loadSpaceship(spaceship);
  if (!sandbox || !object) return undefined;

  sandbox.launchSpaceship(spaceship.securityCode, simulatedAt.getTime());
  object.force({
    id: 'spaceship:thrusters',
    ...force,
    durationMs: Number.MAX_SAFE_INTEGER,
  });
  const snapshot = sandbox.getSpaceshipSnapshot(object, simulatedAt.getTime());
  if (!snapshot) return undefined;

  return RepositoryService.updatePropagatedSpaceship(spaceship, {
    activeFeature: {
      type: 'thrusters',
      thrusters,
      elapsedSeconds: 0,
    },
    motionState: 'flying',
    position: snapshot.position,
    velocity: snapshot.velocity,
    speed: snapshot.speed,
    direction: snapshot.direction,
    simulatedAt,
    updatedAt: simulatedAt,
  });
}

function normalizeThrusters(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  if (!Array.isArray(thrusters)) {
    throw new Error('Invalid spaceship thrusters.');
  }

  const normalizedThrusters = thrusters
    .slice(0, THRUSTER_COUNT)
    .map((thruster) => ({
      powerPercent: Number(thruster.powerPercent),
      active: Boolean(thruster.active),
    }));

  if (
    normalizedThrusters.length !== THRUSTER_COUNT ||
    normalizedThrusters.some(
      (thruster) =>
        !Number.isFinite(thruster.powerPercent) ||
        thruster.powerPercent < 0 ||
        thruster.powerPercent > 100,
    )
  ) {
    throw new Error('Invalid spaceship thrusters.');
  }

  return normalizedThrusters;
}

function getThrusterForce(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  const force = { x: 0, y: 0 };

  thrusters.forEach((thruster, index) => {
    if (!thruster.active || thruster.powerPercent <= 0) return;

    const thrustN =
      MAX_ENGINE_THRUST_KN * 1_000 * (thruster.powerPercent / 100);
    if (index === 0) force.y += thrustN;
    if (index === 1) force.x -= thrustN;
    if (index === 2) force.y -= thrustN;
    if (index === 3) force.x += thrustN;
  });

  return force.x === 0 && force.y === 0 ? undefined : force;
}
