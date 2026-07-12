import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import { SpaceshipService } from '../spaceship.service';
import {
  SPACESHIP_LAUNCH_CLEARANCE_METERS,
  SPACESHIP_RADIUS_METERS,
} from './constants';
import { findClosestReference } from './find-closest-reference';
import {
  createTargetSpeedFeature,
  getBodyPositions,
  getBodyVelocity,
} from './get-spaceship-update';
import { loadWorldSnapshot } from './load-world-snapshot';
import { updateSpaceship } from './update-spaceship';

export async function startSpaceshipTargetSpeedFeature(
  spaceship: SpaceshipDocument,
  params: {
    targetSpeedMetersPerSecond: number;
    maximumThrustPercent: number;
    targetDirection?: number;
  },
) {
  const simulatedAt = new Date();
  const world = await loadWorldSnapshot();
  const currentSpaceship = await updateSpaceship(spaceship, simulatedAt, world);
  const motionState =
    currentSpaceship.motionState ??
    (currentSpaceship.speed === '0' ? 'landed' : 'flying');
  const isLaunchingFromSurface = motionState !== 'flying';
  const currentReferenceName = currentSpaceship.position.relativeTo;
  const currentReferenceBody = currentReferenceName
    ? world.bodiesByName.get(currentReferenceName)
    : undefined;
  const currentReferencePosition = currentReferenceName
    ? getBodyPositions(world, simulatedAt).get(currentReferenceName)
    : undefined;
  const currentReferenceVelocity =
    currentReferenceName && currentReferenceBody
      ? getBodyVelocity(world, currentReferenceName, simulatedAt)
      : undefined;
  const relativePosition = {
    x: Number(currentSpaceship.position.x),
    y: Number(currentSpaceship.position.y),
  };
  const absolutePosition = currentReferencePosition
    ? {
        x: currentReferencePosition.x + relativePosition.x,
        y: currentReferencePosition.y + relativePosition.y,
      }
    : relativePosition;
  const relativeVelocity =
    SpaceshipService.getSpaceshipVelocity(currentSpaceship);
  const worldVelocity = currentReferenceVelocity
    ? {
        x: currentReferenceVelocity.x + relativeVelocity.x,
        y: currentReferenceVelocity.y + relativeVelocity.y,
      }
    : relativeVelocity;
  const planningSpaceship = {
    ...currentSpaceship,
    position: {
      x: Math.round(absolutePosition.x).toString(),
      y: Math.round(absolutePosition.y).toString(),
    },
    velocity: worldVelocity,
  };
  const launchReference =
    isLaunchingFromSurface &&
    currentReferenceBody &&
    currentReferencePosition &&
    currentReferenceVelocity
      ? {
          body: currentReferenceBody,
          position: currentReferencePosition,
          velocity: currentReferenceVelocity,
          surfaceDistance: 0,
        }
      : findClosestReference(planningSpaceship, simulatedAt, world);
  const activeFeature = createTargetSpeedFeature(
    planningSpaceship,
    simulatedAt,
    world,
    params.targetSpeedMetersPerSecond,
    params.maximumThrustPercent,
    params.targetDirection,
  );
  if (!activeFeature) return undefined;

  const referencePosition = launchReference?.position;
  const referenceBody = launchReference?.body;
  const launchRelativePosition = referencePosition
    ? {
        x: absolutePosition.x - referencePosition.x,
        y: absolutePosition.y - referencePosition.y,
      }
    : absolutePosition;
  const relativeRadius = Math.hypot(
    launchRelativePosition.x,
    launchRelativePosition.y,
  );
  const launchRadius =
    isLaunchingFromSurface && referenceBody && relativeRadius > 0
      ? Number(referenceBody.radius) +
        SPACESHIP_RADIUS_METERS +
        SPACESHIP_LAUNCH_CLEARANCE_METERS
      : relativeRadius;
  const launchPosition =
    referencePosition && relativeRadius > 0
      ? {
          x: (launchRelativePosition.x / relativeRadius) * launchRadius,
          y: (launchRelativePosition.y / relativeRadius) * launchRadius,
        }
      : launchRelativePosition;
  const worldPosition = referencePosition
    ? {
        x: Math.round(referencePosition.x + launchPosition.x).toString(),
        y: Math.round(referencePosition.y + launchPosition.y).toString(),
      }
    : currentSpaceship.position;

  return RepositoryService.updatePropagatedSpaceship(currentSpaceship, {
    activeFeature,
    motionState: 'flying',
    position: worldPosition,
    velocity: worldVelocity,
    simulatedAt,
    updatedAt: simulatedAt,
  });
}
