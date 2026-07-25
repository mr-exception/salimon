import Phaser from 'phaser';
import type { Scene } from '.';
import { getRenderWorldBounds } from './configure-camera';

const BASE_GRID_SPACING = 200000;
const MIN_GRID_SCREEN_SPACING = 40;
const MIN_WORLD_SPACE_GRID_ZOOM = 0.0000000001;
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
  if (camera.zoom < MIN_WORLD_SPACE_GRID_ZOOM) {
    drawScreenSpaceWorld(graphics, view, worldBounds, camera.zoom);
    return;
  }

  graphics.setScrollFactor(1);
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

function drawScreenSpaceWorld(
  graphics: Phaser.GameObjects.Graphics,
  view: Phaser.Geom.Rectangle,
  worldBounds: ReturnType<typeof getRenderWorldBounds>,
  zoom: number,
) {
  const spacing = getAdaptiveSpacing(
    BASE_GRID_SPACING,
    MIN_GRID_SCREEN_SPACING,
    zoom,
  );
  const screenSpacing = spacing * zoom;

  graphics.setScrollFactor(0);
  graphics.lineStyle(1, 0x172554, 0.22);

  for (
    let x = (Math.ceil(view.left / spacing) * spacing - view.left) * zoom;
    x <= view.width * zoom;
    x += screenSpacing
  ) {
    graphics.lineBetween(x, 0, x, view.height * zoom);
  }
  for (
    let y = (Math.ceil(view.top / spacing) * spacing - view.top) * zoom;
    y <= view.height * zoom;
    y += screenSpacing
  ) {
    graphics.lineBetween(0, y, view.width * zoom, y);
  }

  const centerX = (worldBounds.centerX - view.left) * zoom;
  const centerY = (worldBounds.centerY - view.top) * zoom;
  const radius = worldBounds.radius * zoom;
  if (Number.isFinite(radius) && radius <= 10_000_000) {
    graphics.lineStyle(2, WORLD_BORDER_COLOR, 0.9);
    graphics.strokeCircle(centerX, centerY, radius);
  }
}

function getAdaptiveSpacing(base: number, minimumPixels: number, zoom: number) {
  let spacing = base;

  while (spacing * zoom < minimumPixels) spacing *= 2;

  return spacing;
}
