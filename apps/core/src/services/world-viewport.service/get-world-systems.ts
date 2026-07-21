import type { SerializedWorldSystems } from '@repo/types';
import { WorldSystemModel } from '@models';
import type { WorldViewportOptions, WorldViewportRequest } from './types';

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
  const viewport = getViewportRectangle(request);
  const requiredBodyNames = new Set(options.requiredBodyNames);

  parseRequiredBodyNames(request.requiredBodyNames).forEach((bodyName) =>
    requiredBodyNames.add(bodyName),
  );

  const visibleSystems = await WorldSystemModel.findSystemsInViewport(
    viewport,
    requiredBodyNames,
  );

  return {
    systems: visibleSystems.map((system) => system.bodies),
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
