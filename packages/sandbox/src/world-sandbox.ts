import { SandBox } from './sandbox';
import { SandboxObject, type SandboxVector } from './sandbox-object';

export const SANDBOX_WORLD_BODY_TICK_MS = 30_000;
export const SANDBOX_SPACESHIP_TICK_MS = 5_000;
export const DEFAULT_SPACESHIP_MASS_KG = 10_000;
export const DEFAULT_SPACESHIP_RADIUS_METERS = 200;

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
    const localPosition = WorldSandbox.toVector(spaceship.position);
    const position = center
      ? WorldSandbox.add(localPosition, center.position)
      : localPosition;

    this.spaceshipCodesByObjectId.set(id, spaceship.securityCode);

    return this.addObject({
      id,
      name: spaceship.securityCode,
      kind: 'spaceship',
      mass: Number(spaceship.mass ?? DEFAULT_SPACESHIP_MASS_KG),
      radius: Number(spaceship.radius ?? DEFAULT_SPACESHIP_RADIUS_METERS),
      tickMs: SANDBOX_SPACESHIP_TICK_MS,
      position,
      capturedAt: (
        spaceship.simulatedAt ??
        spaceship.updatedAt ??
        new Date()
      ).getTime(),
      velocity:
        spaceship.velocity ?? WorldSandbox.getHeadingVelocity(spaceship),
      metadata: {
        securityCode: spaceship.securityCode,
        motionState: spaceship.motionState,
        relativeTo: spaceship.position.relativeTo,
      },
    });
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
    const position = center
      ? WorldSandbox.subtract(object.position, center.position)
      : object.position;
    const velocity = object.velocity ?? { x: 0, y: 0 };
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

  private static toVector(position: SandboxSerializedPosition): SandboxVector {
    return {
      x: Number(position.x),
      y: Number(position.y),
    };
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

    if (!body.orbitalCenter || radius === 0 || speed === 0) {
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
}
