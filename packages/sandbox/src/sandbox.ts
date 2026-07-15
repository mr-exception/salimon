import {
  SandboxObject,
  type SandboxObjectParams,
  type SandboxVector,
} from './sandbox-object';

export type SandboxState = {
  objects: SandboxObjectParams[];
  gravityForceCacheThresholdN: number;
};

export type SandboxCollisionEvent = {
  objectAId: string;
  objectBId: string;
  position: SandboxVector;
  forceN: number;
};

export type SandboxCollisionListener = (event: SandboxCollisionEvent) => void;
export type SandboxObjectTickListener = (object: SandboxObject) => void;

export class SandBox {
  private static readonly fps = 30;
  private static readonly tickIntervalMs = 1000 / SandBox.fps;
  private static readonly defaultGravityForceCacheThresholdN = 0;

  private readonly objects = new Map<string, SandboxObject>();
  private readonly collisionListeners = new Set<SandboxCollisionListener>();
  private readonly objectTickListeners = new Set<SandboxObjectTickListener>();
  private gravityForceCacheThresholdN =
    SandBox.defaultGravityForceCacheThresholdN;
  private gravityForceCacheRefreshDepth = 0;
  private tickCount = 0;
  private tickTimer?: ReturnType<typeof setTimeout>;

  constructor(state: Partial<SandboxState> = {}) {
    if (state.gravityForceCacheThresholdN !== undefined) {
      this.setGravityForceCacheThresholdN(state.gravityForceCacheThresholdN);
    }

    state.objects?.forEach((object) => this.addObject(object));
  }

  addObject(params: SandboxObject | SandboxObjectParams) {
    const object =
      params instanceof SandboxObject ? params : new SandboxObject(params);

    object.setSandbox(this);
    this.objects.set(object.id, object);
    this.refreshGravityForceCachesWhenReady();
    return object;
  }

  removeObject(id: string) {
    const object = this.objects.get(id);
    this.objects.delete(id);
    object?.setSandbox(undefined);
    object?.clearGravityForceCache();
    this.refreshGravityForceCachesWhenReady();
    return object;
  }

  batchObjects<T>(callback: () => T) {
    this.gravityForceCacheRefreshDepth += 1;

    try {
      return callback();
    } finally {
      this.gravityForceCacheRefreshDepth -= 1;

      if (this.gravityForceCacheRefreshDepth === 0) {
        this.refreshGravityForceCaches();
      }
    }
  }

  getObject(id: string) {
    return this.objects.get(id);
  }

  listObjects() {
    return Array.from(this.objects.values());
  }

  listCachedGravityObjects(object: SandboxObject) {
    if (object.orbitalCenterId) {
      return [];
    }

    const cachedObjectIds = object.listGravityForceCache().map((entry) => {
      return entry.objectId;
    });

    return cachedObjectIds.flatMap((objectId) => {
      const cachedObject = this.objects.get(objectId);
      return cachedObject ? [cachedObject] : [];
    });
  }

  setGravityForceCacheThresholdN(thresholdN: number) {
    if (!Number.isFinite(thresholdN) || thresholdN < 0) {
      throw new Error(
        'Gravity force cache threshold must be a non-negative finite value.',
      );
    }

    this.gravityForceCacheThresholdN = thresholdN;
    this.refreshGravityForceCachesWhenReady();
  }

  onCollision(listener: SandboxCollisionListener) {
    this.collisionListeners.add(listener);

    return () => {
      this.collisionListeners.delete(listener);
    };
  }

  onObjectTick(listener: SandboxObjectTickListener) {
    this.objectTickListeners.add(listener);

    return () => {
      this.objectTickListeners.delete(listener);
    };
  }

  tick(timestampMs = Date.now()) {
    const objects = this.listObjects();
    const tickedObjects = objects.filter((object) => {
      if (timestampMs - object.capturedAt < object.tickMs) {
        return false;
      }

      this.refreshGravityForceCache(object);
      return object.tick(timestampMs);
    });

    tickedObjects.forEach((object) => {
      this.objectTickListeners.forEach((listener) => listener(object));
    });
    this.emitCollisions(objects);

    return tickedObjects;
  }

  start() {
    this.stop();

    const tick = () => {
      const startedAt = Date.now();

      const tickedObjects = this.tick(startedAt);

      const calcTime = Date.now() - startedAt;

      this.tickCount += 1;
      if (tickedObjects.length > 0)
        console.log(
          `[tick] objects: ${tickedObjects.length}/${this.objects.size} (${calcTime}ms)`,
        );

      this.tickTimer = setTimeout(
        tick,
        Math.max(SandBox.tickIntervalMs - calcTime, 0),
      );
    };

    tick();
  }

  stop() {
    if (!this.tickTimer) return;

    clearTimeout(this.tickTimer);
    this.tickTimer = undefined;
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

  private refreshGravityForceCaches() {
    this.objects.forEach((object) => this.refreshGravityForceCache(object));
  }

  private refreshGravityForceCachesWhenReady() {
    if (this.gravityForceCacheRefreshDepth > 0) return;

    this.refreshGravityForceCaches();
  }

  private refreshGravityForceCache(object: SandboxObject) {
    if (object.orbitalCenterId) {
      object.clearGravityForceCache();
      return;
    }

    object.setGravityForceCache(
      this.listObjects()
        .flatMap((cachedObject) => {
          const forceN = object.getGravityForceN(cachedObject);
          return forceN > this.gravityForceCacheThresholdN
            ? [{ objectId: cachedObject.id, forceN }]
            : [];
        })
        .sort((left, right) => right.forceN - left.forceN),
    );
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
