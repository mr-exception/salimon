import type { WorldViewportRequest } from './types';

function parseInteger(value: string | undefined, name: string): bigint {
  if (!value || !/^-?\d+$/.test(value)) {
    throw new Error(`${name} must be an integer in meters`);
  }

  return BigInt(value);
}

export function getSearchArea(request: WorldViewportRequest) {
  const coordinate = request.coordinate?.split(',');
  const x = parseInteger(request.x ?? coordinate?.[0]?.trim(), 'x');
  const y = parseInteger(request.y ?? coordinate?.[1]?.trim(), 'y');
  const radius = parseInteger(request.radius, 'radius');

  if (radius < 0n) {
    throw new Error('radius must be greater than or equal to zero');
  }

  return { x, y, radius };
}

