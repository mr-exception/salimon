import type { SpaceshipDocument } from '@models';
import { WorldService } from '@repo/world';
import { RepositoryService } from '../repository.service';
import {
  FREE_FLIGHT_BODY_RADIUS_CLEARANCE_RATIO,
  SPACESHIP_RADIUS_METERS,
} from './constants';
import type { parseSpaceshipUpdate } from './parse-spaceship-update';

type SpaceshipUpdate = ReturnType<typeof parseSpaceshipUpdate>;

export function updateSpaceship(
  securityCode: string,
  update: SpaceshipUpdate,
): Promise<SpaceshipDocument | undefined> {
  const now = new Date();
  return RepositoryService.updateSpaceshipBySecurityCode(securityCode, {
    ...update,
    simulatedAt: now,
    updatedAt: now,
  });
}

export async function saveSpaceship(
  securityCode: string,
  update: SpaceshipUpdate,
): Promise<SpaceshipDocument | undefined> {
  if (update.motionState !== 'flying') {
    throw new Error('Spaceship must be free flying to save');
  }
  if (update.activeFeature) {
    throw new Error('All thrusters must be off to save');
  }
  await assertSpaceshipHasFreeFlightClearance(update);

  return updateSpaceship(securityCode, update);
}

async function assertSpaceshipHasFreeFlightClearance(update: SpaceshipUpdate) {
  const worldData = await RepositoryService.getWorldData();
  const bodies = [...worldData.planets, ...worldData.moons, ...worldData.stars];
  const bodiesByName = new Map(bodies.map((body) => [body.name, body]));
  const bodyPositions = WorldService.getBodyPositions(
    { bodies, bodiesByName },
    new Date(),
  );
  const spaceshipPosition = {
    x: Number(update.position.x),
    y: Number(update.position.y),
  };

  if (update.position.relativeTo) {
    const referencePosition = bodyPositions.get(update.position.relativeTo);
    if (!referencePosition) {
      throw new Error(
        `Position reference ${update.position.relativeTo} was not found`,
      );
    }
    spaceshipPosition.x += referencePosition.x;
    spaceshipPosition.y += referencePosition.y;
  }

  for (const body of bodies) {
    const bodyPosition = bodyPositions.get(body.name);
    if (!bodyPosition) continue;

    const centerDistance = Math.hypot(
      spaceshipPosition.x - bodyPosition.x,
      spaceshipPosition.y - bodyPosition.y,
    );
    const surfaceDistance = Math.max(
      0,
      centerDistance - Number(body.radius) - SPACESHIP_RADIUS_METERS,
    );
    const minimumSurfaceDistance =
      Number(body.radius) * FREE_FLIGHT_BODY_RADIUS_CLEARANCE_RATIO;

    if (surfaceDistance >= minimumSurfaceDistance) continue;

    throw new Error(
      `Spaceship must be at least ${Math.round(
        minimumSurfaceDistance,
      ).toLocaleString()} m from ${body.name}'s surface to save`,
    );
  }
}
