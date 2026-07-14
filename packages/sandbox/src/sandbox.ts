import {
  SandboxObject,
  type SandboxObjectParams,
  type SandboxVector,
} from "./sandbox-object";

export type SandboxState = {
  objects: SandboxObjectParams[];
};

export type SandboxCollisionEvent = {
  objectAId: string;
  objectBId: string;
  position: SandboxVector;
  forceN: number;
};

export type SandboxCollisionListener = (event: SandboxCollisionEvent) => void;

export class SandBox {
  private static readonly fps = 30;
  private static readonly tickIntervalMs = 1000 / SandBox.fps;

  private readonly objects = new Map<string, SandboxObject>();
  private readonly collisionListeners = new Set<SandboxCollisionListener>();
  private tickCount = 0;
  private tickTimer?: ReturnType<typeof setTimeout>;

  constructor(state: Partial<SandboxState> = {}) {
    state.objects?.forEach((object) => this.addObject(object));
  }

  addObject(params: SandboxObject | SandboxObjectParams) {
    const object =
      params instanceof SandboxObject ? params : new SandboxObject(params);

    object.setSandbox(this);
    this.objects.set(object.id, object);
    return object;
  }

  removeObject(id: string) {
    const object = this.objects.get(id);
    this.objects.delete(id);
    object?.setSandbox(undefined);
    return object;
  }

  getObject(id: string) {
    return this.objects.get(id);
  }

  listObjects() {
    return Array.from(this.objects.values());
  }

  onCollision(listener: SandboxCollisionListener) {
    this.collisionListeners.add(listener);

    return () => {
      this.collisionListeners.delete(listener);
    };
  }

  start() {
    if (this.tickTimer) {
      clearTimeout(this.tickTimer);
    }

    const tick = () => {
      const startedAt = Date.now();
      const objects = this.listObjects();

      objects.forEach((object) => object.tick(startedAt));
      this.emitCollisions(objects);

      const calcTime = Date.now() - startedAt;

      this.tickCount += 1;
      console.log(
        `tick passed, count: ${this.tickCount}, calcTime: ${calcTime}ms, fps: ${SandBox.fps}`,
      );

      this.tickTimer = setTimeout(
        tick,
        Math.max(SandBox.tickIntervalMs - calcTime, 0),
      );
    };

    tick();
  }

  private emitCollisions(objects: SandboxObject[]) {
    for (let index = 0; index < objects.length; index += 1) {
      for (
        let compareIndex = index + 1;
        compareIndex < objects.length;
        compareIndex += 1
      ) {
        const collision = this.getCollisionEvent(
          objects[index],
          objects[compareIndex],
        );

        if (collision) {
          this.collisionListeners.forEach((listener) => listener(collision));
        }
      }
    }
  }

  private getCollisionEvent(
    objectA: SandboxObject,
    objectB: SandboxObject,
  ): SandboxCollisionEvent | undefined {
    const collisionDistance = objectA.radius + objectB.radius;
    const delta = {
      x: objectB.position.x - objectA.position.x,
      y: objectB.position.y - objectA.position.y,
    };
    const distance = Math.hypot(delta.x, delta.y);

    if (distance > collisionDistance) {
      return undefined;
    }

    const normal =
      distance > 0
        ? { x: delta.x / distance, y: delta.y / distance }
        : { x: 1, y: 0 };
    const position = this.getCollisionPosition(
      objectA,
      objectB,
      normal,
      distance,
    );

    return {
      objectAId: objectA.id,
      objectBId: objectB.id,
      position,
      forceN: this.getCollisionForceN(objectA, objectB, normal),
    };
  }

  private getCollisionPosition(
    objectA: SandboxObject,
    objectB: SandboxObject,
    normal: SandboxVector,
    distance: number,
  ) {
    if (objectA.radius > 0) {
      return {
        x: objectA.position.x + normal.x * objectA.radius,
        y: objectA.position.y + normal.y * objectA.radius,
      };
    }

    if (objectB.radius > 0) {
      return {
        x: objectB.position.x - normal.x * objectB.radius,
        y: objectB.position.y - normal.y * objectB.radius,
      };
    }

    return {
      x: objectA.position.x + normal.x * (distance / 2),
      y: objectA.position.y + normal.y * (distance / 2),
    };
  }

  private getCollisionForceN(
    objectA: SandboxObject,
    objectB: SandboxObject,
    normal: SandboxVector,
  ) {
    const velocityA = objectA.velocity ?? { x: 0, y: 0 };
    const velocityB = objectB.velocity ?? { x: 0, y: 0 };
    const relativeVelocity = {
      x: velocityB.x - velocityA.x,
      y: velocityB.y - velocityA.y,
    };
    const closingSpeed = Math.max(
      0,
      -(relativeVelocity.x * normal.x + relativeVelocity.y * normal.y),
    );
    const reducedMass =
      objectA.mass + objectB.mass === 0
        ? 0
        : (objectA.mass * objectB.mass) / (objectA.mass + objectB.mass);

    return reducedMass * closingSpeed * SandBox.fps;
  }
}
