import type { SerializedWorldSystems } from '@repo/types';
import { WorldSystemModel } from '@models';
import type { WorldViewportOptions, WorldViewportRequest } from './types';

const METERS_PER_LIGHT_YEAR = 9_460_730_472_580_800n;
const MIN_VIEWPORT_SIZE = METERS_PER_LIGHT_YEAR * 10n;
const WORLD_SECTOR_SIZE = METERS_PER_LIGHT_YEAR * 10n;
const WORLD_SECTOR_HALF_SIZE = WORLD_SECTOR_SIZE / 2n;

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
  const viewport =
    getSectorViewportRectangle(request) ?? getViewportRectangle(request);
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

  return enforceMinimumViewportSize({
    left: x1 < x2 ? x1 : x2,
    right: x1 > x2 ? x1 : x2,
    top: y1 < y2 ? y1 : y2,
    bottom: y1 > y2 ? y1 : y2,
  });
}

function getSectorViewportRectangle(
  request: WorldViewportRequest,
): ViewportRectangle | undefined {
  if (request.sectorX === undefined && request.sectorY === undefined) {
    return undefined;
  }

  const sectorX = parseSectorCoordinate(request.sectorX, 'sectorX');
  const sectorY = parseSectorCoordinate(request.sectorY, 'sectorY');
  const left = BigInt(sectorX) * WORLD_SECTOR_SIZE - WORLD_SECTOR_HALF_SIZE;
  const top = BigInt(sectorY) * WORLD_SECTOR_SIZE - WORLD_SECTOR_HALF_SIZE;

  return {
    left,
    right: left + WORLD_SECTOR_SIZE,
    top,
    bottom: top + WORLD_SECTOR_SIZE,
  };
}

function parseSectorCoordinate(
  value: string | number | undefined,
  name: string,
) {
  const parsed = typeof value === 'number' ? value : Number(value);
  if (!Number.isInteger(parsed)) {
    throw new Error(`${name} must be an integer`);
  }

  return parsed;
}

function enforceMinimumViewportSize(
  viewport: ViewportRectangle,
): ViewportRectangle {
  const [left, right] = enforceMinimumRangeSize(
    viewport.left,
    viewport.right,
    MIN_VIEWPORT_SIZE,
  );
  const [top, bottom] = enforceMinimumRangeSize(
    viewport.top,
    viewport.bottom,
    MIN_VIEWPORT_SIZE,
  );

  return { left, right, top, bottom };
}

function enforceMinimumRangeSize(start: bigint, end: bigint, minimum: bigint) {
  const size = end - start;
  if (size >= minimum) return [start, end] as const;

  const extra = minimum - size;
  const before = extra / 2n;
  const after = extra - before;

  return [start - before, end + after] as const;
}

function parseRequiredBodyNames(requiredBodyNames: string | string[] = []) {
  return (
    Array.isArray(requiredBodyNames) ? requiredBodyNames : [requiredBodyNames]
  )
    .flatMap((value) => value.split(','))
    .map((bodyName) => bodyName.trim())
    .filter(Boolean);
}
