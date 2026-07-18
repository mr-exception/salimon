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

export type SandboxCrashEvent = SandboxCollisionEvent;
export type SandboxCollisionListener = (event: SandboxCollisionEvent) => void;
export type SandboxCrashListener = (event: SandboxCrashEvent) => void;
export type SandboxObjectTickListener = (object: SandboxObject) => void;

class SandboxEventEmitter<TEvent> {
  private readonly listeners = new Set<(event: TEvent) => void>();

  subscribe(listener: (event: TEvent) => void) {
    this.listeners.add(listener);

    return () => {
      this.listeners.delete(listener);
    };
  }

  emit(event: TEvent) {
    this.listeners.forEach((listener) => listener(event));
  }
}

export class SandBox {
  private static readonly fps = 30;
  private static readonly tickIntervalMs = 1000 / SandBox.fps;
  private static readonly minimumTickDelayMs = 10;
  private static readonly defaultGravityForceCacheThresholdN = 0;

  private readonly objects = new Map<string, SandboxObject>();
  private readonly crashEvents = new SandboxEventEmitter<SandboxCrashEvent>();
  private readonly objectTickListeners = new Set<SandboxObjectTickListener>();
  private readonly activeCrashPairs = new Set<string>();
  private hasCheckedCrashes = false;
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

  static calculateCrashForceN(
    objectAMassKg: number,
    objectBMassKg: number,
    closingSpeedMetersPerSecond: number,
  ) {
    if (
      !Number.isFinite(objectAMassKg) ||
      !Number.isFinite(objectBMassKg) ||
      !Number.isFinite(closingSpeedMetersPerSecond)
    ) {
      throw new Error('Crash force inputs must be finite numbers.');
    }

    const reducedMass =
      objectAMassKg + objectBMassKg === 0
        ? 0
        : (objectAMassKg * objectBMassKg) / (objectAMassKg + objectBMassKg);

    return reducedMass * Math.max(0, closingSpeedMetersPerSecond) * SandBox.fps;
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
    return this.onCrash(listener);
  }

  onCrash(listener: SandboxCrashListener) {
    return this.crashEvents.subscribe(listener);
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
    this.emitCrashes(
      tickedObjects.length > 0 || this.hasCheckedCrashes
        ? tickedObjects
        : objects,
      objects,
    );

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
        Math.max(
          SandBox.tickIntervalMs - calcTime,
          SandBox.minimumTickDelayMs,
        ),
      );
    };

    tick();
  }

  stop() {
    if (!this.tickTimer) return;

    clearTimeout(this.tickTimer);
    this.tickTimer = undefined;
  }

  private emitCrashes(tickedObjects: SandboxObject[], objects: SandboxObject[]) {
    if (tickedObjects.length === 0) return;
    this.hasCheckedCrashes = true;

    const currentCrashPairs = new Set(this.activeCrashPairs);
    const checkedPairs = new Set<string>();

    for (const object of tickedObjects) {
      for (const comparedObject of objects) {
        if (object.id === comparedObject.id) continue;
        if (
          object.kind !== 'spaceship' &&
          comparedObject.kind !== 'spaceship'
        ) {
          continue;
        }

        const pairKey = SandBox.getObjectPairKey(
          object.id,
          comparedObject.id,
        );
        if (checkedPairs.has(pairKey)) continue;
        checkedPairs.add(pairKey);

        const crash = this.getCrashEvent(object, comparedObject);
        if (crash) {
          currentCrashPairs.add(pairKey);

          if (!this.activeCrashPairs.has(pairKey)) {
            this.crashEvents.emit(crash);
          }
        } else {
          currentCrashPairs.delete(pairKey);
        }
      }
    }

    this.activeCrashPairs.clear();
    currentCrashPairs.forEach((pairKey) => this.activeCrashPairs.add(pairKey));
  }

  private refreshGravityForceCaches() {
    this.objects.forEach((object) => this.refreshGravityForceCache(object));
  }

  private refreshGravityForceCachesWhenReady() {
    if (this.gravityForceCacheRefreshDepth > 0) return;

    this.refreshGravityForceCaches();
  }

  private refreshGravityForceCache(object: SandboxObject) {
    if (
      object.orbitalCenterId ||
      (object.kind === 'spaceship' &&
        typeof object.metadata?.relativeObjectId === 'string')
    ) {
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

  private getCrashEvent(
    objectA: SandboxObject,
    objectB: SandboxObject,
  ): SandboxCrashEvent | undefined {
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
      forceN: this.getCrashForceN(objectA, objectB, normal),
    };
  }

  private static getObjectPairKey(objectAId: string, objectBId: string) {
    return objectAId < objectBId
      ? `${objectAId}:${objectBId}`
      : `${objectBId}:${objectAId}`;
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

  private getCrashForceN(
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
    return SandBox.calculateCrashForceN(
      objectA.mass,
      objectB.mass,
      closingSpeed,
    );
  }
}
