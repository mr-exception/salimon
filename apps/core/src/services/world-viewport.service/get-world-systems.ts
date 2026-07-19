import type { SerializedWorldBody, SerializedWorldSystems } from '@repo/types';
import { WorldSystemModel } from '@models';
import type { WorldViewportOptions, WorldViewportRequest } from './types';

type Coordinate = {
  x: bigint;
  y: bigint;
};

type ViewportRectangle = {
  left: bigint;
  right: bigint;
  top: bigint;
  bottom: bigint;
};

export async function getWorldSystems(
  request: WorldViewportRequest,
  options: WorldViewportOptions = {},
): Promise<SerializedWorldSystems> {
  const systems = (await WorldSystemModel.findAllSystems()).map(
    (system) => system.bodies,
  );
  const viewport = getViewportRectangle(request);
  const requiredBodyNames = new Set(options.requiredBodyNames);

  parseRequiredBodyNames(request.requiredBodyNames).forEach((bodyName) =>
    requiredBodyNames.add(bodyName),
  );

  const bodies = systems.flat();
  const positionsByName = resolvePositions(bodies);
  const visibleSystems = systems.filter((system) => {
    const primary = getPrimaryBody(system);
    const position = primary ? positionsByName.get(primary.name) : undefined;
    const isInViewport = position
      ? isInsideViewport(position, viewport)
      : false;
    const isRequired = system.some((body) => requiredBodyNames.has(body.name));

    return isInViewport || isRequired;
  });
  return {
    systems: visibleSystems,
  };
}

function parseInteger(value: string | undefined, name: string) {
  if (!value || !/^-?\d+$/.test(value)) {
    throw new Error(`${name} must be an integer in meters`);
  }

  return BigInt(value);
}

function getViewportRectangle(
  request: WorldViewportRequest,
): ViewportRectangle {
  const x1 = parseInteger(request.x1 ?? request.left, 'x1');
  const y1 = parseInteger(request.y1 ?? request.top, 'y1');
  const x2 = parseInteger(request.x2 ?? request.right, 'x2');
  const y2 = parseInteger(request.y2 ?? request.bottom, 'y2');

  return {
    left: x1 < x2 ? x1 : x2,
    right: x1 > x2 ? x1 : x2,
    top: y1 < y2 ? y1 : y2,
    bottom: y1 > y2 ? y1 : y2,
  };
}

function parseRequiredBodyNames(requiredBodyNames: string | string[] = []) {
  return (
    Array.isArray(requiredBodyNames) ? requiredBodyNames : [requiredBodyNames]
  )
    .flatMap((value) => value.split(','))
    .map((bodyName) => bodyName.trim())
    .filter(Boolean);
}

function getPrimaryBody(system: SerializedWorldBody[]) {
  return system.find((body) => body.type === 'star') ?? system[0];
}

function resolvePositions(bodies: SerializedWorldBody[]) {
  const bodiesByName = new Map(bodies.map((body) => [body.name, body]));
  const positionsByName = new Map<string, Coordinate>();

  function resolve(
    body: SerializedWorldBody,
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

      const referencePosition = resolve(
        reference,
        new Set(ancestors).add(body.name),
      );
      position.x += referencePosition.x;
      position.y += referencePosition.y;
    }

    positionsByName.set(body.name, position);
    return position;
  }

  bodies.forEach((body) => resolve(body));
  return positionsByName;
}

function isInsideViewport(position: Coordinate, viewport: ViewportRectangle) {
  return (
    position.x >= viewport.left &&
    position.x <= viewport.right &&
    position.y >= viewport.top &&
    position.y <= viewport.bottom
  );
}
