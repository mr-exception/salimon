import { SandBox } from './sandbox';
import { SandboxObject, type SandboxVector } from './sandbox-object';

export const SANDBOX_WORLD_BODY_TICK_MS = 30_000;
export const SANDBOX_SPACESHIP_TICK_MS = 500;
export const DEFAULT_SPACESHIP_MASS_KG = 10_000;
export const DEFAULT_SPACESHIP_RADIUS_METERS = 200;
export const SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS = 1;
export const SANDBOX_MAX_ENGINE_THRUST_N = 1_000_000;

export type SandboxNumericValue = bigint | number | string;

export type SandboxSerializedPosition = {
  x: SandboxNumericValue;
  y: SandboxNumericValue;
  relativeTo?: string;
};

export type SandboxWorldBodyKind = 'star' | 'planet' | 'moon';

export type SandboxWorldBodyLike = {
  name: string;
  position: SandboxSerializedPosition;
  orbitalCenter: string | null;
  clockwise: boolean;
  speed: SandboxNumericValue;
  mass: SandboxNumericValue;
  radius?: SandboxNumericValue;
  updatedAt?: Date;
};

export type SandboxSpaceshipLike = {
  securityCode: string;
  position: SandboxSerializedPosition;
  direction: number;
  speed: SandboxNumericValue;
  velocity?: SandboxVector;
  motionState?: 'flying' | 'landed' | 'crashed';
  simulatedAt?: Date;
  updatedAt?: Date;
  mass?: SandboxNumericValue;
  radius?: SandboxNumericValue;
  activeFeature?: unknown;
};

export type SandboxBodySnapshot = {
  name: string;
  position: { x: string; y: string; relativeTo?: string };
  velocity?: SandboxVector;
  updatedAt: Date;
};

export type SandboxSpaceshipSnapshot = {
  securityCode: string;
  position: { x: string; y: string; relativeTo?: string };
  velocity: SandboxVector;
  speed: string;
  direction: number;
  simulatedAt: Date;
  updatedAt: Date;
};

export type SandboxSpaceshipTargetSpeedPlan = {
  targetSpeedMetersPerSecond: number;
  maximumThrustPercent: number;
  targetDirection?: number;
  targetVelocity: SandboxVector;
  maximumAcceleration: number;
  durationSeconds: number;
  elapsedSeconds: number;
};

type BodyReference = {
  name: string;
  kind: SandboxWorldBodyKind;
  relativeTo?: string;
};

export class WorldSandbox extends SandBox {
  private readonly bodyReferencesByObjectId = new Map<string, BodyReference>();
  private readonly spaceshipCodesByObjectId = new Map<string, string>();

  static getBodyObjectId(name: string) {
    return `body:${name}`;
  }

  static getSpaceshipObjectId(securityCode: string) {
    return `spaceship:${securityCode}`;
  }

  loadBodies(
    bodies: readonly SandboxWorldBodyLike[],
    kind: SandboxWorldBodyKind,
  ) {
    return this.batchObjects(() =>
      bodies.map((body) => this.loadBody(body, kind)),
    );
  }

  loadBody(body: SandboxWorldBodyLike, kind: SandboxWorldBodyKind) {
    const id = WorldSandbox.getBodyObjectId(body.name);
    const relativeTo =
      body.position.relativeTo ?? body.orbitalCenter ?? undefined;
    const center = relativeTo
      ? this.getObject(WorldSandbox.getBodyObjectId(relativeTo))
      : undefined;
    const localPosition = WorldSandbox.toVector(body.position);
    const position = center
      ? WorldSandbox.add(localPosition, center.position)
      : localPosition;

    this.bodyReferencesByObjectId.set(id, {
      name: body.name,
      kind,
      relativeTo,
    });

    return this.addObject({
      id,
      name: body.name,
      kind,
      mass: Number(body.mass),
      radius: body.radius === undefined ? 0 : Number(body.radius),
      tickMs: SANDBOX_WORLD_BODY_TICK_MS,
      position,
      capturedAt: body.updatedAt?.getTime(),
      velocity: WorldSandbox.getOrbitalVelocity(body),
      orbitalCenterId: relativeTo
        ? WorldSandbox.getBodyObjectId(relativeTo)
        : undefined,
      metadata: { bodyKind: kind },
    });
  }

  loadSpaceships(spaceships: readonly SandboxSpaceshipLike[]) {
    return this.batchObjects(() =>
      spaceships.map((spaceship) => this.loadSpaceship(spaceship)),
    );
  }

  loadSpaceship(spaceship: SandboxSpaceshipLike) {
    const id = WorldSandbox.getSpaceshipObjectId(spaceship.securityCode);
    const center = spaceship.position.relativeTo
      ? this.getObject(
          WorldSandbox.getBodyObjectId(spaceship.position.relativeTo),
        )
      : undefined;
    const radius = Number(spaceship.radius ?? DEFAULT_SPACESHIP_RADIUS_METERS);
    const localPosition = WorldSandbox.normalizeSpaceshipLocalPosition(
      WorldSandbox.toVector(spaceship.position),
      center,
      radius,
      spaceship.motionState,
    );
    const position = center
      ? WorldSandbox.add(localPosition, center.position)
      : localPosition;
    const relativeVelocity =
      spaceship.velocity ?? WorldSandbox.getHeadingVelocity(spaceship);
    const velocity = center?.velocity
      ? WorldSandbox.add(relativeVelocity, center.velocity)
      : relativeVelocity;

    this.spaceshipCodesByObjectId.set(id, spaceship.securityCode);

    const object = this.addObject({
      id,
      name: spaceship.securityCode,
      kind: 'spaceship',
      mass: Number(spaceship.mass ?? DEFAULT_SPACESHIP_MASS_KG),
      radius,
      tickMs: SANDBOX_SPACESHIP_TICK_MS,
      position,
      capturedAt: (
        spaceship.simulatedAt ??
        spaceship.updatedAt ??
        new Date()
      ).getTime(),
      velocity,
      metadata: {
        securityCode: spaceship.securityCode,
        motionState: spaceship.motionState,
        relativeTo: spaceship.position.relativeTo,
        ...(spaceship.position.relativeTo
          ? { relativePosition: localPosition }
          : {}),
        ...(spaceship.position.relativeTo ? { relativeVelocity } : {}),
        ...(spaceship.position.relativeTo
          ? {
              relativeObjectId: WorldSandbox.getBodyObjectId(
                spaceship.position.relativeTo,
              ),
            }
          : {}),
      },
    });

    this.restoreSpaceshipActiveForce(object, spaceship.activeFeature);

    return object;
  }

  getBodySnapshot(object: SandboxObject, timestampMs = object.capturedAt) {
    const reference = this.bodyReferencesByObjectId.get(object.id);
    if (!reference) return undefined;

    const center = reference.relativeTo
      ? this.getObject(WorldSandbox.getBodyObjectId(reference.relativeTo))
      : undefined;
    const position = center
      ? WorldSandbox.subtract(object.position, center.position)
      : object.position;

    return {
      name: reference.name,
      position: WorldSandbox.serializePosition(position, reference.relativeTo),
      velocity: object.velocity,
      updatedAt: new Date(timestampMs),
    } satisfies SandboxBodySnapshot;
  }

  getSpaceshipSnapshot(object: SandboxObject, timestampMs = object.capturedAt) {
    const securityCode = this.spaceshipCodesByObjectId.get(object.id);
    if (!securityCode) return undefined;

    const relativeTo =
      typeof object.metadata?.relativeTo === 'string'
        ? object.metadata.relativeTo
        : undefined;
    const center = relativeTo
      ? this.getObject(WorldSandbox.getBodyObjectId(relativeTo))
      : undefined;
    const position =
      center && relativeTo
        ? this.getSpaceshipRelativePosition(object, relativeTo)
        : object.position;
    const objectVelocity = object.velocity ?? { x: 0, y: 0 };
    const velocity =
      center && relativeTo
        ? (WorldSandbox.getStoredVector(object, 'relativeVelocity') ??
          WorldSandbox.subtract(
            objectVelocity,
            center.velocity ?? { x: 0, y: 0 },
          ))
        : objectVelocity;
    const speed = Math.hypot(velocity.x, velocity.y);

    return {
      securityCode,
      position: WorldSandbox.serializePosition(position, relativeTo),
      velocity,
      speed: Math.round(speed).toString(),
      direction:
        speed > 0
          ? ((Math.atan2(velocity.y, velocity.x) * 180) / Math.PI + 450) % 360
          : 0,
      simulatedAt: new Date(timestampMs),
      updatedAt: new Date(timestampMs),
    } satisfies SandboxSpaceshipSnapshot;
  }

  getBodyKind(object: SandboxObject) {
    return this.bodyReferencesByObjectId.get(object.id)?.kind;
  }

  getSpaceshipSecurityCode(object: SandboxObject) {
    return this.spaceshipCodesByObjectId.get(object.id);
  }

  launchSpaceship(securityCode: string, timestampMs = Date.now()) {
    const object = this.getObject(
      WorldSandbox.getSpaceshipObjectId(securityCode),
    );
    if (!object) return undefined;

    this.launchSpaceshipIfNeeded(object);
    object.metadata = {
      ...object.metadata,
      motionState: 'flying',
    };
    object.capturedAt = timestampMs;

    return this.getSpaceshipSnapshot(object, timestampMs);
  }

  startSpaceshipTargetSpeed(
    securityCode: string,
    params: {
      targetSpeedMetersPerSecond: number;
      maximumThrustPercent: number;
      targetDirection?: number;
    },
    timestampMs = Date.now(),
  ) {
    const object = this.getObject(
      WorldSandbox.getSpaceshipObjectId(securityCode),
    );
    if (!object) return undefined;

    const plan = this.getTargetSpeedPlan(object, params);
    if (!plan) return undefined;

    this.launchSpaceshipIfNeeded(object);
    object.metadata = {
      ...object.metadata,
      motionState: 'flying',
    };
    const velocity = object.velocity ?? { x: 0, y: 0 };
    const velocityChange = WorldSandbox.subtract(plan.targetVelocity, velocity);
    const changeMagnitude = Math.hypot(velocityChange.x, velocityChange.y);
    if (changeMagnitude === 0) return undefined;

    const forceMagnitude = object.mass * plan.maximumAcceleration;
    object.force({
      id: 'spaceship:target-speed',
      x: (velocityChange.x / changeMagnitude) * forceMagnitude,
      y: (velocityChange.y / changeMagnitude) * forceMagnitude,
      durationMs: plan.durationSeconds * 1_000,
    });
    object.capturedAt = timestampMs;

    return {
      plan,
      snapshot: this.getSpaceshipSnapshot(object, timestampMs),
    };
  }

  startSpaceshipThrusters(
    securityCode: string,
    thrusters: readonly { powerPercent: number; active: boolean }[],
    timestampMs = Date.now(),
  ) {
    const object = this.getObject(
      WorldSandbox.getSpaceshipObjectId(securityCode),
    );
    if (!object) return undefined;

    const force = WorldSandbox.getThrusterForce(thrusters);
    if (!force) return undefined;

    this.launchSpaceshipIfNeeded(object);
    object.metadata = {
      ...object.metadata,
      motionState: 'flying',
    };
    object.force({
      id: 'spaceship:thrusters',
      ...force,
      durationMs: Number.MAX_SAFE_INTEGER,
    });
    object.capturedAt = timestampMs;

    return this.getSpaceshipSnapshot(object, timestampMs);
  }

  stopSpaceshipForce(securityCode: string, timestampMs = Date.now()) {
    const object = this.getObject(
      WorldSandbox.getSpaceshipObjectId(securityCode),
    );
    if (!object) return undefined;

    object.force({ x: 0, y: 0, durationMs: 0 });
    object.capturedAt = timestampMs;
    return this.getSpaceshipSnapshot(object, timestampMs);
  }

  crashSpaceship(securityCode: string, timestampMs = Date.now()) {
    const object = this.getObject(
      WorldSandbox.getSpaceshipObjectId(securityCode),
    );
    if (!object) return undefined;

    object.velocity = { x: 0, y: 0 };
    object.force({ x: 0, y: 0, durationMs: 0 });
    object.capturedAt = timestampMs;
    object.metadata = {
      ...object.metadata,
      motionState: 'crashed',
    };
    return this.getSpaceshipSnapshot(object, timestampMs);
  }

  hasActiveForce(object: SandboxObject) {
    return object.activeForce !== undefined;
  }

  private static toVector(position: SandboxSerializedPosition): SandboxVector {
    return {
      x: Number(position.x),
      y: Number(position.y),
    };
  }

  private getTargetSpeedPlan(
    object: SandboxObject,
    params: {
      targetSpeedMetersPerSecond: number;
      maximumThrustPercent: number;
      targetDirection?: number;
    },
  ): SandboxSpaceshipTargetSpeedPlan | undefined {
    const targetSpeedMetersPerSecond = Number(
      params.targetSpeedMetersPerSecond,
    );
    const maximumThrustPercent = Number(params.maximumThrustPercent);
    if (
      !Number.isFinite(targetSpeedMetersPerSecond) ||
      targetSpeedMetersPerSecond < 0 ||
      !Number.isFinite(maximumThrustPercent) ||
      maximumThrustPercent <= 0 ||
      maximumThrustPercent > 100 ||
      (params.targetDirection !== undefined &&
        !Number.isFinite(params.targetDirection))
    ) {
      throw new Error('Invalid target speed feature parameters.');
    }

    const velocity = object.velocity ?? { x: 0, y: 0 };
    const currentSpeed = Math.hypot(velocity.x, velocity.y);
    const direction =
      params.targetDirection ??
      (currentSpeed > 0 ? Math.atan2(velocity.y, velocity.x) : 0);
    const targetVelocity = {
      x: Math.cos(direction) * targetSpeedMetersPerSecond,
      y: Math.sin(direction) * targetSpeedMetersPerSecond,
    };
    const maximumAcceleration =
      (SANDBOX_MAX_ENGINE_THRUST_N / object.mass) *
      (maximumThrustPercent / 100);
    const velocityChange = WorldSandbox.subtract(targetVelocity, velocity);
    const durationSeconds =
      Math.hypot(velocityChange.x, velocityChange.y) / maximumAcceleration;

    if (!Number.isFinite(durationSeconds) || durationSeconds <= 0) {
      return undefined;
    }

    return {
      targetSpeedMetersPerSecond,
      maximumThrustPercent,
      ...(params.targetDirection === undefined
        ? {}
        : { targetDirection: params.targetDirection }),
      targetVelocity,
      maximumAcceleration,
      durationSeconds,
      elapsedSeconds: 0,
    };
  }

  private launchSpaceshipIfNeeded(object: SandboxObject) {
    if (object.metadata?.motionState === 'flying') return;

    const relativeTo =
      typeof object.metadata?.relativeTo === 'string'
        ? object.metadata.relativeTo
        : undefined;
    const center = relativeTo
      ? this.getObject(WorldSandbox.getBodyObjectId(relativeTo))
      : undefined;
    if (!relativeTo || !center) return;

    const relativePosition = this.getSpaceshipRelativePosition(
      object,
      relativeTo,
    );
    const minimumLaunchRadius =
      center.radius + object.radius + SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS;
    const relativeRadius = Math.hypot(relativePosition.x, relativePosition.y);
    if (relativeRadius === 0) {
      const launchPosition = { x: minimumLaunchRadius, y: 0 };
      object.position = WorldSandbox.add(center.position, launchPosition);
      object.velocity = center.velocity
        ? { ...center.velocity }
        : { x: 0, y: 0 };
      object.metadata = {
        ...object.metadata,
        relativePosition: launchPosition,
        relativeVelocity: { x: 0, y: 0 },
      };
      return;
    }

    if (relativeRadius >= minimumLaunchRadius) {
      object.position = WorldSandbox.add(center.position, relativePosition);
      object.velocity = center.velocity
        ? { ...center.velocity }
        : { x: 0, y: 0 };
      object.metadata = {
        ...object.metadata,
        relativePosition,
        relativeVelocity: { x: 0, y: 0 },
      };
      return;
    }

    const launchPosition = {
      x: (relativePosition.x / relativeRadius) * minimumLaunchRadius,
      y: (relativePosition.y / relativeRadius) * minimumLaunchRadius,
    };

    object.position = WorldSandbox.add(center.position, launchPosition);
    object.velocity = center.velocity ? { ...center.velocity } : { x: 0, y: 0 };
    object.metadata = {
      ...object.metadata,
      relativePosition: launchPosition,
      relativeVelocity: { x: 0, y: 0 },
    };
  }

  private restoreSpaceshipActiveForce(
    object: SandboxObject,
    activeFeature: unknown,
  ) {
    if (!activeFeature || typeof activeFeature !== 'object') return;

    const feature = activeFeature as {
      type?: unknown;
      thrusters?: unknown;
      targetVelocity?: unknown;
      maximumAcceleration?: unknown;
      durationSeconds?: unknown;
      elapsedSeconds?: unknown;
    };
    if (feature.type === 'thrusters' || feature.type === 'manual-force') {
      if (!Array.isArray(feature.thrusters)) return;

      const force = WorldSandbox.getThrusterForce(
        feature.thrusters as { powerPercent: number; active: boolean }[],
      );
      if (!force) return;

      object.force({
        id: 'spaceship:thrusters',
        ...force,
        durationMs: Number.MAX_SAFE_INTEGER,
      });
      return;
    }

    if (feature.type !== 'target-speed') return;

    const targetVelocity = feature.targetVelocity;
    if (
      !targetVelocity ||
      typeof targetVelocity !== 'object' ||
      typeof (targetVelocity as SandboxVector).x !== 'number' ||
      typeof (targetVelocity as SandboxVector).y !== 'number' ||
      typeof feature.maximumAcceleration !== 'number' ||
      typeof feature.durationSeconds !== 'number'
    ) {
      return;
    }

    const velocity = object.velocity ?? { x: 0, y: 0 };
    const velocityChange = WorldSandbox.subtract(
      targetVelocity as SandboxVector,
      velocity,
    );
    const changeMagnitude = Math.hypot(velocityChange.x, velocityChange.y);
    const remainingSeconds = Math.max(
      0,
      feature.durationSeconds -
        (typeof feature.elapsedSeconds === 'number'
          ? feature.elapsedSeconds
          : 0),
    );
    if (changeMagnitude === 0 || remainingSeconds === 0) return;

    const forceMagnitude = object.mass * feature.maximumAcceleration;
    object.force({
      id: 'spaceship:target-speed',
      x: (velocityChange.x / changeMagnitude) * forceMagnitude,
      y: (velocityChange.y / changeMagnitude) * forceMagnitude,
      durationMs: remainingSeconds * 1_000,
    });
  }

  private static add(left: SandboxVector, right: SandboxVector): SandboxVector {
    return {
      x: left.x + right.x,
      y: left.y + right.y,
    };
  }

  private static subtract(
    left: SandboxVector,
    right: SandboxVector,
  ): SandboxVector {
    return {
      x: left.x - right.x,
      y: left.y - right.y,
    };
  }

  private static normalizeSpaceshipLocalPosition(
    position: SandboxVector,
    center: SandboxObject | undefined,
    spaceshipRadius: number,
    motionState: SandboxSpaceshipLike['motionState'],
  ) {
    if (
      !center ||
      motionState === 'flying' ||
      position.x !== 0 ||
      position.y !== 0
    ) {
      return position;
    }

    return {
      x: center.radius + spaceshipRadius,
      y: 0,
    };
  }

  private getSpaceshipRelativePosition(
    object: SandboxObject,
    relativeTo: string,
  ) {
    const storedRelativePosition = WorldSandbox.getStoredVector(
      object,
      'relativePosition',
    );
    if (storedRelativePosition) {
      return storedRelativePosition;
    }

    const center = this.getObject(WorldSandbox.getBodyObjectId(relativeTo));
    return center
      ? WorldSandbox.subtract(object.position, center.position)
      : object.position;
  }

  private static getStoredVector(object: SandboxObject, key: string) {
    const value = object.metadata?.[key];
    if (
      value &&
      typeof value === 'object' &&
      typeof (value as SandboxVector).x === 'number' &&
      typeof (value as SandboxVector).y === 'number'
    ) {
      return value as SandboxVector;
    }

    return undefined;
  }

  private static serializePosition(
    position: SandboxVector,
    relativeTo?: string,
  ) {
    return {
      x: Math.round(position.x).toString(),
      y: Math.round(position.y).toString(),
      ...(relativeTo ? { relativeTo } : {}),
    };
  }

  private static getOrbitalVelocity(body: SandboxWorldBodyLike): SandboxVector {
    const position = WorldSandbox.toVector(body.position);
    const radius = Math.hypot(position.x, position.y);
    const speed = Number(body.speed);
    const orbitalCenter = body.orbitalCenter ?? body.position.relativeTo;

    if (!orbitalCenter || radius === 0 || speed === 0) {
      return { x: 0, y: 0 };
    }

    const direction = body.clockwise ? 1 : -1;
    return {
      x: (-direction * position.y * speed) / radius,
      y: (direction * position.x * speed) / radius,
    };
  }

  private static getHeadingVelocity(
    spaceship: Pick<SandboxSpaceshipLike, 'direction' | 'speed'>,
  ) {
    const speed = Number(spaceship.speed);
    const headingRadians = (spaceship.direction * Math.PI) / 180;
    return {
      x: Math.sin(headingRadians) * speed,
      y: -Math.cos(headingRadians) * speed,
    };
  }

  private static getThrusterForce(
    thrusters: readonly { powerPercent: number; active: boolean }[],
  ) {
    const force = { x: 0, y: 0 };

    thrusters.forEach((thruster, index) => {
      if (!thruster.active || thruster.powerPercent <= 0) return;

      const thrustN =
        SANDBOX_MAX_ENGINE_THRUST_N * (thruster.powerPercent / 100);
      if (index === 0) force.y += thrustN;
      if (index === 1) force.x -= thrustN;
      if (index === 2) force.y -= thrustN;
      if (index === 3) force.x += thrustN;
    });

    return force.x === 0 && force.y === 0 ? undefined : force;
  }
}
