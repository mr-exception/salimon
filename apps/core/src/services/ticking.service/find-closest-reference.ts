import type { SpaceshipDocument } from '@models';
import { SPACESHIP_RADIUS_METERS } from './constants';
import { getBodyPositions, getBodyVelocity } from './get-spaceship-update';
import type { WorldSnapshot } from './types';

export function findClosestReference(
  spaceship: SpaceshipDocument,
  simulatedAt: Date,
  world: WorldSnapshot,
) {
  const spaceshipPosition = {
    x: Number(spaceship.position.x),
    y: Number(spaceship.position.y),
  };
  const positions = getBodyPositions(world, simulatedAt);
  let closest:
    | {
        body: WorldSnapshot['bodies'][number];
        position: { x: number; y: number };
        velocity: { x: number; y: number };
        surfaceDistance: number;
      }
    | undefined;

  for (const body of world.bodies) {
    const position = positions.get(body.name);
    if (!position) continue;

    const centerDistance = Math.hypot(
      spaceshipPosition.x - position.x,
      spaceshipPosition.y - position.y,
    );
    const surfaceDistance = Math.max(
      0,
      centerDistance - Number(body.radius) - SPACESHIP_RADIUS_METERS,
    );
    if (closest && surfaceDistance >= closest.surfaceDistance) continue;

    closest = {
      body,
      position,
      velocity: getBodyVelocity(world, body.name, simulatedAt),
      surfaceDistance,
    };
  }

  return closest;
}

