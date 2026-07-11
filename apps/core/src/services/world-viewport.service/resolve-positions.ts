import type { WorldBodyDocument } from '@models';
import type { Coordinate } from './types';

export function resolvePositions(bodies: WorldBodyDocument[]) {
  const bodiesByName = new Map(bodies.map((body) => [body.name, body]));
  const positionsByName = new Map<string, Coordinate>();

  function resolve(
    body: WorldBodyDocument,
    ancestors = new Set<string>(),
  ): Coordinate {
    const cached = positionsByName.get(body.name);
    if (cached) return cached;
    if (ancestors.has(body.name)) {
      throw new Error(`Circular position reference involving ${body.name}`);
    }

    const position = {
      x: BigInt(body.position.x),
      y: BigInt(body.position.y),
    };
    const referenceName = body.position.relativeTo;
    if (referenceName) {
      const reference = bodiesByName.get(referenceName);
      if (!reference) {
        throw new Error(
          `Position reference ${referenceName} for ${body.name} was not found`,
        );
      }

      const nextAncestors = new Set(ancestors).add(body.name);
      const referencePosition = resolve(reference, nextAncestors);
      position.x += referencePosition.x;
      position.y += referencePosition.y;
    }

    positionsByName.set(body.name, position);
    return position;
  }

  bodies.forEach((body) => resolve(body));
  return positionsByName;
}

