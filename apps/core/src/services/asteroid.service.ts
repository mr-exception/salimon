import type { WorldBodyDocument } from '@models';
import type {
  AsteroidDeposit,
  AsteroidDto,
  AsteroidSizeClass,
  InventoryMaterial,
  SerializedPosition,
} from '@repo/types';
import { WorldService } from '@repo/world';
import { TickingService } from './ticking.service';
import { resolvePositions } from './world-viewport.service/resolve-positions';

type Coordinate = {
  x: bigint;
  y: bigint;
};

type AsteroidSessionState = Omit<AsteroidDto, 'capturedAt'>;

type AsteroidDensityBand = {
  targetCount: number;
  minTonnes: number;
  maxTonnes: number;
};

type AsteroidDensityConfig = Record<AsteroidSizeClass, AsteroidDensityBand>;

const ASTEROID_SESSION_RANGE_METERS = 20_000_000;
const ASTEROID_SPAWN_MIN_DISTANCE_METERS = 25_000;
const ASTEROID_SPAWN_MAX_DISTANCE_METERS = 1_200_000;
const ASTEROID_MIN_TONNES = 1;
const ASTEROID_MAX_TONNES = 3_000;
const ASTEROID_BODY_CLEARANCE_METERS = 600_000;
const ASTEROID_SPAWN_ATTEMPTS = 32;
const DEFAULT_ASTEROID_DENSITY: AsteroidDensityConfig = {
  small: { targetCount: 18, minTonnes: 1, maxTonnes: 60 },
  medium: { targetCount: 8, minTonnes: 60, maxTonnes: 600 },
  large: { targetCount: 3, minTonnes: 600, maxTonnes: 3_000 },
};
const ASTEROID_MATERIAL_RARITY: {
  material: InventoryMaterial;
  weight: number;
}[] = [
  { material: 'silicates', weight: 34 },
  { material: 'iron', weight: 28 },
  { material: 'carbon', weight: 16 },
  { material: 'ice', weight: 12 },
  { material: 'hydrogen', weight: 5 },
  { material: 'nitrogen', weight: 3 },
  { material: 'silver', weight: 1.4 },
  { material: 'gold', weight: 0.3 },
];

export class AsteroidService {
  static getSessionRangeMeters() {
    return ASTEROID_SESSION_RANGE_METERS;
  }

  static getDefaultDensityConfig() {
    return structuredClone(DEFAULT_ASTEROID_DENSITY);
  }

  static async updateSessionAsteroids(params: {
    asteroids: AsteroidDto[];
    spaceshipPosition: SerializedPosition;
    density?: AsteroidDensityConfig;
    time?: Date;
  }) {
    const time = params.time ?? new Date();
    const { planets, moons, stars } =
      await TickingService.getWorldSystemsBodies();
    const bodies = [...stars, ...planets, ...moons];
    const bodyByName = new Map(bodies.map((body) => [body.name, body]));
    const positions = resolvePositions(bodies);
    const spaceshipPosition = this.resolvePosition(
      params.spaceshipPosition,
      positions,
    );
    const advanced = params.asteroids
      .map((asteroid) => this.advanceAsteroid(asteroid, time))
      .filter((asteroid) =>
        this.isAsteroidInSessionRange(asteroid, spaceshipPosition, positions),
      );
    const density = params.density ?? DEFAULT_ASTEROID_DENSITY;
    const bodyCandidates = this.getSpawnBodiesInRange({
      bodies,
      bodyByName,
      positions,
      spaceshipPosition,
      spaceshipReferenceName: params.spaceshipPosition.relativeTo,
    });
    const spawned = this.spawnAsteroidsForSystems({
      asteroids: advanced,
      bodies: bodyCandidates,
      positions,
      spaceshipPosition,
      bodyByName,
      density,
      time,
    });

    return spawned.map((asteroid) => this.toDto(asteroid, time));
  }

  static spawnAsteroidsForSystems(params: {
    asteroids: AsteroidSessionState[];
    bodies: WorldBodyDocument[];
    positions: Map<string, Coordinate>;
    spaceshipPosition: Coordinate;
    bodyByName: Map<string, WorldBodyDocument>;
    density: AsteroidDensityConfig;
    time: Date;
  }) {
    const asteroids = [...params.asteroids];
    const bodiesBySystem = this.groupBodiesBySystem(
      params.bodies,
      params.bodyByName,
    );

    bodiesBySystem.forEach((bodies, systemName) => {
      this.spawnAsteroidsForSystem({
        systemName,
        asteroids,
        bodies,
        positions: params.positions,
        spaceshipPosition: params.spaceshipPosition,
        density: params.density,
        time: params.time,
      });
    });

    return asteroids;
  }

  static spawnAsteroidsForSystem(params: {
    systemName: string;
    asteroids: AsteroidSessionState[];
    bodies: WorldBodyDocument[];
    positions: Map<string, Coordinate>;
    spaceshipPosition: Coordinate;
    density: AsteroidDensityConfig;
    time: Date;
  }) {
    (
      Object.entries(params.density) as [
        AsteroidSizeClass,
        AsteroidDensityBand,
      ][]
    ).forEach(([sizeClass, density]) => {
      const currentCount = params.asteroids.filter(
        (asteroid) =>
          asteroid.systemName === params.systemName &&
          asteroid.sizeClass === sizeClass,
      ).length;
      const spawnCount = Math.max(0, density.targetCount - currentCount);

      for (let index = 0; index < spawnCount; index += 1) {
        const asteroid = this.createAsteroidForSystem({
          systemName: params.systemName,
          sizeClass,
          density,
          bodies: params.bodies,
          positions: params.positions,
          spaceshipPosition: params.spaceshipPosition,
          time: params.time,
        });
        if (asteroid) params.asteroids.push(asteroid);
      }
    });
  }

  private static createAsteroidForSystem(params: {
    systemName: string;
    sizeClass: AsteroidSizeClass;
    density: AsteroidDensityBand;
    bodies: WorldBodyDocument[];
    positions: Map<string, Coordinate>;
    spaceshipPosition: Coordinate;
    time: Date;
  }): AsteroidSessionState | undefined {
    const bodies = params.bodies.filter((body) =>
      this.canBodySpawnAsteroidInSessionRange(
        body,
        params.positions,
        params.spaceshipPosition,
      ),
    );
    if (bodies.length === 0) return undefined;

    for (let attempt = 0; attempt < ASTEROID_SPAWN_ATTEMPTS; attempt += 1) {
      const orbitingBody =
        bodies[attempt % bodies.length] ??
        bodies[this.randomInteger(0, bodies.length - 1)];
      const bodyPosition = params.positions.get(orbitingBody.name);
      if (!bodyPosition) continue;

      const absolutePosition = this.randomPositionNearSpaceship(
        params.spaceshipPosition,
      );
      const orbitRadius = this.distance(absolutePosition, bodyPosition);
      if (orbitRadius < this.getMinimumOrbitRadius(orbitingBody)) {
        continue;
      }
      const position = {
        x: (absolutePosition.x - bodyPosition.x).toString(),
        y: (absolutePosition.y - bodyPosition.y).toString(),
        relativeTo: orbitingBody.name,
      };

      const massTonnes = this.randomMass(params.density);
      return {
        id: `${params.systemName}:${params.time.getTime()}:${Math.random()
          .toString(36)
          .slice(2, 10)}`,
        systemName: params.systemName,
        orbitingBodyName: orbitingBody.name,
        sizeClass: params.sizeClass,
        massTonnes,
        position,
        orbitalCenter: orbitingBody.name,
        clockwise: Math.random() >= 0.5,
        speed: this.randomOrbitSpeed(orbitRadius, orbitingBody).toString(),
        deposits: this.createDeposits(massTonnes),
      };
    }

    return undefined;
  }

  private static canBodySpawnAsteroidInSessionRange(
    body: WorldBodyDocument,
    positions: Map<string, Coordinate>,
    spaceshipPosition: Coordinate,
  ) {
    const bodyPosition = positions.get(body.name);
    if (!bodyPosition) return false;

    const bodyDistance = this.distance(bodyPosition, spaceshipPosition);
    return (
      bodyDistance + ASTEROID_SESSION_RANGE_METERS >=
      this.getMinimumOrbitRadius(body)
    );
  }

  private static advanceAsteroid(
    asteroid: AsteroidDto,
    time: Date,
  ): AsteroidSessionState {
    const capturedAt = Date.parse(asteroid.capturedAt);
    const elapsedSeconds = Number.isFinite(capturedAt)
      ? Math.max(0, (time.getTime() - capturedAt) / 1000)
      : 0;

    return {
      ...asteroid,
      position: WorldService.advanceBodyPosition(asteroid, elapsedSeconds),
    };
  }

  private static toDto(
    asteroid: AsteroidSessionState,
    time: Date,
  ): AsteroidDto {
    return {
      ...asteroid,
      capturedAt: time.toISOString(),
    };
  }

  private static getSpawnBodiesInRange(params: {
    bodies: WorldBodyDocument[];
    bodyByName: Map<string, WorldBodyDocument>;
    positions: Map<string, Coordinate>;
    spaceshipPosition: Coordinate;
    spaceshipReferenceName?: string;
  }) {
    const referencedBody = params.spaceshipReferenceName
      ? params.bodyByName.get(params.spaceshipReferenceName)
      : undefined;
    if (referencedBody) {
      const bodyNames = new Set<string>();
      const bodies: WorldBodyDocument[] = [];
      let body: WorldBodyDocument | undefined = referencedBody;
      while (body && !bodyNames.has(body.name)) {
        bodyNames.add(body.name);
        bodies.push(body);
        body = body.orbitalCenter
          ? params.bodyByName.get(body.orbitalCenter)
          : undefined;
      }
      params.bodies.forEach((candidate) => {
        if (
          candidate.orbitalCenter === referencedBody.name &&
          !bodyNames.has(candidate.name)
        ) {
          bodies.push(candidate);
        }
      });
      return bodies;
    }

    return [...params.bodies]
      .filter((body) => params.positions.has(body.name))
      .sort(
        (left, right) =>
          this.distance(
            params.positions.get(left.name)!,
            params.spaceshipPosition,
          ) -
          this.distance(
            params.positions.get(right.name)!,
            params.spaceshipPosition,
          ),
      )
      .slice(0, 6);
  }

  private static isAsteroidInSessionRange(
    asteroid: AsteroidSessionState,
    spaceshipPosition: Coordinate,
    bodyPositions: Map<string, Coordinate>,
  ) {
    const position = this.resolvePosition(asteroid.position, bodyPositions);
    return (
      this.distance(position, spaceshipPosition) <=
      ASTEROID_SESSION_RANGE_METERS
    );
  }

  private static resolvePosition(
    position: SerializedPosition,
    bodyPositions: Map<string, Coordinate>,
  ): Coordinate {
    const localPosition = {
      x: BigInt(position.x),
      y: BigInt(position.y),
    };
    const reference = position.relativeTo
      ? bodyPositions.get(position.relativeTo)
      : undefined;

    return reference
      ? {
          x: reference.x + localPosition.x,
          y: reference.y + localPosition.y,
        }
      : localPosition;
  }

  private static groupBodiesBySystem(
    bodies: WorldBodyDocument[],
    bodyByName: Map<string, WorldBodyDocument>,
  ) {
    const systems = new Map<string, WorldBodyDocument[]>();

    bodies.forEach((body) => {
      const systemName = this.getSystemName(body, bodyByName);
      const systemBodies = systems.get(systemName) ?? [];
      systemBodies.push(body);
      systems.set(systemName, systemBodies);
    });

    return systems;
  }

  private static getSystemName(
    body: WorldBodyDocument,
    bodyByName: Map<string, WorldBodyDocument>,
  ): string {
    let current = body;
    const path = new Set<string>();

    while (current.orbitalCenter && !path.has(current.name)) {
      path.add(current.name);
      const center = bodyByName.get(current.orbitalCenter);
      if (!center) break;
      current = center;
    }

    return current.name;
  }

  private static randomPositionNearSpaceship(spaceshipPosition: Coordinate) {
    const angle = Math.random() * Math.PI * 2;
    const distance = this.randomBetween(
      ASTEROID_SPAWN_MIN_DISTANCE_METERS,
      ASTEROID_SPAWN_MAX_DISTANCE_METERS,
    );
    return {
      x: spaceshipPosition.x + BigInt(Math.round(Math.cos(angle) * distance)),
      y: spaceshipPosition.y + BigInt(Math.round(Math.sin(angle) * distance)),
    };
  }

  private static getMinimumOrbitRadius(body: WorldBodyDocument) {
    return Number(body.radius) + ASTEROID_BODY_CLEARANCE_METERS;
  }

  private static randomOrbitSpeed(
    orbitRadius: number,
    body: WorldBodyDocument,
  ) {
    const escapeAdjustedSpeed = Math.sqrt(
      (6.6743e-11 * Number(body.mass)) / Math.max(1, orbitRadius),
    );
    return Math.max(0.05, Math.min(2_000, escapeAdjustedSpeed));
  }

  private static randomMass(density: AsteroidDensityBand) {
    const min = Math.max(ASTEROID_MIN_TONNES, density.minTonnes);
    const max = Math.min(ASTEROID_MAX_TONNES, Math.max(min, density.maxTonnes));
    return Math.round(this.randomBetween(min, max));
  }

  private static createDeposits(massTonnes: number): AsteroidDeposit[] {
    const materialCount = this.randomInteger(1, 3);
    const materials = this.pickWeightedUniqueMaterials(materialCount);
    const portions = materials.map(() => this.randomBetween(0.35, 1));
    const totalPortions = portions.reduce(
      (total, portion) => total + portion,
      0,
    );

    return materials.map((material, index) => ({
      material,
      amount: Math.round(massTonnes * (portions[index] / totalPortions)),
    }));
  }

  private static pickWeightedUniqueMaterials(count: number) {
    const available = [...ASTEROID_MATERIAL_RARITY];
    const materials: InventoryMaterial[] = [];

    while (materials.length < count && available.length > 0) {
      const totalWeight = available.reduce(
        (total, candidate) => total + candidate.weight,
        0,
      );
      let roll = this.randomBetween(0, totalWeight);
      const index = available.findIndex((candidate) => {
        roll -= candidate.weight;
        return roll <= 0;
      });
      const [selected] = available.splice(Math.max(0, index), 1);
      materials.push(selected.material);
    }

    return materials;
  }

  private static randomInteger(min: number, max: number) {
    return Math.floor(this.randomBetween(min, max + 1));
  }

  private static randomBetween(min: number, max: number) {
    return min + Math.random() * (max - min);
  }

  private static distance(left: Coordinate, right: Coordinate) {
    return Math.hypot(Number(left.x - right.x), Number(left.y - right.y));
  }
}
