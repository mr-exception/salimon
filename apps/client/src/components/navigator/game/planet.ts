import Phaser from 'phaser';
import type { Planet as PlanetData } from '@types';
import { drawCelestialBody } from './draw-celestial-body';
import { getRenderPosition } from './get-render-position';
import { getPlanetPhysicsLabel } from './physics';

const PLANET_VARIANT_COLORS = [
  0x22c55e, // 0: green
  0x3b82f6, // 1: blue
  0xef4444, // 2: red
  0xf97316, // 3: orange
  0xa855f7, // 4: purple
  0x06b6d4, // 5: cyan
  0xec4899, // 6: pink
  0xeab308, // 7: yellow
  0x14b8a6, // 8: teal
  0x6366f1, // 9: indigo
  0x84cc16, // 10: lime
] as const;

const LABEL_SCREEN_GAP = 6;

export class Planet extends Phaser.GameObjects.Container {
  readonly planet: PlanetData;
  private readonly planetGraphics: Phaser.GameObjects.Graphics;
  private readonly rotationGraphics: Phaser.GameObjects.Graphics;
  private readonly label: Phaser.GameObjects.Text;
  private readonly physicsBody: MatterJS.BodyType;

  constructor(scene: Phaser.Scene, planet: PlanetData) {
    const position = getRenderPosition(planet.position);
    super(scene, Number(position.x), Number(position.y));

    this.planet = planet;
    this.planetGraphics = new Phaser.GameObjects.Graphics(scene);
    this.rotationGraphics = new Phaser.GameObjects.Graphics(scene);
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
    this.add([this.planetGraphics, this.rotationGraphics, this.label]);
    this.draw();
    this.rotationGraphics.setAngle(planet.rotationDegrees);
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
    this.rotationGraphics.setVisible(shapeVisible);
    if (shapeVisible) {
      const color =
        PLANET_VARIANT_COLORS[this.planet.variant] ?? PLANET_VARIANT_COLORS[0];
      drawCelestialBody(
        this.planetGraphics,
        color,
        radius,
        zoom,
        viewport,
        this.x,
        this.y,
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
    this.rotationGraphics.rotation =
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

  private draw() {
    const color =
      PLANET_VARIANT_COLORS[this.planet.variant] ?? PLANET_VARIANT_COLORS[0];

    this.planetGraphics
      .fillStyle(color)
      .fillCircle(0, 0, Number(this.planet.radius));
    this.rotationGraphics.fillStyle(0xffffff, 0.35);
    this.rotationGraphics.fillCircle(
      Number(this.planet.radius) * 0.45,
      0,
      Math.max(1, Number(this.planet.radius) * 0.08),
    );
  }
}
