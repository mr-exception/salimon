import type { SpaceshipDocument } from '@models';
import { SpaceshipService } from '../spaceship.service';
import { getBodyPositions, getBodyVelocity } from './get-spaceship-update';
import type { WorldSnapshot } from './types';

export function getAbsoluteSpaceshipUpdate(
  spaceship: SpaceshipDocument,
  simulatedAt: Date,
  world: WorldSnapshot,
): Partial<SpaceshipDocument> | undefined {
  if (spaceship.motionState !== 'flying') return undefined;

  const referenceName = spaceship.position.relativeTo;
  if (!referenceName) return undefined;

  const referencePosition = getBodyPositions(world, simulatedAt).get(
    referenceName,
  );
  if (!referencePosition) return undefined;

  const relativePosition = {
    x: Number(spaceship.position.x),
    y: Number(spaceship.position.y),
  };
  const referenceVelocity = getBodyVelocity(world, referenceName, simulatedAt);
  const relativeVelocity = SpaceshipService.getSpaceshipVelocity(spaceship);

  return {
    position: {
      x: Math.round(referencePosition.x + relativePosition.x).toString(),
      y: Math.round(referencePosition.y + relativePosition.y).toString(),
    },
    velocity: {
      x: referenceVelocity.x + relativeVelocity.x,
      y: referenceVelocity.y + relativeVelocity.y,
    },
  };
}

