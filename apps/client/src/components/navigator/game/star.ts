import Phaser from 'phaser';
import type { Star as StarData } from '@types';
import { getRenderPosition } from './get-render-position';

const LABEL_SCREEN_GAP = 6;
const GLOW_SCREEN_RADIUS = 7;
export const STAR_PATTERN_TEXTURE_SIZE = 1_024;
export const STAR_PATTERN_VARIANT_COUNT = 4;

export function getStarPatternTextureKey(variant: number) {
  const supportedVariant =
    Number.isInteger(variant) &&
    variant >= 0 &&
    variant < STAR_PATTERN_VARIANT_COUNT
      ? variant
      : 0;
  return `star-pattern-${supportedVariant}`;
}

export class Star extends Phaser.GameObjects.Container {
  readonly star: StarData;
  private readonly glowGraphics: Phaser.GameObjects.Graphics;
  private readonly starGraphics: Phaser.GameObjects.Graphics;
  private readonly patternImage: Phaser.GameObjects.Image;
  private readonly label: Phaser.GameObjects.Text;

  constructor(scene: Phaser.Scene, star: StarData) {
    const position = getRenderPosition(star.position);
    super(scene, Number(position.x), Number(position.y));

    this.star = star;
    this.glowGraphics = new Phaser.GameObjects.Graphics(scene);
    this.starGraphics = new Phaser.GameObjects.Graphics(scene);
    this.patternImage = new Phaser.GameObjects.Image(
      scene,
      0,
      0,
      getStarPatternTextureKey(star.variant),
    )
      .setDisplaySize(Number(star.radius) * 2, Number(star.radius) * 2)
      .setTint(star.color);
    this.label = new Phaser.GameObjects.Text(scene, 0, 0, star.name, {
      color: '#fef3c7',
      fontFamily: 'system-ui, sans-serif',
      fontSize: '14px',
      fontStyle: 'bold',
      stroke: '#050816',
      strokeThickness: 3,
    })
      .setOrigin(0.5, 1)
      .setResolution(Math.max(2, window.devicePixelRatio));
    this.label.texture.setFilter(Phaser.Textures.FilterMode.NEAREST);

    this.setName(star.name);
    this.add([
      this.glowGraphics,
      this.starGraphics,
      this.patternImage,
      this.label,
    ]);
    this.draw();
    this.patternImage.setAngle(star.rotationDegrees);
    scene.add.existing(this);
  }

  setRenderVisibility(
    zoom: number,
    viewport: Phaser.Geom.Rectangle,
    alwaysVisible = false,
    showViewportLabel = false,
  ) {
    const { x, y } = getRenderPosition(this.star.position);
    const radius = Number(this.star.radius);
    const intersectsViewport =
      x + radius >= viewport.left &&
      x - radius <= viewport.right &&
      y + radius >= viewport.top &&
      y - radius <= viewport.bottom;
    const shapeVisible =
      zoom >= this.star.shapeRenderZoomLevel && intersectsViewport;
    const glowVisible =
      zoom < this.star.shapeRenderZoomLevel && intersectsViewport;

    const bodyVisible =
      intersectsViewport && (glowVisible || shapeVisible || alwaysVisible);

    this.setVisible(bodyVisible);
    if (!bodyVisible) return;

    this.glowGraphics.setVisible(glowVisible);
    this.glowGraphics.setScale(1 / zoom);
    this.starGraphics.setVisible(shapeVisible);
    this.patternImage.setVisible(shapeVisible);
    this.label.setVisible(
      alwaysVisible || showViewportLabel || zoom >= this.star.renderZoomLevel,
    );
    this.label.setScale(1 / zoom);
    this.label.setY(-radius - LABEL_SCREEN_GAP / zoom);
  }

  syncPosition() {
    const position = getRenderPosition(this.star.position);
    this.setPosition(Number(position.x), Number(position.y));
  }

  syncRotation(elapsedSeconds: number) {
    this.patternImage.rotation =
      ((this.star.rotationDegrees * Math.PI) / 180 +
        (Math.PI * 2 * elapsedSeconds) / this.star.rotationPeriodSeconds) %
      (Math.PI * 2);
  }

  containsScreenPoint(
    x: number,
    y: number,
    camera: Phaser.Cameras.Scene2D.Camera,
  ) {
    if (!this.visible) return false;

    const screenX = (this.x - camera.worldView.left) * camera.zoom;
    const screenY = (this.y - camera.worldView.top) * camera.zoom;
    const screenRadius = Number(this.star.radius) * camera.zoom;
    const hitRadius = Math.max(8, screenRadius);
    const hitsGlow =
      this.glowGraphics.visible &&
      Phaser.Math.Distance.Between(x, y, screenX, screenY) <=
        GLOW_SCREEN_RADIUS;
    const hitsShape =
      this.starGraphics.visible &&
      Phaser.Math.Distance.Between(x, y, screenX, screenY) <= hitRadius;
    const labelBottom = screenY - screenRadius - LABEL_SCREEN_GAP;
    const hitsLabel =
      this.label.visible &&
      x >= screenX - this.label.width / 2 &&
      x <= screenX + this.label.width / 2 &&
      y >= labelBottom - this.label.height &&
      y <= labelBottom;

    return hitsGlow || hitsShape || hitsLabel;
  }

  private draw() {
    this.glowGraphics.fillStyle(this.star.color, 0.12);
    this.glowGraphics.fillCircle(0, 0, GLOW_SCREEN_RADIUS);
    this.glowGraphics.fillStyle(this.star.color, 0.3);
    this.glowGraphics.fillCircle(0, 0, 4);
    this.glowGraphics.fillStyle(this.star.color, 0.9);
    this.glowGraphics.fillCircle(0, 0, 1.5);

    this.starGraphics.fillStyle(this.star.color, 0.3);
    this.starGraphics.fillCircle(0, 0, Number(this.star.radius));
  }
}
