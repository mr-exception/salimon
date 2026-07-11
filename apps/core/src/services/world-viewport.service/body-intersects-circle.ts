import type { WorldBodyDocument } from '@models';
import type { Coordinate } from './types';

function isInsideCircle(
  position: Coordinate,
  center: Coordinate,
  radiusSquared: bigint,
) {
  const deltaX = position.x - center.x;
  const deltaY = position.y - center.y;
  return deltaX * deltaX + deltaY * deltaY <= radiusSquared;
}

export function bodyIntersectsCircle(
  body: WorldBodyDocument,
  position: Coordinate,
  center: Coordinate,
  radius: bigint,
) {
  const bodyRadius = BigInt(body.radius);
  return isInsideCircle(position, center, (radius + bodyRadius) ** 2n);
}

