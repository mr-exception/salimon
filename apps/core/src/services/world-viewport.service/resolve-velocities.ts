import type { WorldBodyDocument } from '@models';
import type { Velocity } from '@repo/types';
import type { Coordinate } from './types';

export function resolveVelocities(
  bodies: WorldBodyDocument[],
  positions: Map<string, Coordinate>,
) {
  const bodiesByName = new Map(bodies.map((body) => [body.name, body]));
  const velocitiesByName = new Map<string, Velocity>();

  function resolve(
    body: WorldBodyDocument,
    ancestors = new Set<string>(),
  ): Velocity {
    const cached = velocitiesByName.get(body.name);
    if (cached) return cached;
    if (ancestors.has(body.name)) {
      throw new Error(`Circular velocity reference involving ${body.name}`);
    }

    const centerName = body.orbitalCenter;
    const center = centerName ? bodiesByName.get(centerName) : undefined;
    const nextAncestors = new Set(ancestors).add(body.name);
    const centerVelocity: Velocity = center
      ? resolve(center, nextAncestors)
      : { x: 0, y: 0 };
    if (!centerName || !center || body.speed === '0') {
      velocitiesByName.set(body.name, centerVelocity);
      return centerVelocity;
    }

    const bodyPosition = positions.get(body.name);
    const centerPosition = positions.get(centerName);
    if (!bodyPosition || !centerPosition) {
      velocitiesByName.set(body.name, centerVelocity);
      return centerVelocity;
    }

    const x = Number(bodyPosition.x - centerPosition.x);
    const y = Number(bodyPosition.y - centerPosition.y);
    const radius = Math.hypot(x, y);
    if (radius === 0) {
      velocitiesByName.set(body.name, centerVelocity);
      return centerVelocity;
    }

    const direction = body.clockwise ? 1 : -1;
    const speed = Number(BigInt(body.speed));
    const velocity = {
      x: centerVelocity.x + (direction * -y * speed) / radius,
      y: centerVelocity.y + (direction * x * speed) / radius,
    };
    velocitiesByName.set(body.name, velocity);
    return velocity;
  }

  bodies.forEach((body) => resolve(body));
  return velocitiesByName;
}

