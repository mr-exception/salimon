import Phaser from 'phaser';
import { getSpaceshipMotionState } from '@store';
import type { Spaceship as SpaceshipData } from '@repo/types';
import { getRenderPosition } from './get-render-position';
import { SPACESHIP_PHYSICS_LABEL } from './physics';

const SPACESHIP_LENGTH_M = 400;
const SPACESHIP_WIDTH_M = 400;
const MIN_SPACESHIP_LENGTH_PX = 16;
const DIRECTION_ARROW_START_M = 280;
const DIRECTION_ARROW_TIP_M = 1_080;
const DIRECTION_ARROW_LENGTH_PX = 120;
const TARGET_ARROW_COLOR = 0x22c55e;
const THRUSTER_GLOW_COLOR = 0x22d3ee;
const THRUSTER_OFFSET_M = 224;

type Vector = { x: number; y: number };

export const SPACESHIP_TEXTURE_KEY = 'spaceship';

export class Spaceship extends Phaser.GameObjects.Container {
  readonly spaceship: SpaceshipData;
  private readonly shipImage: Phaser.GameObjects.Image;
  private readonly thrusterGlows: Phaser.GameObjects.Ellipse[];
  private readonly targetArrow: Phaser.GameObjects.Graphics;
  private readonly physicsBody: MatterJS.BodyType;
  private targetDirection?: number;
  private thrustVector?: Vector;
  private thrusterPulse?: Phaser.Tweens.Tween;
  private thrustersActive = false;
  private worldScale = 1;

  constructor(scene: Phaser.Scene, spaceship: SpaceshipData) {
    const renderPosition = getRenderPosition(spaceship.position);
    super(scene, Number(renderPosition.x), Number(renderPosition.y));

    this.spaceship = spaceship;
    this.shipImage = new Phaser.GameObjects.Image(
      scene,
      0,
      0,
      SPACESHIP_TEXTURE_KEY,
    );
    this.shipImage.setDisplaySize(SPACESHIP_WIDTH_M, SPACESHIP_LENGTH_M);
    this.thrusterGlows = [
      createThrusterGlow(scene, 0, -THRUSTER_OFFSET_M, 112, 88),
      createThrusterGlow(scene, THRUSTER_OFFSET_M, 0, 88, 112),
      createThrusterGlow(scene, 0, THRUSTER_OFFSET_M, 112, 88),
      createThrusterGlow(scene, -THRUSTER_OFFSET_M, 0, 88, 112),
    ];
    this.targetArrow = createDirectionArrow(scene, TARGET_ARROW_COLOR, 0.72);
    this.targetArrow.setVisible(false);

    this.setName(spaceship.name);
    this.add([this.targetArrow, ...this.thrusterGlows, this.shipImage]);
    scene.add.existing(this);
    this.physicsBody = scene.matter.add.circle(
      this.x,
      this.y,
      Number(spaceship.radius),
      {
        label: SPACESHIP_PHYSICS_LABEL,
        ignoreGravity: true,
        mass: Number(spaceship.mass),
        restitution: 0,
        friction: 1,
        frictionStatic: 1,
      },
    );
    this.once(Phaser.GameObjects.Events.DESTROY, () => {
      scene.matter.world.remove(this.physicsBody);
    });
  }

  setRenderVisibility(zoom: number, viewport: Phaser.Geom.Rectangle) {
    const minimumWorldScale =
      MIN_SPACESHIP_LENGTH_PX / SPACESHIP_LENGTH_M / zoom;
    const worldScale = Math.max(1, minimumWorldScale);
    this.worldScale = worldScale;
    this.setScale(worldScale);
    this.syncRenderPosition();
    const arrowScale =
      DIRECTION_ARROW_LENGTH_PX /
      (DIRECTION_ARROW_TIP_M - DIRECTION_ARROW_START_M) /
      worldScale /
      zoom;
    this.targetArrow.setScale(arrowScale);
    this.setVisible(this.intersectsViewport(viewport));
  }

  syncPosition() {
    this.syncRenderPosition();
    this.scene.matter.body.setPosition(this.physicsBody, {
      x: this.x,
      y: this.y,
    });
    this.scene.matter.body.setVelocity(this.physicsBody, { x: 0, y: 0 });
    this.syncDirectionIndicators();
    this.syncThrusterGlows();
  }

  snapToSurface(reference: Vector, referenceRadius: number) {
    if (getSpaceshipMotionState() === 'flying') return;

    const offsetX = this.x - reference.x;
    const offsetY = this.y - reference.y;
    const distance = Math.hypot(offsetX, offsetY);
    if (distance === 0) return;

    const contactDistance =
      referenceRadius + Number(this.spaceship.radius) * this.worldScale;
    this.setPosition(
      reference.x + (offsetX / distance) * contactDistance,
      reference.y + (offsetY / distance) * contactDistance,
    );
    this.scene.matter.body.setPosition(this.physicsBody, {
      x: this.x,
      y: this.y,
    });
  }

  setThrustersActive(active: boolean, thrustVector?: Vector) {
    if (active && this.thrustersActive && this.thrusterPulse) {
      this.thrustVector = thrustVector;
      this.syncThrusterGlows();
      return;
    }

    this.thrusterPulse?.stop();
    this.thrusterPulse = undefined;
    this.thrustersActive = active;
    this.thrustVector = active ? thrustVector : undefined;
    this.thrusterGlows.forEach((glow) => glow.setScale(1).setAlpha(0.85));
    this.syncThrusterGlows();
    if (!active || !thrustVector) return;

    this.thrusterPulse = this.scene.tweens.add({
      targets: this.thrusterGlows,
      scale: 1.2,
      alpha: 0.45,
      duration: 120,
      yoyo: true,
      repeat: -1,
    });
  }

  setTargetDirection(direction: number) {
    this.targetDirection = direction;
    this.syncDirectionIndicators();
  }

  clearTargetDirection() {
    this.targetDirection = undefined;
    this.targetArrow.setVisible(false);
  }

  private syncRenderPosition() {
    const renderPosition = getRenderPosition(this.spaceship.position);
    const referenceName = this.spaceship.position.relativeTo;
    if (
      getSpaceshipMotionState() === 'flying' ||
      !referenceName ||
      this.worldScale <= 1
    ) {
      this.setPosition(Number(renderPosition.x), Number(renderPosition.y));
      return;
    }

    const referencePosition = getRenderPosition({
      x: 0n,
      y: 0n,
      relativeTo: referenceName,
    });
    const offsetX = Number(renderPosition.x - referencePosition.x);
    const offsetY = Number(renderPosition.y - referencePosition.y);
    const distance = Math.hypot(offsetX, offsetY);
    if (distance === 0) {
      this.setPosition(Number(renderPosition.x), Number(renderPosition.y));
      return;
    }

    const visualRadiusOffset =
      Number(this.spaceship.radius) * (this.worldScale - 1);
    this.setPosition(
      Number(renderPosition.x) + (offsetX / distance) * visualRadiusOffset,
      Number(renderPosition.y) + (offsetY / distance) * visualRadiusOffset,
    );
  }

  intersectsViewport(viewport: Phaser.Geom.Rectangle) {
    const radius = Math.max(
      Number(this.spaceship.radius) * this.worldScale,
      (SPACESHIP_LENGTH_M * this.worldScale) / 2,
    );

    return (
      this.x + radius >= viewport.left &&
      this.x - radius <= viewport.right &&
      this.y + radius >= viewport.top &&
      this.y - radius <= viewport.bottom
    );
  }

  private syncDirectionIndicators() {
    this.syncTargetDirection();
  }

  private syncTargetDirection() {
    if (this.targetDirection === undefined) return;

    this.targetArrow.setRotation(this.targetDirection).setVisible(true);
  }

  private syncThrusterGlows() {
    const thrustVector = this.thrustVector;
    if (!thrustVector || Math.hypot(thrustVector.x, thrustVector.y) === 0) {
      this.thrusterGlows.forEach((glow) => glow.setVisible(false));
      return;
    }

    const exhaust = {
      x: -thrustVector.x,
      y: -thrustVector.y,
    };
    const activeThrusters = new Set<number>();
    if (Math.abs(exhaust.x) > 1e-8) {
      activeThrusters.add(exhaust.x > 0 ? 1 : 3);
    }
    if (Math.abs(exhaust.y) > 1e-8) {
      activeThrusters.add(exhaust.y > 0 ? 2 : 0);
    }

    this.thrusterGlows.forEach((glow, index) =>
      glow.setVisible(activeThrusters.has(index)),
    );
  }
}

function createDirectionArrow(
  scene: Phaser.Scene,
  color: number,
  alpha: number,
) {
  return new Phaser.GameObjects.Graphics(scene)
    .lineStyle(11, color, alpha)
    .beginPath()
    .moveTo(DIRECTION_ARROW_START_M, 0)
    .lineTo(DIRECTION_ARROW_TIP_M, 0)
    .lineTo(DIRECTION_ARROW_TIP_M - 132, -84)
    .moveTo(DIRECTION_ARROW_TIP_M, 0)
    .lineTo(DIRECTION_ARROW_TIP_M - 132, 84)
    .strokePath();
}

function createThrusterGlow(
  scene: Phaser.Scene,
  x: number,
  y: number,
  width: number,
  height: number,
) {
  return new Phaser.GameObjects.Ellipse(
    scene,
    x,
    y,
    width,
    height,
    THRUSTER_GLOW_COLOR,
    0.85,
  )
    .setBlendMode(Phaser.BlendModes.ADD)
    .setVisible(false);
}
