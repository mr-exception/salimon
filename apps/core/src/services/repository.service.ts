import {
  SpaceshipModel,
  WorldBodyModel,
  type SpaceshipDocument,
  type WorldBodyCollectionName,
  type WorldBodyDocument,
} from '@models';

const DATABASE_FLUSH_INTERVAL_MS = 5 * 60 * 1_000;

type WorldData = Record<WorldBodyCollectionName, WorldBodyDocument[]>;
type Timer = ReturnType<typeof setInterval>;

function cloneDate(value: Date | undefined) {
  return value ? new Date(value) : undefined;
}

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

function cloneSpaceship(spaceship: SpaceshipDocument): SpaceshipDocument {
  return {
    ...spaceship,
    position: { ...spaceship.position },
    velocity: spaceship.velocity ? { ...spaceship.velocity } : undefined,
    stats: spaceship.stats
      ? {
          ...spaceship.stats,
          thrusterDurability: [...spaceship.stats.thrusterDurability],
        }
      : undefined,
    simulatedAt: cloneDate(spaceship.simulatedAt),
    createdAt: new Date(spaceship.createdAt),
    updatedAt: new Date(spaceship.updatedAt),
  };
}

function toPublicBody(body: WorldBodyDocument): WorldBodyDocument {
  const { _id, updatedAt, ...publicBody } = cloneBody(body);
  void _id;
  void updatedAt;
  return publicBody as WorldBodyDocument;
}

export class RepositoryService {
  private static worldData: WorldData | undefined;
  private static spaceshipsBySecurityCode:
    | Map<string, SpaceshipDocument>
    | undefined;
  private static startPromise: Promise<void> | undefined;
  private static databaseFlushTimer: Timer | undefined;
  private static databaseFlushPromise: Promise<void> | undefined;

  static async start() {
    RepositoryService.startPromise ??= RepositoryService.loadFromDatabase();
    return RepositoryService.startPromise;
  }

  static stop() {
    if (RepositoryService.databaseFlushTimer) {
      clearInterval(RepositoryService.databaseFlushTimer);
      RepositoryService.databaseFlushTimer = undefined;
    }

    RepositoryService.startPromise = undefined;
    RepositoryService.worldData = undefined;
    RepositoryService.spaceshipsBySecurityCode = undefined;
  }

  static async getWorldData() {
    await RepositoryService.start();
    return cloneWorldData(RepositoryService.requireWorldData());
  }

  static async getWorldSystemsBodies() {
    const worldData = await RepositoryService.getWorldData();
    return {
      planets: worldData.planets.map(toPublicBody),
      moons: worldData.moons.map(toPublicBody),
      stars: worldData.stars.map(toPublicBody),
    };
  }

  static async updateWorldBodies(
    updater: (worldData: WorldData) => number,
  ): Promise<number> {
    await RepositoryService.start();
    return updater(RepositoryService.requireWorldData());
  }

  static async updateSpaceships(
    updater: (
      spaceshipsBySecurityCode: Map<string, SpaceshipDocument>,
    ) => number,
  ): Promise<number> {
    await RepositoryService.start();
    return updater(RepositoryService.requireSpaceshipsBySecurityCode());
  }

  static async insertSpaceship(spaceship: SpaceshipDocument) {
    await RepositoryService.start();
    RepositoryService.requireSpaceshipsBySecurityCode().set(
      spaceship.securityCode,
      cloneSpaceship(spaceship),
    );
    return cloneSpaceship(spaceship);
  }

  static async findSpaceshipBySecurityCode(securityCode: string) {
    await RepositoryService.start();
    const spaceship =
      RepositoryService.requireSpaceshipsBySecurityCode().get(securityCode);
    return spaceship ? cloneSpaceship(spaceship) : undefined;
  }

  static async updateSpaceshipBySecurityCode(
    securityCode: string,
    update: Partial<SpaceshipDocument>,
  ) {
    await RepositoryService.start();
    const spaceships = RepositoryService.requireSpaceshipsBySecurityCode();
    const spaceship = spaceships.get(securityCode);
    if (!spaceship) return undefined;

    const updatedSpaceship = cloneSpaceship({
      ...spaceship,
      ...update,
      position: update.position ?? spaceship.position,
      velocity: update.velocity ?? spaceship.velocity,
      stats: update.stats ?? spaceship.stats,
    });
    spaceships.set(securityCode, updatedSpaceship);
    return cloneSpaceship(updatedSpaceship);
  }

  static async findOldestSpaceshipsForSimulation(
    invocationTime: Date,
    batchSize: number,
  ) {
    await RepositoryService.start();
    return [...RepositoryService.requireSpaceshipsBySecurityCode().values()]
      .filter((spaceship) => {
        const simulatedAt = spaceship.simulatedAt;
        return simulatedAt
          ? simulatedAt < invocationTime
          : spaceship.updatedAt < invocationTime;
      })
      .sort(
        (left, right) =>
          (left.simulatedAt?.getTime() ?? left.updatedAt.getTime()) -
          (right.simulatedAt?.getTime() ?? right.updatedAt.getTime()),
      )
      .slice(0, batchSize)
      .map(cloneSpaceship);
  }

  static async updatePropagatedSpaceship(
    spaceship: SpaceshipDocument,
    update: Partial<SpaceshipDocument>,
  ) {
    await RepositoryService.start();
    const spaceships = RepositoryService.requireSpaceshipsBySecurityCode();
    const current = spaceships.get(spaceship.securityCode);
    if (!current) return cloneSpaceship(spaceship);

    if (current.updatedAt.getTime() !== spaceship.updatedAt.getTime()) {
      return cloneSpaceship(current);
    }

    const updatedSpaceship = cloneSpaceship({
      ...current,
      ...update,
      position: update.position ?? current.position,
      velocity: update.velocity ?? current.velocity,
      stats: update.stats ?? current.stats,
    });
    spaceships.set(spaceship.securityCode, updatedSpaceship);
    return cloneSpaceship(updatedSpaceship);
  }

  static async flushToDatabase() {
    await RepositoryService.start();

    if (RepositoryService.databaseFlushPromise) {
      return RepositoryService.databaseFlushPromise;
    }

    RepositoryService.databaseFlushPromise = (async () => {
      const worldData = RepositoryService.requireWorldData();
      const spaceships = [
        ...RepositoryService.requireSpaceshipsBySecurityCode().values(),
      ];

      await Promise.all([
        ...(
          [
            ['planets', worldData.planets],
            ['moons', worldData.moons],
            ['stars', worldData.stars],
          ] as const
        ).map(([collectionName, bodies]) =>
          WorldBodyModel.replaceBodies(collectionName, bodies.map(cloneBody)),
        ),
        SpaceshipModel.replaceSpaceships(spaceships.map(cloneSpaceship)),
      ]);
    })().finally(() => {
      RepositoryService.databaseFlushPromise = undefined;
    });

    return RepositoryService.databaseFlushPromise;
  }

  private static async loadFromDatabase() {
    const [worldData, spaceships] = await Promise.all([
      WorldBodyModel.findAllWorldBodies(),
      SpaceshipModel.findAll(),
    ]);

    RepositoryService.worldData = cloneWorldData(worldData);
    RepositoryService.spaceshipsBySecurityCode = new Map(
      spaceships.map((spaceship) => [
        spaceship.securityCode,
        cloneSpaceship(spaceship),
      ]),
    );

    RepositoryService.databaseFlushTimer ??= setInterval(() => {
      void RepositoryService.flushToDatabase().catch((error: unknown) => {
        console.error('Failed to store repository data', error);
      });
    }, DATABASE_FLUSH_INTERVAL_MS);
  }

  private static requireWorldData() {
    if (!RepositoryService.worldData) {
      throw new Error('Repository world data has not been loaded');
    }

    return RepositoryService.worldData;
  }

  private static requireSpaceshipsBySecurityCode() {
    if (!RepositoryService.spaceshipsBySecurityCode) {
      throw new Error('Repository spaceship data has not been loaded');
    }

    return RepositoryService.spaceshipsBySecurityCode;
  }
}
