import Phaser from 'phaser';
import type {
  AsteroidDeposit,
  AsteroidDto,
  InventoryMaterial,
} from '@repo/types';

export type AsteroidMaterial = InventoryMaterial;

export type AsteroidMiningResult = {
  material: AsteroidMaterial;
  amount: number;
};

const MATERIAL_COLORS: Record<AsteroidMaterial, number> = {
  iron: 0xb6c2ce,
  silicates: 0xd6a76a,
  ice: 0x93c5fd,
  silver: 0xe5e7eb,
  carbon: 0x334155,
  gold: 0xfacc15,
  hydrogen: 0x67e8f9,
  nitrogen: 0x818cf8,
};
const MATERIAL_RARITY_WEIGHT: Record<AsteroidMaterial, number> = {
  silicates: 34,
  iron: 28,
  carbon: 16,
  ice: 12,
  hydrogen: 5,
  nitrogen: 3,
  silver: 1.4,
  gold: 0.3,
};
const OUTLINE_COLOR = 0x0f172a;
const ROCK_COLOR = 0x6b7280;
const MARKER_SCREEN_SIZE_PX = 12;

export class Asteroid extends Phaser.GameObjects.Container {
  readonly id: string;
  asteroid: AsteroidDto;
  readonly initialMassTonnes: number;
  readonly deposits: AsteroidDeposit[];
  private readonly initialRadius: number;
  private readonly rock: Phaser.GameObjects.Graphics;
  private readonly marker: Phaser.GameObjects.Rectangle;
  private readonly oreGlows: Phaser.GameObjects.Ellipse[] = [];
  private readonly spinSpeed: number;
  private currentRadius: number;

  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    radius: number,
    asteroid: AsteroidDto,
    deposits: AsteroidDeposit[],
  ) {
    super(scene, x, y);

    this.id = asteroid.id;
    this.asteroid = asteroid;
    this.initialRadius = radius;
    this.currentRadius = radius;
    this.initialMassTonnes = asteroid.massTonnes;
    this.deposits = deposits.map((deposit) => ({ ...deposit }));
    this.spinSpeed = Phaser.Math.FloatBetween(-0.45, 0.45);
    this.rock = new Phaser.GameObjects.Graphics(scene);
    this.marker = new Phaser.GameObjects.Rectangle(
      scene,
      0,
      0,
      MARKER_SCREEN_SIZE_PX,
      MARKER_SCREEN_SIZE_PX,
      MATERIAL_COLORS[this.getHighestRarityMaterial()],
      0.9,
    )
      .setStrokeStyle(1, OUTLINE_COLOR, 0.85)
      .setVisible(false);

    this.add([this.rock, this.marker]);
    this.drawRock();
    this.drawOreGlows(scene);
    this.setDepth(8);
    this.setSize(radius * 2, radius * 2);
    scene.add.existing(this);
  }

  get radius() {
    return this.currentRadius;
  }

  get remainingMassTonnes() {
    return this.deposits.reduce((total, deposit) => total + deposit.amount, 0);
  }

  update(deltaSeconds: number, zoom: number) {
    this.rotation += this.spinSpeed * deltaSeconds;
    this.updateMarkerMode(zoom);
  }

  syncPosition(x: number, y: number) {
    this.setPosition(x, y);
  }

  syncAsteroid(asteroid: AsteroidDto) {
    this.asteroid = asteroid;
  }

  containsScreenPoint(
    x: number,
    y: number,
    camera: Phaser.Cameras.Scene2D.Camera,
  ) {
    if (!this.visible) return false;

    const screenX = (this.x - camera.worldView.left) * camera.zoom;
    const screenY = (this.y - camera.worldView.top) * camera.zoom;
    const screenRadius = this.currentRadius * camera.zoom;
    const hitRadius = Math.max(MARKER_SCREEN_SIZE_PX / 2, screenRadius);

    return Phaser.Math.Distance.Between(x, y, screenX, screenY) <= hitRadius;
  }

  mine(amountTonnes: number): AsteroidMiningResult[] {
    const remainingMass = this.remainingMassTonnes;
    if (
      !Number.isFinite(amountTonnes) ||
      amountTonnes <= 0 ||
      remainingMass <= 0
    ) {
      return [];
    }

    const minedMass = Math.min(remainingMass, amountTonnes);
    const minedFraction = minedMass / remainingMass;
    const mined = this.deposits.map((deposit) => {
      const amount = Math.min(deposit.amount, deposit.amount * minedFraction);
      deposit.amount -= amount;
      return { material: deposit.material, amount };
    });

    this.deposits.forEach((deposit) => {
      if (deposit.amount < 0.0001) deposit.amount = 0;
    });
    this.refreshSize();
    return mined.filter((deposit) => deposit.amount > 0);
  }

  isDepleted() {
    return this.remainingMassTonnes <= 0.0001;
  }

  isPastViewport(viewport: Phaser.Geom.Rectangle, margin: number) {
    const paddedMargin = margin + this.currentRadius;
    return (
      this.x < viewport.left - paddedMargin ||
      this.x > viewport.right + paddedMargin ||
      this.y < viewport.top - paddedMargin ||
      this.y > viewport.bottom + paddedMargin
    );
  }

  shift(deltaX: number, deltaY: number) {
    this.x += deltaX;
    this.y += deltaY;
  }

  private refreshSize() {
    const massRatio = Phaser.Math.Clamp(
      this.remainingMassTonnes / this.initialMassTonnes,
      0,
      1,
    );
    const nextScale = Math.cbrt(massRatio);
    this.currentRadius = this.initialRadius * nextScale;
    this.setScale(nextScale);
    this.setSize(
      this.initialRadius * 2 * nextScale,
      this.initialRadius * 2 * nextScale,
    );

    const glowAlpha = Phaser.Math.Clamp(massRatio, 0.12, 0.68);
    this.oreGlows.forEach((glow) => glow.setAlpha(glowAlpha));
  }

  private updateMarkerMode(zoom: number) {
    const screenDiameter = this.currentRadius * 2 * zoom;
    const showMarker = screenDiameter < MARKER_SCREEN_SIZE_PX;
    this.rock.setVisible(!showMarker);
    this.oreGlows.forEach((glow) => glow.setVisible(!showMarker));
    this.marker.setVisible(showMarker);

    if (!showMarker) return;

    const scale = Math.max(this.scaleX, 0.0001);
    const markerSize = MARKER_SCREEN_SIZE_PX / (zoom * scale);
    this.marker
      .setSize(markerSize, markerSize)
      .setDisplaySize(markerSize, markerSize)
      .setStrokeStyle(1 / (zoom * scale), OUTLINE_COLOR, 0.85);
  }

  private getHighestRarityMaterial() {
    return this.deposits.reduce<AsteroidMaterial>(
      (rarest, deposit) =>
        MATERIAL_RARITY_WEIGHT[deposit.material] <
        MATERIAL_RARITY_WEIGHT[rarest]
          ? deposit.material
          : rarest,
      this.deposits[0]?.material ?? 'silicates',
    );
  }

  private drawRock() {
    const points = 10;
    const polygon: Phaser.Types.Math.Vector2Like[] = [];

    for (let index = 0; index < points; index += 1) {
      const angle = (Math.PI * 2 * index) / points;
      const pointRadius =
        this.initialRadius * Phaser.Math.FloatBetween(0.72, 1.08);
      polygon.push({
        x: Math.cos(angle) * pointRadius,
        y: Math.sin(angle) * pointRadius,
      });
    }

    this.rock
      .lineStyle(this.initialRadius * 0.035, OUTLINE_COLOR, 0.75)
      .fillStyle(ROCK_COLOR, 0.96)
      .beginPath()
      .moveTo(polygon[0].x, polygon[0].y);

    polygon.slice(1).forEach((point) => {
      this.rock.lineTo(point.x, point.y);
    });
    this.rock.closePath().fillPath().strokePath();

    this.rock.fillStyle(0x111827, 0.24);
    this.rock.fillCircle(
      -this.initialRadius * 0.28,
      this.initialRadius * 0.18,
      this.initialRadius * 0.11,
    );
    this.rock.fillCircle(
      this.initialRadius * 0.36,
      this.initialRadius * 0.3,
      this.initialRadius * 0.075,
    );
    this.rock.fillCircle(
      this.initialRadius * 0.1,
      -this.initialRadius * 0.34,
      this.initialRadius * 0.09,
    );
  }

  private drawOreGlows(scene: Phaser.Scene) {
    const offsets = [
      { x: 0.22, y: -0.18 },
      { x: -0.22, y: 0.12 },
      { x: 0.04, y: 0.28 },
    ];

    this.deposits.forEach((deposit, index) => {
      const offset = offsets[index] ?? offsets[0];
      const glow = new Phaser.GameObjects.Ellipse(
        scene,
        this.initialRadius * offset.x,
        this.initialRadius * offset.y,
        this.initialRadius * 0.58,
        this.initialRadius * 0.36,
        MATERIAL_COLORS[deposit.material],
        0.68,
      ).setBlendMode(Phaser.BlendModes.ADD);
      this.oreGlows.push(glow);
      this.add(glow);
    });
  }
}
