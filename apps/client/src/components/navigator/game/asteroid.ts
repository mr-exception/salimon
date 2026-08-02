import Phaser from 'phaser';
import type { Position } from '@repo/types';
import type { Asteroid as AsteroidData } from '@store';
import {
  getRenderPosition,
  getRenderPositionFromOrigin,
} from './get-render-position';

const ASTEROID_COLOR = 0x94a3b8;
const ASTEROID_STROKE_COLOR = 0xe2e8f0;
const MIN_DETAILED_SCREEN_WIDTH_PX = 10;
const SIMPLE_ASTEROID_SCREEN_RADIUS_PX = 3;
const MIN_HIT_RADIUS_PX = 8;
const SCALE_REFERENCE_WIDTH_PX = 200;
const MIN_ASTEROID_SCALE_DISTANCE_METERS = 35_000_000;
const MIN_ASTEROID_RENDER_ZOOM =
  SCALE_REFERENCE_WIDTH_PX / MIN_ASTEROID_SCALE_DISTANCE_METERS;

export class Asteroid extends Phaser.GameObjects.Container {
  readonly asteroid: AsteroidData;
  private readonly graphics: Phaser.GameObjects.Graphics;
  private readonly shapePoints: Phaser.Math.Vector2[];

  constructor(scene: Phaser.Scene, asteroid: AsteroidData) {
    const position = getRenderPosition(asteroid.position);
    super(scene, Number(position.x), Number(position.y));

    this.asteroid = asteroid;
    this.graphics = new Phaser.GameObjects.Graphics(scene);
    this.shapePoints = createAsteroidShapePoints(asteroid.id);

    this.setName(asteroid.name);
    this.setDepth(2);
    this.add(this.graphics);
    scene.add.existing(this);
  }

  setRenderVisibility(
    zoom: number,
    viewport: Phaser.Geom.Rectangle,
    parentShapeVisible: boolean,
    showAsteroids: boolean,
  ) {
    const radius = Number(this.asteroid.radius);
    const intersectsViewport =
      this.x + radius >= viewport.left &&
      this.x - radius <= viewport.right &&
      this.y + radius >= viewport.top &&
      this.y - radius <= viewport.bottom;
    const visible =
      showAsteroids &&
      zoom >= MIN_ASTEROID_RENDER_ZOOM &&
      parentShapeVisible &&
      intersectsViewport;

    this.setVisible(visible);
    if (!visible) return;

    this.draw(zoom);
  }

  syncPosition(originPosition?: Position) {
    const position = originPosition
      ? getRenderPositionFromOrigin(this.asteroid.position, originPosition)
      : getRenderPosition(this.asteroid.position);
    this.setPosition(
      Number(position.x) + (this.asteroid.positionRemainder?.x ?? 0),
      Number(position.y) + (this.asteroid.positionRemainder?.y ?? 0),
    );
  }

  containsScreenPoint(
    x: number,
    y: number,
    camera: Phaser.Cameras.Scene2D.Camera,
  ) {
    if (!this.visible) return false;

    const screenX = (this.x - camera.worldView.left) * camera.zoom;
    const screenY = (this.y - camera.worldView.top) * camera.zoom;
    const hitRadius = Math.max(MIN_HIT_RADIUS_PX, this.getScreenRadius(camera.zoom));

    return Phaser.Math.Distance.Between(x, y, screenX, screenY) <= hitRadius;
  }

  private draw(zoom: number) {
    const radius = Number(this.asteroid.radius);
    const screenRadius = this.getScreenRadius(zoom);
    const screenWidth = screenRadius * 2;
    this.graphics.clear();

    if (screenWidth < MIN_DETAILED_SCREEN_WIDTH_PX) {
      this.graphics
        .fillStyle(ASTEROID_COLOR, 0.95)
        .fillCircle(0, 0, screenRadius / zoom);
      return;
    }

    this.graphics
      .fillStyle(ASTEROID_COLOR, 0.82)
      .lineStyle(1 / zoom, ASTEROID_STROKE_COLOR, 0.42)
      .beginPath();
    this.shapePoints.forEach((point, index) => {
      const x = point.x * radius;
      const y = point.y * radius;
      if (index === 0) {
        this.graphics.moveTo(x, y);
      } else {
        this.graphics.lineTo(x, y);
      }
    });
    this.graphics.closePath().fillPath().strokePath();
  }

  private getScreenRadius(zoom: number) {
    const radius = Number(this.asteroid.radius);
    const screenRadius = radius * zoom;

    return screenRadius * 2 < MIN_DETAILED_SCREEN_WIDTH_PX
      ? SIMPLE_ASTEROID_SCREEN_RADIUS_PX
      : screenRadius;
  }
}

function createAsteroidShapePoints(seed: string) {
  const random = createSeededRandom(seed);
  const points: Phaser.Math.Vector2[] = [];
  const segmentCount = 9;

  for (let index = 0; index < segmentCount; index += 1) {
    const angle = (index / segmentCount) * Math.PI * 2;
    const radius = 0.68 + random() * 0.42;
    points.push(
      new Phaser.Math.Vector2(
        Math.cos(angle) * radius,
        Math.sin(angle) * radius,
      ),
    );
  }

  return points;
}

function createSeededRandom(seed: string) {
  let value = 2166136261;
  for (let index = 0; index < seed.length; index += 1) {
    value ^= seed.charCodeAt(index);
    value = Math.imul(value, 16777619);
  }

  return () => {
    value += 0x6d2b79f5;
    let result = value;
    result = Math.imul(result ^ (result >>> 15), result | 1);
    result ^= result + Math.imul(result ^ (result >>> 7), result | 61);
    return ((result ^ (result >>> 14)) >>> 0) / 4294967296;
  };
}
