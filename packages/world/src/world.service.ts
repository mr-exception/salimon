export const FULL_ROTATION_RADIANS = Math.PI * 2;
export const GRAVITATIONAL_CONSTANT = 6.6743e-11;
export const MAX_PROPAGATION_STEPS = 20_000;
export const TARGET_STEP_SECONDS = 30;
export const SPACESHIP_MASS_KG = 10_000;
export const MAX_ENGINE_THRUST_KN = 1_000;
export const CRASH_SPEED_METERS_PER_SECOND = 15;

export type NumericValue = bigint | number | string;

export type Vector = {
  x: number;
  y: number;
};

export type SerializedPosition = {
  x: string;
  y: string;
  relativeTo?: string;
  relativeToId?: string;
};

export type Motion = {
  position: Vector;
  velocity: Vector;
};

export type WorldBodyLike = {
  name: string;
  position: {
    x: NumericValue;
    y: NumericValue;
    relativeTo?: string;
    relativeToId?: string;
  };
  mass: NumericValue;
  radius?: NumericValue;
  orbitalCenter: string | null;
  clockwise: boolean;
  speed: NumericValue;
  updatedAt?: Date;
  rotationPeriodSeconds?: number;
};

export type WorldSnapshotLike<TBody extends WorldBodyLike> = {
  bodies: readonly TBody[];
  bodiesByName: ReadonlyMap<string, TBody>;
};

export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';

export class WorldService {
  static add(value: Vector, change: Vector, scale = 1): Vector {
    return {
      x: value.x + change.x * scale,
      y: value.y + change.y * scale,
    };
  }

  static toNumber(value: NumericValue) {
    return Number(value);
  }

  static advanceBodyPosition(
    body: Pick<
      WorldBodyLike,
      'position' | 'orbitalCenter' | 'clockwise' | 'speed'
    >,
    elapsedSeconds: number,
  ): SerializedPosition {
    const x = WorldService.toNumber(body.position.x);
    const y = WorldService.toNumber(body.position.y);
    const orbitalRadius = Math.hypot(x, y);
    const speed = WorldService.toNumber(body.speed);
    const orbitalCenter = body.orbitalCenter ?? body.position.relativeTo;

    if (
      !orbitalCenter ||
      orbitalRadius === 0 ||
      speed === 0 ||
      elapsedSeconds <= 0
    ) {
      return WorldService.serializePosition(body.position);
    }

    const direction = body.clockwise ? 1 : -1;
    const angle =
      ((direction * speed * elapsedSeconds) / orbitalRadius) %
      FULL_ROTATION_RADIANS;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);

    return {
      x: Math.round(x * cos - y * sin).toString(),
      y: Math.round(x * sin + y * cos).toString(),
      ...(body.position.relativeTo
        ? { relativeTo: body.position.relativeTo }
        : {}),
      ...(body.position.relativeToId
        ? { relativeToId: body.position.relativeToId }
        : {}),
    };
  }

  static getBodyPositions<TBody extends WorldBodyLike>(
    world: WorldSnapshotLike<TBody>,
    time: Date,
  ) {
    const positions = new Map<string, Vector>();

    const resolve = (body: TBody, path: Set<string>): Vector => {
      const cached = positions.get(body.name);
      if (cached) return cached;
      if (path.has(body.name)) {
        throw new Error(`Circular position reference involving ${body.name}`);
      }

      const initialX = WorldService.toNumber(body.position.x);
      const initialY = WorldService.toNumber(body.position.y);
      const radius = Math.hypot(initialX, initialY);
      const speed = WorldService.toNumber(body.speed);
      const orbitalCenter = body.orbitalCenter ?? body.position.relativeTo;
      const updatedAt = body.updatedAt ?? time;
      const elapsedSeconds = (time.getTime() - updatedAt.getTime()) / 1_000;
      let localPosition = { x: initialX, y: initialY };

      if (orbitalCenter && radius > 0 && speed !== 0) {
        const direction = body.clockwise ? 1 : -1;
        const angle = (direction * speed * elapsedSeconds) / radius;
        const cos = Math.cos(angle);
        const sin = Math.sin(angle);
        localPosition = {
          x: initialX * cos - initialY * sin,
          y: initialX * sin + initialY * cos,
        };
      }

      const referenceName = body.position.relativeTo;
      if (referenceName) {
        const reference = world.bodiesByName.get(referenceName);
        if (!reference) {
          throw new Error(
            `Position reference ${referenceName} for ${body.name} was not found`,
          );
        }
        const nextPath = new Set(path).add(body.name);
        localPosition = WorldService.add(
          localPosition,
          resolve(reference, nextPath),
        );
      }

      positions.set(body.name, localPosition);
      return localPosition;
    };

    world.bodies.forEach((body) => resolve(body, new Set()));
    return positions;
  }

  static getBodyVelocity<TBody extends WorldBodyLike>(
    world: WorldSnapshotLike<TBody>,
    bodyName: string,
    time: Date,
    sampleSeconds = 0.5,
  ) {
    const before = WorldService.getBodyPositions(
      world,
      new Date(time.getTime() - sampleSeconds * 1_000),
    ).get(bodyName);
    const after = WorldService.getBodyPositions(
      world,
      new Date(time.getTime() + sampleSeconds * 1_000),
    ).get(bodyName);
    if (!before || !after) return { x: 0, y: 0 };
    return {
      x: (after.x - before.x) / (sampleSeconds * 2),
      y: (after.y - before.y) / (sampleSeconds * 2),
    };
  }

  static calculateGravityAcceleration<TBody extends WorldBodyLike>(
    position: Vector,
    bodies: readonly TBody[],
    getBodyPosition: (body: TBody) => Vector | undefined,
  ): Vector {
    let x = 0;
    let y = 0;

    for (const body of bodies) {
      const bodyPosition = getBodyPosition(body);
      if (!bodyPosition) continue;

      const deltaX = bodyPosition.x - position.x;
      const deltaY = bodyPosition.y - position.y;
      const radiusSquared = deltaX ** 2 + deltaY ** 2;
      if (radiusSquared === 0) continue;

      const scale =
        (GRAVITATIONAL_CONSTANT * WorldService.toNumber(body.mass)) /
        (radiusSquared * Math.sqrt(radiusSquared));
      x += deltaX * scale;
      y += deltaY * scale;
    }

    return { x, y };
  }

  static calculateAcceleration(
    position: Vector,
    calculateGravityAcceleration: (position: Vector) => Vector,
    thrustAcceleration?: Vector,
  ): Vector {
    const gravityAcceleration = calculateGravityAcceleration(position);
    return {
      x: gravityAcceleration.x + (thrustAcceleration?.x ?? 0),
      y: gravityAcceleration.y + (thrustAcceleration?.y ?? 0),
    };
  }

  static integrateStep(
    motion: Motion,
    seconds: number,
    calculateAcceleration: (position: Vector, offsetSeconds: number) => Vector,
  ): Motion {
    const position1 = motion.velocity;
    const velocity1 = calculateAcceleration(motion.position, 0);
    const position2 = WorldService.add(motion.velocity, velocity1, seconds / 2);
    const velocity2 = calculateAcceleration(
      WorldService.add(motion.position, position1, seconds / 2),
      seconds / 2,
    );
    const position3 = WorldService.add(motion.velocity, velocity2, seconds / 2);
    const velocity3 = calculateAcceleration(
      WorldService.add(motion.position, position2, seconds / 2),
      seconds / 2,
    );
    const position4 = WorldService.add(motion.velocity, velocity3, seconds);
    const velocity4 = calculateAcceleration(
      WorldService.add(motion.position, position3, seconds),
      seconds,
    );

    return {
      position: {
        x:
          motion.position.x +
          (seconds / 6) *
            (position1.x + 2 * position2.x + 2 * position3.x + position4.x),
        y:
          motion.position.y +
          (seconds / 6) *
            (position1.y + 2 * position2.y + 2 * position3.y + position4.y),
      },
      velocity: {
        x:
          motion.velocity.x +
          (seconds / 6) *
            (velocity1.x + 2 * velocity2.x + 2 * velocity3.x + velocity4.x),
        y:
          motion.velocity.y +
          (seconds / 6) *
            (velocity1.y + 2 * velocity2.y + 2 * velocity3.y + velocity4.y),
      },
    };
  }

  static calculateRequiredBurnAcceleration(
    targetVelocity: Vector,
    remainingSeconds: number,
    currentVelocity: Vector,
    position: Vector,
    calculateGravityAcceleration: (position: Vector) => Vector,
  ): Vector {
    const desiredAcceleration = {
      x: (targetVelocity.x - currentVelocity.x) / remainingSeconds,
      y: (targetVelocity.y - currentVelocity.y) / remainingSeconds,
    };
    const gravityAcceleration = calculateGravityAcceleration(position);

    return {
      x: desiredAcceleration.x - gravityAcceleration.x,
      y: desiredAcceleration.y - gravityAcceleration.y,
    };
  }

  static calculateTargetSpeedBurnDuration(
    targetVelocity: Vector,
    currentVelocity: Vector,
    position: Vector,
    maximumAcceleration: number,
    calculateGravityAcceleration: (position: Vector) => Vector,
  ) {
    if (!Number.isFinite(maximumAcceleration) || maximumAcceleration <= 0) {
      return undefined;
    }

    const velocityChange = {
      x: targetVelocity.x - currentVelocity.x,
      y: targetVelocity.y - currentVelocity.y,
    };
    const velocityChangeSquared = velocityChange.x ** 2 + velocityChange.y ** 2;
    if (velocityChangeSquared === 0) return 0;
    const engineOnlyMinimumDuration =
      Math.sqrt(velocityChangeSquared) / maximumAcceleration;

    const gravityAcceleration = calculateGravityAcceleration(position);
    const compensationAcceleration = {
      x: -gravityAcceleration.x,
      y: -gravityAcceleration.y,
    };
    const linearCoefficient =
      2 *
      (velocityChange.x * compensationAcceleration.x +
        velocityChange.y * compensationAcceleration.y);
    const constantCoefficient =
      compensationAcceleration.x ** 2 +
      compensationAcceleration.y ** 2 -
      maximumAcceleration ** 2;
    const discriminant =
      linearCoefficient ** 2 - 4 * velocityChangeSquared * constantCoefficient;
    if (discriminant < 0) return undefined;

    const root = Math.sqrt(discriminant);
    const reciprocalDurations = [
      (-linearCoefficient + root) / (2 * velocityChangeSquared),
      (-linearCoefficient - root) / (2 * velocityChangeSquared),
    ].filter((value) => Number.isFinite(value) && value > 0);
    const reciprocalDuration = Math.max(...reciprocalDurations);
    if (!Number.isFinite(reciprocalDuration)) return undefined;

    return Math.max(1 / reciprocalDuration, engineOnlyMinimumDuration);
  }

  static calculateMaximumEngineAcceleration(maximumThrustPercent = 100) {
    return (
      ((MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG) *
      (maximumThrustPercent / 100)
    );
  }

  static rotateAttachedPosition(
    position: Vector,
    elapsedSeconds: number,
    rotationPeriodSeconds: number | undefined,
    collisionRadius: number,
  ): Vector {
    if (!rotationPeriodSeconds || rotationPeriodSeconds <= 0) return position;
    const angle =
      (FULL_ROTATION_RADIANS * elapsedSeconds) / rotationPeriodSeconds;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    const x = position.x * cos - position.y * sin;
    const y = position.x * sin + position.y * cos;
    const radius = Math.hypot(x, y);
    const scale = radius > 0 ? collisionRadius / radius : 1;
    return { x: x * scale, y: y * scale };
  }

  static getSurfaceVelocity(
    position: Vector,
    rotationPeriodSeconds: number | undefined,
  ): Vector {
    if (!rotationPeriodSeconds || rotationPeriodSeconds <= 0) {
      return { x: 0, y: 0 };
    }
    const angularVelocity = FULL_ROTATION_RADIANS / rotationPeriodSeconds;
    return {
      x: -position.y * angularVelocity,
      y: position.x * angularVelocity,
    };
  }

  static getImpactMotionState(impactSpeed: number): SpaceshipMotionState {
    return impactSpeed > CRASH_SPEED_METERS_PER_SECOND ? 'crashed' : 'landed';
  }

  private static serializePosition(position: {
    x: NumericValue;
    y: NumericValue;
    relativeTo?: string;
    relativeToId?: string;
  }): SerializedPosition {
    return {
      x: position.x.toString(),
      y: position.y.toString(),
      ...(position.relativeTo ? { relativeTo: position.relativeTo } : {}),
      ...(position.relativeToId ? { relativeToId: position.relativeToId } : {}),
    };
  }
}
