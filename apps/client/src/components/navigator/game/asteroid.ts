import Phaser from 'phaser';

export type AsteroidMaterial = 'iron' | 'silicates' | 'ice';

export type AsteroidPayload = {
  material: AsteroidMaterial;
  amount: number;
};

const MATERIAL_COLORS: Record<AsteroidMaterial, number> = {
  iron: 0xb6c2ce,
  silicates: 0xd6a76a,
  ice: 0x93c5fd,
};
const OUTLINE_COLOR = 0x0f172a;

export class Asteroid extends Phaser.GameObjects.Container {
  readonly payload: AsteroidPayload;
  readonly radius: number;
  private readonly rock: Phaser.GameObjects.Graphics;
  private readonly oreGlow: Phaser.GameObjects.Ellipse;
  private readonly velocity: Phaser.Math.Vector2;
  private readonly spinSpeed: number;

  constructor(
    scene: Phaser.Scene,
    x: number,
    y: number,
    radius: number,
    payload: AsteroidPayload,
    velocity: Phaser.Math.Vector2,
  ) {
    super(scene, x, y);

    this.radius = radius;
    this.payload = payload;
    this.velocity = velocity;
    this.spinSpeed = Phaser.Math.FloatBetween(-0.9, 0.9);
    this.rock = new Phaser.GameObjects.Graphics(scene);
    this.oreGlow = new Phaser.GameObjects.Ellipse(
      scene,
      radius * 0.22,
      -radius * 0.18,
      radius * 0.58,
      radius * 0.36,
      MATERIAL_COLORS[payload.material],
      0.68,
    ).setBlendMode(Phaser.BlendModes.ADD);

    this.add([this.rock, this.oreGlow]);
    this.drawRock();
    this.setDepth(8);
    this.setSize(radius * 2, radius * 2);
    this.setInteractive(
      new Phaser.Geom.Circle(0, 0, radius * 1.2),
      Phaser.Geom.Circle.Contains,
    );
    scene.add.existing(this);
  }

  update(deltaSeconds: number, zoom: number) {
    this.x += this.velocity.x * deltaSeconds;
    this.y += this.velocity.y * deltaSeconds;
    this.rotation += this.spinSpeed * deltaSeconds;
    this.setScale(1 / zoom);
  }

  mine(amount: number) {
    if (!Number.isFinite(amount) || amount <= 0 || this.payload.amount <= 0) {
      return 0;
    }

    const minedAmount = Math.min(this.payload.amount, amount);
    this.payload.amount -= minedAmount;
    this.oreGlow.setAlpha(
      Phaser.Math.Clamp(
        this.payload.amount / Math.max(1, this.radius * 2),
        0.15,
        0.68,
      ),
    );
    return minedAmount;
  }

  isDepleted() {
    return this.payload.amount <= 0;
  }

  isPastViewport(viewport: Phaser.Geom.Rectangle, margin: number) {
    return (
      this.x < viewport.left - margin ||
      this.x > viewport.right + margin ||
      this.y < viewport.top - margin ||
      this.y > viewport.bottom + margin
    );
  }

  private drawRock() {
    const points = 9;
    const polygon: Phaser.Types.Math.Vector2Like[] = [];

    for (let index = 0; index < points; index += 1) {
      const angle = (Math.PI * 2 * index) / points;
      const pointRadius = this.radius * Phaser.Math.FloatBetween(0.72, 1.08);
      polygon.push({
        x: Math.cos(angle) * pointRadius,
        y: Math.sin(angle) * pointRadius,
      });
    }

    this.rock
      .lineStyle(2, OUTLINE_COLOR, 0.75)
      .fillStyle(0x6b7280, 0.96)
      .beginPath()
      .moveTo(polygon[0].x, polygon[0].y);

    polygon.slice(1).forEach((point) => {
      this.rock.lineTo(point.x, point.y);
    });
    this.rock.closePath().fillPath().strokePath();

    this.rock.fillStyle(0x111827, 0.24);
    this.rock.fillCircle(-this.radius * 0.28, this.radius * 0.18, 3.5);
    this.rock.fillCircle(this.radius * 0.36, this.radius * 0.3, 2.4);
    this.rock.fillCircle(this.radius * 0.1, -this.radius * 0.34, 2.8);
  }
}
