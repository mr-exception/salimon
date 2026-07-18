import type { SpaceshipDocument } from '@models';
import { WorldSandbox } from '@repo/sandbox';
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
  const simulatedAt = new Date();
  const sandbox = tickingState.sandbox;
  sandbox?.getObject(
    WorldSandbox.getSpaceshipObjectId(spaceship.securityCode),
  ) ?? sandbox?.loadSpaceship(spaceship);
  const snapshot = sandbox?.startSpaceshipThrusters(
    spaceship.securityCode,
    thrusters,
    simulatedAt.getTime(),
  );
  if (!snapshot) return undefined;

  return RepositoryService.updateSpaceshipBySecurityCode(
    spaceship.securityCode,
    {
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
    },
  );
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
