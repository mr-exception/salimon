import type { SandBox } from "./sandbox";

export type SandboxVector = {
  x: number;
  y: number;
};

export type SandboxForce = SandboxVector & {
  id?: string;
  durationMs: number;
};

export type SandboxObjectParams = {
  id: string;
  name?: string;
  kind?: string;
  mass: number;
  radius?: number;
  tickMs: number;
  position: SandboxVector;
  capturedAt?: number;
  velocity?: SandboxVector;
  orbitalCenterId?: string;
  metadata?: Record<string, unknown>;
};

export class SandboxObject {
  private static readonly gravitationalConstant = 6.6743e-11;

  id: string;
  name?: string;
  kind?: string;
  mass: number;
  radius: number;
  tickMs: number;
  position: SandboxVector;
  capturedAt: number;
  velocity?: SandboxVector;
  orbitalCenterId?: string;
  metadata?: Record<string, unknown>;
  activeForce?: SandboxForce;
  private sandbox?: SandBox;

  constructor(params: SandboxObjectParams) {
    this.id = params.id;
    this.name = params.name;
    this.kind = params.kind;
    this.mass = params.mass;
    this.radius = params.radius ?? 0;
    this.tickMs = params.tickMs;
    this.position = params.position;
    this.capturedAt = params.capturedAt ?? Date.now();
    this.velocity = params.velocity;
    this.orbitalCenterId = params.orbitalCenterId;
    this.metadata = params.metadata;
  }

  setSandbox(sandbox?: SandBox) {
    this.sandbox = sandbox;
  }

  tick(timestampMs: number) {
    if (!Number.isFinite(timestampMs)) {
      throw new Error("Tick timestamp must be a finite millisecond value.");
    }

    if (!Number.isFinite(this.tickMs) || this.tickMs <= 0) {
      throw new Error("Object tickMs must be a positive millisecond value.");
    }

    if (timestampMs - this.capturedAt < this.tickMs) {
      return false;
    }

    if (!Number.isFinite(this.capturedAt)) {
      throw new Error("Captured timestamp must be a finite millisecond value.");
    }

    const elapsedMilliseconds = timestampMs - this.capturedAt;

    if (elapsedMilliseconds <= 0) {
      return false;
    }

    const forceElapsedMilliseconds = Math.min(
      elapsedMilliseconds,
      this.activeForce?.durationMs ?? 0,
    );
    const forceAcceleration = this.getForceAcceleration();

    if (forceElapsedMilliseconds > 0 && forceAcceleration) {
      this.advanceWithGravity(
        forceElapsedMilliseconds / 1000,
        forceAcceleration,
      );
    }

    const remainingMilliseconds =
      elapsedMilliseconds - forceElapsedMilliseconds;

    if (remainingMilliseconds > 0) {
      this.advanceBySeconds(remainingMilliseconds / 1000);
    }

    this.updateForceDuration(forceElapsedMilliseconds);
    this.capturedAt = timestampMs;
    return true;
  }

  force(force: SandboxForce) {
    if (!Number.isFinite(force.x) || !Number.isFinite(force.y)) {
      throw new Error("Force must be a finite vector value.");
    }

    if (!Number.isFinite(force.durationMs) || force.durationMs < 0) {
      throw new Error(
        "Force duration must be a non-negative millisecond value.",
      );
    }

    if (!Number.isFinite(this.mass) || this.mass === 0) {
      throw new Error(
        "Object mass must be finite and non-zero to apply force.",
      );
    }

    if (force.durationMs === 0) {
      this.activeForce = undefined;
      return;
    }

    this.activeForce = { ...force };
  }

  private advanceBySeconds(seconds: number) {
    if (this.orbitalCenterId) {
      this.advanceOrbit(seconds);
    } else {
      this.advanceWithGravity(seconds);
    }
  }

  private getForceAcceleration() {
    if (!this.activeForce) {
      return undefined;
    }

    if (!Number.isFinite(this.mass) || this.mass === 0) {
      throw new Error(
        "Object mass must be finite and non-zero to apply force.",
      );
    }

    return {
      x: this.activeForce.x / this.mass,
      y: this.activeForce.y / this.mass,
    };
  }

  private updateForceDuration(elapsedMilliseconds: number) {
    if (!this.activeForce || elapsedMilliseconds <= 0) {
      return;
    }

    const durationMs = this.activeForce.durationMs - elapsedMilliseconds;

    this.activeForce =
      durationMs > 0 ? { ...this.activeForce, durationMs } : undefined;
  }

  private advanceOrbit(seconds: number) {
    const velocity = this.velocity ?? { x: 0, y: 0 };
    const speed = SandboxObject.getVectorLength(velocity);

    if (speed === 0) {
      return;
    }

    const center = this.sandbox?.getObject(this.orbitalCenterId ?? "");

    if (!center) {
      this.position = {
        x: this.position.x + velocity.x * seconds,
        y: this.position.y + velocity.y * seconds,
      };
      return;
    }

    const radialVector = {
      x: this.position.x - center.position.x,
      y: this.position.y - center.position.y,
    };
    const radius = SandboxObject.getVectorLength(radialVector);

    if (radius === 0) {
      return;
    }

    const crossProduct =
      radialVector.x * velocity.y - radialVector.y * velocity.x;
    const direction = crossProduct < 0 ? -1 : 1;
    const angle = direction * (speed / radius) * seconds;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    const nextRadialVector = {
      x: radialVector.x * cos - radialVector.y * sin,
      y: radialVector.x * sin + radialVector.y * cos,
    };

    this.position = {
      x: center.position.x + nextRadialVector.x,
      y: center.position.y + nextRadialVector.y,
    };
    this.velocity = {
      x: (-direction * nextRadialVector.y * speed) / radius,
      y: (direction * nextRadialVector.x * speed) / radius,
    };
  }

  private advanceWithGravity(
    seconds: number,
    additionalAcceleration: SandboxVector = { x: 0, y: 0 },
  ) {
    const velocity = this.velocity ?? { x: 0, y: 0 };
    const gravityAcceleration = this.calculateGravityAcceleration();
    const acceleration = {
      x: gravityAcceleration.x + additionalAcceleration.x,
      y: gravityAcceleration.y + additionalAcceleration.y,
    };
    const nextVelocity = {
      x: velocity.x + acceleration.x * seconds,
      y: velocity.y + acceleration.y * seconds,
    };

    this.velocity = nextVelocity;
    this.position = {
      x: this.position.x + nextVelocity.x * seconds,
      y: this.position.y + nextVelocity.y * seconds,
    };
  }

  private calculateGravityAcceleration() {
    const acceleration = { x: 0, y: 0 };
    const objects = this.sandbox?.listObjects() ?? [];

    objects.forEach((object) => {
      if (object.id === this.id || object.mass === 0) {
        return;
      }

      const distance = {
        x: object.position.x - this.position.x,
        y: object.position.y - this.position.y,
      };
      const distanceSquared = distance.x ** 2 + distance.y ** 2;

      if (distanceSquared === 0) {
        return;
      }

      const distanceLength = Math.sqrt(distanceSquared);
      const accelerationMagnitude =
        SandboxObject.gravitationalConstant * (object.mass / distanceSquared);

      acceleration.x += accelerationMagnitude * (distance.x / distanceLength);
      acceleration.y += accelerationMagnitude * (distance.y / distanceLength);
    });

    return acceleration;
  }

  private static getVectorLength(vector: SandboxVector) {
    return Math.sqrt(vector.x ** 2 + vector.y ** 2);
  }
}
