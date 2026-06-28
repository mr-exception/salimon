import Phaser from 'phaser';
import type { Planet as PlanetData } from '@types';
import { drawCelestialBody } from './draw-celestial-body';
import { getRenderPosition } from './get-render-position';
import { getPlanetPhysicsLabel } from './physics';

const LABEL_SCREEN_GAP = 6;
export const PLANET_PATTERN_VARIANT_COUNT = 10;
export const PLANET_PATTERN_TEXTURE_SIZE = 1_024;

export function getPlanetPatternTextureKey(variant: number) {
  const supportedVariant =
    Number.isInteger(variant) &&
    variant >= 0 &&
    variant < PLANET_PATTERN_VARIANT_COUNT
      ? variant
      : 0;
  return `planet-pattern-${supportedVariant}`;
}

export class Planet extends Phaser.GameObjects.Container {
  readonly planet: PlanetData;
  private readonly planetGraphics: Phaser.GameObjects.Graphics;
  private readonly patternImage: Phaser.GameObjects.Image;
  private readonly label: Phaser.GameObjects.Text;
  private readonly physicsBody: MatterJS.BodyType;

  constructor(scene: Phaser.Scene, planet: PlanetData) {
    const position = getRenderPosition(planet.position);
    super(scene, Number(position.x), Number(position.y));

    this.planet = planet;
    this.planetGraphics = new Phaser.GameObjects.Graphics(scene);
    this.patternImage = new Phaser.GameObjects.Image(
      scene,
      0,
      0,
      getPlanetPatternTextureKey(planet.variant),
    )
      .setDisplaySize(Number(planet.radius) * 2, Number(planet.radius) * 2)
      .setTint(planet.color);
    this.label = new Phaser.GameObjects.Text(scene, 0, 0, planet.name, {
      color: '#e2e8f0',
      fontFamily: 'system-ui, sans-serif',
      fontSize: '14px',
      fontStyle: 'bold',
      stroke: '#050816',
      strokeThickness: 3,
    })
      .setOrigin(0.5, 1)
      .setResolution(Math.max(2, window.devicePixelRatio));
    this.label.texture.setFilter(Phaser.Textures.FilterMode.NEAREST);

    this.setName(planet.name);
    this.add([this.planetGraphics, this.patternImage, this.label]);
    this.patternImage.setAngle(planet.rotationDegrees);
    scene.add.existing(this);
    this.physicsBody = scene.matter.add.circle(
      this.x,
      this.y,
      Number(planet.radius),
      {
        label: getPlanetPhysicsLabel(planet.name),
        isStatic: true,
        ignoreGravity: true,
        restitution: 0,
        friction: 1,
        frictionStatic: 1,
      },
    );
    this.once(Phaser.GameObjects.Events.DESTROY, () => {
      scene.matter.world.remove(this.physicsBody);
    });
  }

  setRenderVisibility(
    zoom: number,
    viewport: Phaser.Geom.Rectangle,
    alwaysVisible = false,
    showViewportLabel = false,
  ) {
    const { x, y } = getRenderPosition(this.planet.position);
    const radius = Number(this.planet.radius);
    const intersectsViewport =
      x + radius >= viewport.left &&
      x - radius <= viewport.right &&
      y + radius >= viewport.top &&
      y - radius <= viewport.bottom;

    const shapeVisible =
      zoom >= this.planet.shapeRenderZoomLevel && intersectsViewport;

    const bodyVisible =
      intersectsViewport &&
      (shapeVisible || alwaysVisible || showViewportLabel);

    this.setVisible(bodyVisible);
    if (!bodyVisible) return;

    this.planetGraphics.setVisible(shapeVisible);
    this.patternImage.setVisible(shapeVisible);
    if (shapeVisible) {
      drawCelestialBody(
        this.planetGraphics,
        this.planet.color,
        radius,
        zoom,
        viewport,
        this.x,
        this.y,
        0.3,
      );
    }
    this.label.setVisible(
      alwaysVisible || showViewportLabel || zoom >= this.planet.renderZoomLevel,
    );
    this.label.setScale(1 / zoom);
    this.label.setY(-radius - LABEL_SCREEN_GAP / zoom);
  }

  syncPosition() {
    const position = getRenderPosition(this.planet.position);
    this.setPosition(Number(position.x), Number(position.y));
    this.scene.matter.body.setPosition(this.physicsBody, {
      x: this.x,
      y: this.y,
    });
  }

  syncRotation(elapsedSeconds: number) {
    this.patternImage.rotation =
      ((this.planet.rotationDegrees * Math.PI) / 180 +
        (Math.PI * 2 * elapsedSeconds) / this.planet.rotationPeriodSeconds) %
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
    const screenRadius = Number(this.planet.radius) * camera.zoom;
    const hitRadius = Math.max(8, screenRadius);
    const hitsShape =
      this.planetGraphics.visible &&
      Phaser.Math.Distance.Between(x, y, screenX, screenY) <= hitRadius;
    const labelBottom = screenY - screenRadius - LABEL_SCREEN_GAP;
    const hitsLabel =
      this.label.visible &&
      x >= screenX - this.label.width / 2 &&
      x <= screenX + this.label.width / 2 &&
      y >= labelBottom - this.label.height &&
      y <= labelBottom;

    return hitsShape || hitsLabel;
  }
}
