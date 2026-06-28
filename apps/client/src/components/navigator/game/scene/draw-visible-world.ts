import Phaser from 'phaser';
import type { Scene } from '.';
import { getRenderWorldBounds } from './configure-camera';

const BASE_GRID_SPACING = 200000;
const MIN_GRID_SCREEN_SPACING = 40;
const WORLD_BORDER_COLOR = 0x1e3a8a;

export function drawVisibleWorld(this: Scene) {
  const graphics = this.grid;
  if (!graphics) return;

  const camera = this.cameras.main;
  camera.preRender();

  const view = camera.worldView;
  const viewportKey = [
    view.x,
    view.y,
    view.width,
    view.height,
    camera.zoom,
  ].join(':');
  if (viewportKey === this.lastViewportKey) return;

  this.lastViewportKey = viewportKey;
  graphics.clear();

  const worldBounds = getRenderWorldBounds();
  const left = Math.max(view.left, worldBounds.minX);
  const right = Math.min(view.right, worldBounds.maxX);
  const top = Math.max(view.top, worldBounds.minY);
  const bottom = Math.min(view.bottom, worldBounds.maxY);

  drawGrid(
    graphics,
    left,
    right,
    top,
    bottom,
    worldBounds.centerX,
    worldBounds.centerY,
    worldBounds.radius,
    camera.zoom,
  );
  drawWorldBorder(
    graphics,
    worldBounds.centerX,
    worldBounds.centerY,
    worldBounds.radius,
    camera.zoom,
  );
}

function drawGrid(
  graphics: Phaser.GameObjects.Graphics,
  left: number,
  right: number,
  top: number,
  bottom: number,
  centerX: number,
  centerY: number,
  radius: number,
  zoom: number,
) {
  const spacing = getAdaptiveSpacing(
    BASE_GRID_SPACING,
    MIN_GRID_SCREEN_SPACING,
    zoom,
  );

  graphics.lineStyle(1 / zoom, 0x172554, 0.22);
  for (let x = Math.ceil(left / spacing) * spacing; x <= right; x += spacing) {
    const yExtent = Math.sqrt(Math.max(0, radius ** 2 - (x - centerX) ** 2));
    const lineTop = Math.max(top, centerY - yExtent);
    const lineBottom = Math.min(bottom, centerY + yExtent);
    if (lineTop <= lineBottom) {
      graphics.lineBetween(x, lineTop, x, lineBottom);
    }
  }
  for (let y = Math.ceil(top / spacing) * spacing; y <= bottom; y += spacing) {
    const xExtent = Math.sqrt(Math.max(0, radius ** 2 - (y - centerY) ** 2));
    const lineLeft = Math.max(left, centerX - xExtent);
    const lineRight = Math.min(right, centerX + xExtent);
    if (lineLeft <= lineRight) {
      graphics.lineBetween(lineLeft, y, lineRight, y);
    }
  }
}

function drawWorldBorder(
  graphics: Phaser.GameObjects.Graphics,
  centerX: number,
  centerY: number,
  radius: number,
  zoom: number,
) {
  graphics.lineStyle(2 / zoom, WORLD_BORDER_COLOR, 0.9);
  graphics.strokeCircle(centerX, centerY, radius);
}

function getAdaptiveSpacing(base: number, minimumPixels: number, zoom: number) {
  let spacing = base;

  while (spacing * zoom < minimumPixels) spacing *= 2;

  return spacing;
}
