import {
  WorldBodyModel,
  type SerializedPosition,
  type WorldBodyCollectionName,
  type WorldBodyDocument,
} from '@models';

const FULL_ROTATION_RADIANS = Math.PI * 2;
const SIMULATION_INTERVAL_MS = 1_000;
const DATABASE_FLUSH_INTERVAL_MS = 5 * 60 * 1_000;

type WorldData = Record<WorldBodyCollectionName, WorldBodyDocument[]>;
type Timer = ReturnType<typeof setInterval>;

function cloneBody(body: WorldBodyDocument): WorldBodyDocument {
  return {
    ...body,
    position: { ...body.position },
    updatedAt: new Date(body.updatedAt),
  };
}

function cloneWorldData(worldData: WorldData): WorldData {
  return {
    planets: worldData.planets.map(cloneBody),
    moons: worldData.moons.map(cloneBody),
    stars: worldData.stars.map(cloneBody),
  };
}

function advancePosition(
  body: WorldBodyDocument,
  elapsedSeconds: number,
): SerializedPosition {
  const x = BigInt(body.position.x);
  const y = BigInt(body.position.y);
  const orbitalRadius = Math.hypot(Number(x), Number(y));
  const speed = Number(BigInt(body.speed));

  if (
    !body.orbitalCenter ||
    orbitalRadius === 0 ||
    speed === 0 ||
    elapsedSeconds <= 0
  ) {
    return body.position;
  }

  const direction = body.clockwise ? 1 : -1;
  const angle =
    ((direction * speed * elapsedSeconds) / orbitalRadius) %
    FULL_ROTATION_RADIANS;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);

  return {
    x: BigInt(Math.round(Number(x) * cos - Number(y) * sin)).toString(),
    y: BigInt(Math.round(Number(x) * sin + Number(y) * cos)).toString(),
    ...(body.position.relativeTo
      ? { relativeTo: body.position.relativeTo }
      : {}),
  };
}

function toPublicBody(body: WorldBodyDocument): WorldBodyDocument {
  const { _id, updatedAt, ...publicBody } = cloneBody(body);
  void _id;
  void updatedAt;
  return publicBody as WorldBodyDocument;
}

export class OrbitalUpdaterService {
  private static worldData: WorldData | undefined;
  private static simulationTimer: Timer | undefined;
  private static databaseFlushTimer: Timer | undefined;
  private static startPromise: Promise<void> | undefined;
  private static databaseFlushPromise: Promise<void> | undefined;

  static async start() {
    OrbitalUpdaterService.startPromise ??=
      OrbitalUpdaterService.startSimulation();
    return OrbitalUpdaterService.startPromise;
  }

  private static async startSimulation() {
    const loadedWorld = await WorldBodyModel.findAllWorldBodies();
    OrbitalUpdaterService.worldData = cloneWorldData(loadedWorld);
    OrbitalUpdaterService.updatePositions(new Date());

    OrbitalUpdaterService.simulationTimer ??= setInterval(() => {
      OrbitalUpdaterService.updatePositions(new Date());
    }, SIMULATION_INTERVAL_MS);

    OrbitalUpdaterService.databaseFlushTimer ??= setInterval(() => {
      void OrbitalUpdaterService.flushToDatabase().catch((error: unknown) => {
        console.error('Failed to store simulated world data', error);
      });
    }, DATABASE_FLUSH_INTERVAL_MS);
  }

  static stop() {
    if (OrbitalUpdaterService.simulationTimer) {
      clearInterval(OrbitalUpdaterService.simulationTimer);
      OrbitalUpdaterService.simulationTimer = undefined;
    }

    if (OrbitalUpdaterService.databaseFlushTimer) {
      clearInterval(OrbitalUpdaterService.databaseFlushTimer);
      OrbitalUpdaterService.databaseFlushTimer = undefined;
    }

    OrbitalUpdaterService.startPromise = undefined;
  }

  static async getWorldData() {
    await OrbitalUpdaterService.start();
    return cloneWorldData(OrbitalUpdaterService.requireWorldData());
  }

  static async getWorldSystemsBodies() {
    const worldData = await OrbitalUpdaterService.getWorldData();
    return {
      planets: worldData.planets.map(toPublicBody),
      moons: worldData.moons.map(toPublicBody),
      stars: worldData.stars.map(toPublicBody),
    };
  }

  static async updateOrbitalBodies(time: string | Date) {
    await OrbitalUpdaterService.start();
    const invocationTime = new Date(time);
    if (Number.isNaN(invocationTime.getTime())) {
      throw new Error('Invocation time is invalid');
    }

    const selected = OrbitalUpdaterService.updatePositions(invocationTime);
    await OrbitalUpdaterService.flushToDatabase();

    return {
      selected,
      updated: selected,
    };
  }

  static async flushToDatabase() {
    await OrbitalUpdaterService.start();

    if (OrbitalUpdaterService.databaseFlushPromise) {
      return OrbitalUpdaterService.databaseFlushPromise;
    }

    OrbitalUpdaterService.databaseFlushPromise = (async () => {
      const worldData = OrbitalUpdaterService.requireWorldData();
      await Promise.all(
        (
          [
            ['planets', worldData.planets],
            ['moons', worldData.moons],
            ['stars', worldData.stars],
          ] as const
        ).map(([collectionName, bodies]) =>
          WorldBodyModel.replaceBodies(
            collectionName,
            bodies.map(cloneBody),
          ),
        ),
      );
    })().finally(() => {
      OrbitalUpdaterService.databaseFlushPromise = undefined;
    });

    return OrbitalUpdaterService.databaseFlushPromise;
  }

  private static updatePositions(invocationTime: Date) {
    const worldData = OrbitalUpdaterService.requireWorldData();
    let updated = 0;

    for (const body of [
      ...worldData.stars,
      ...worldData.planets,
      ...worldData.moons,
    ]) {
      const elapsedSeconds =
        (invocationTime.getTime() - body.updatedAt.getTime()) / 1_000;
      body.position = advancePosition(body, elapsedSeconds);
      body.updatedAt = invocationTime;
      updated += 1;
    }

    return updated;
  }

  private static requireWorldData() {
    if (!OrbitalUpdaterService.worldData) {
      throw new Error('World simulation has not been started');
    }

    return OrbitalUpdaterService.worldData;
  }
}
