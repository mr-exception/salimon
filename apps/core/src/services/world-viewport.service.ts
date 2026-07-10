import type { WorldBodyDocument } from '@models';
import type { SerializedWorldSystems, Velocity } from '@repo/types';
import { TickingService } from './ticking.service';

type Coordinate = {
  x: bigint;
  y: bigint;
};

type WorldBodyResponse = Omit<WorldBodyDocument, 'updatedAt'> & {
  velocity: Velocity;
  positionCapturedAt: string;
};

type PlanetSystem = {
  planet: WorldBodyResponse;
  moons: WorldBodyResponse[];
};

type StarSystem = {
  star: WorldBodyResponse;
  planets: PlanetSystem[];
};

export type WorldViewportRequest = {
  x?: string;
  y?: string;
  radius?: string;
  coordinate?: string;
};

function parseInteger(value: string | undefined, name: string): bigint {
  if (!value || !/^-?\d+$/.test(value)) {
    throw new Error(`${name} must be an integer in meters`);
  }

  return BigInt(value);
}

function getSearchArea(request: WorldViewportRequest) {
  const coordinate = request.coordinate?.split(',');
  const x = parseInteger(request.x ?? coordinate?.[0]?.trim(), 'x');
  const y = parseInteger(request.y ?? coordinate?.[1]?.trim(), 'y');
  const radius = parseInteger(request.radius, 'radius');

  if (radius < 0n) {
    throw new Error('radius must be greater than or equal to zero');
  }

  return { x, y, radius };
}

function resolvePositions(bodies: WorldBodyDocument[]) {
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

function isInsideCircle(
  position: Coordinate,
  center: Coordinate,
  radiusSquared: bigint,
) {
  const deltaX = position.x - center.x;
  const deltaY = position.y - center.y;
  return deltaX * deltaX + deltaY * deltaY <= radiusSquared;
}

function groupByOrbitalCenter(bodies: WorldBodyDocument[]) {
  const bodiesByOrbitalCenter = new Map<string | null, WorldBodyDocument[]>();

  for (const body of bodies) {
    const siblings = bodiesByOrbitalCenter.get(body.orbitalCenter) ?? [];
    siblings.push(body);
    bodiesByOrbitalCenter.set(body.orbitalCenter, siblings);
  }

  return bodiesByOrbitalCenter;
}

function resolveVelocities(
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

function withVelocity(
  body: WorldBodyDocument,
  velocities: Map<string, Velocity>,
): WorldBodyResponse {
  const { updatedAt, ...responseBody } = body;

  return {
    ...responseBody,
    velocity: velocities.get(body.name) ?? { x: 0, y: 0 },
    positionCapturedAt: updatedAt.toISOString(),
  };
}

export class WorldViewportService {
  static async getWorldSystems(
    request: WorldViewportRequest,
  ): Promise<SerializedWorldSystems> {
    const searchArea = getSearchArea(request);
    const { planets, moons, stars } =
      await TickingService.getWorldSystemsBodies();
    const orbitingBodies = [...planets, ...moons];
    const positions = resolvePositions([...orbitingBodies, ...stars]);
    const center = { x: searchArea.x, y: searchArea.y };
    const radiusSquared = searchArea.radius * searchArea.radius;
    const planetsByOrbitalCenter = groupByOrbitalCenter(orbitingBodies);
    const velocities = resolveVelocities([...orbitingBodies, ...stars], positions);
    const systems: StarSystem[] = stars
      .map((star) => {
        const planetSystems = (planetsByOrbitalCenter.get(star.name) ?? []).map(
          (planet) => ({
            planet,
            moons: planetsByOrbitalCenter.get(planet.name) ?? [],
          }),
        );
        const hasVisibleBody =
          isInsideCircle(positions.get(star.name)!, center, radiusSquared) ||
          planetSystems.some(
            ({ planet, moons: planetMoons }) =>
              isInsideCircle(
                positions.get(planet.name)!,
                center,
                radiusSquared,
              ) ||
              planetMoons.some((moon) =>
                isInsideCircle(
                  positions.get(moon.name)!,
                  center,
                  radiusSquared,
                ),
              ),
          );

        return hasVisibleBody
          ? {
              star: withVelocity(star, velocities),
              planets: planetSystems.map(({ planet, moons: planetMoons }) => ({
                planet: withVelocity(planet, velocities),
                moons: planetMoons.map((moon) => withVelocity(moon, velocities)),
              })),
            }
          : undefined;
      })
      .filter((system): system is StarSystem => system !== undefined);

    return { systems } as unknown as SerializedWorldSystems;
  }
}
