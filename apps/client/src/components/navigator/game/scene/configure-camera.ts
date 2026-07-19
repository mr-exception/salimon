import type { Scene } from '.';
import {
  getRenderPosition,
  getWorldPositionFromRenderPosition,
} from '../get-render-position';
import { MAX_ZOOM } from './configure-input';

const LIGHT_YEAR_METERS = 9_460_730_472_580_800n;
const MILKY_WAY_DIAMETER_LIGHT_YEARS = 150_000n;
const WORLD_SIZE = MILKY_WAY_DIAMETER_LIGHT_YEARS * LIGHT_YEAR_METERS;

export const WORLD_RADIUS = WORLD_SIZE / 2n;
export const WORLD_MIN = -WORLD_RADIUS;
export const WORLD_MAX = WORLD_RADIUS;

export function isInsideWorld(x: number, y: number) {
  const worldPosition = getWorldPositionFromRenderPosition(x, y);

  return (
    worldPosition.x * worldPosition.x + worldPosition.y * worldPosition.y <=
    WORLD_RADIUS * WORLD_RADIUS
  );
}

export function getRenderWorldBounds() {
  const worldCenter = getRenderPosition({ x: 0n, y: 0n });
  const radius = Number(WORLD_RADIUS);

  return {
    centerX: worldCenter.x,
    centerY: worldCenter.y,
    minX: worldCenter.x - radius,
    maxX: worldCenter.x + radius,
    minY: worldCenter.y - radius,
    maxY: worldCenter.y + radius,
    radius,
  };
}

export function configureCamera(this: Scene) {
  const camera = this.cameras.main;
  camera.setBounds(
    Number(WORLD_MIN),
    Number(WORLD_MIN),
    Number(WORLD_SIZE),
    Number(WORLD_SIZE),
  );
  camera.setRoundPixels(true);
  camera.centerOn(0, 0);
  camera.setZoom(MAX_ZOOM);
  this.onZoomChange?.(camera.zoom);
}
