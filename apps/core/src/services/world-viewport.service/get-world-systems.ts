import type { SerializedWorldSystems } from '@repo/types';
import type { SerializedWorldBody } from '@repo/types';
import { WorldSystemModel } from '@models';
import type { WorldViewportOptions, WorldViewportRequest } from './types';

const MIN_RENDER_SHAPE_SCREEN_WIDTH = 16;

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
  const zoom = parseZoom(request.zoom);

  const visibleSystems = await WorldSystemModel.findSystemsInViewport(
    viewport,
    requiredBodyNames,
  );

  return {
    systems: visibleSystems
      .map((system) =>
        system.bodies.filter((body) =>
          shouldTransmitBody(body, zoom, requiredBodyNames),
        ),
      )
      .filter((system) => system.length > 0),
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

function parseZoom(zoom: string | number | undefined) {
  if (zoom === undefined) return undefined;

  const value = typeof zoom === 'number' ? zoom : Number(zoom);
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error('zoom must be a positive number');
  }

  return value;
}

function shouldTransmitBody(
  body: SerializedWorldBody,
  zoom: number | undefined,
  requiredBodyNames: ReadonlySet<string>,
) {
  if (body.type === 'star' || zoom === undefined) return true;
  if (requiredBodyNames.has(body.name)) return true;

  return zoom >= getMinZoomRenderShape(body);
}

function getMinZoomRenderShape(body: SerializedWorldBody) {
  if (body.minZoomRenderShape !== undefined) return body.minZoomRenderShape;

  const radius = Number(body.radius);
  if (!Number.isFinite(radius) || radius <= 0) return 0;

  return MIN_RENDER_SHAPE_SCREEN_WIDTH / 2 / radius;
}
