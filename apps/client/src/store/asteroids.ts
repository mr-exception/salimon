import type { InventoryMaterial, Planet, Star } from '@repo/types';
import { WorldService, type Vector } from '@repo/world';
import {
  getBodyWorldVelocity,
  getWorldPosition,
  INVENTORY_MATERIALS,
  spaceshipState,
} from './world';

export type AsteroidMaterial = {
  name: InventoryMaterial;
  massKg: number;
};

export type Asteroid = {
  id: string;
  name: string;
  orbitingBodyName: string;
  orbitingBodyKind: 'Planet' | 'Star';
  position: {
    x: bigint;
    y: bigint;
    relativeTo: string;
  };
  cTime: number;
  radius: bigint;
  mass: bigint;
  orbitalCenter: string | null;
  speed: bigint;
  clockwise: boolean;
  orbitSurfaceDistanceMeters: number;
  materials: AsteroidMaterial[];
};

type AsteroidRecord = Omit<
  Asteroid,
  'position' | 'radius' | 'mass' | 'speed'
> & {
  position: {
    x: string;
    y: string;
    relativeTo: string;
  };
  radius: string;
  mass: string;
  speed: string;
};

export type AsteroidParent = {
  body: Planet | Star;
  kind: 'Planet' | 'Star';
};

const DATABASE_NAME = 'salimon-asteroids';
const DATABASE_VERSION = 1;
const ASTEROIDS_STORE = 'asteroids';
const EARTH_REFERENCE_MASS_KG = 5.972e24;
const EARTH_REFERENCE_ASTEROID_COUNT = 50;
const MAX_ASTEROIDS_PER_PARENT = 50_000;
const ASTEROID_SPAWN_RANGE_METERS = 10_000_000_000;
const MIN_ORBIT_SURFACE_DISTANCE_METERS = 400_000;
const MAX_ORBIT_SURFACE_DISTANCE_METERS = 3_000_000;
const MIN_ORBIT_SPEED_METERS_PER_SECOND = 6_000;
const MAX_ORBIT_SPEED_METERS_PER_SECOND = 25_000;
const MIN_MASS_KG = 1_500;
const MAX_MASS_KG = 15_000;
const MIN_RADIUS_METERS = 75;
const MAX_RADIUS_METERS = 750;

let databasePromise: Promise<IDBDatabase> | undefined;
let asteroidsPromise: Promise<Asteroid[]> | undefined;
let asteroidPersistTimer: ReturnType<typeof setTimeout> | undefined;
const pendingPersistedAsteroids = new Map<string, Asteroid>();

export async function getClientAsteroids() {
  asteroidsPromise ??= readStoredAsteroids();
  return asteroidsPromise;
}

export async function ensureClientAsteroidsForParents(
  parents: AsteroidParent[],
) {
  const asteroids = await getClientAsteroids();
  const asteroidCountByParent = new Map<string, number>();
  asteroids.forEach((asteroid) => {
    asteroidCountByParent.set(
      asteroid.orbitingBodyName,
      (asteroidCountByParent.get(asteroid.orbitingBodyName) ?? 0) + 1,
    );
  });

  const generated: Asteroid[] = [];
  parents.forEach(({ body, kind }) => {
    if (!isClientAsteroidParentNearSpaceship(body)) return;

    const existingCount = asteroidCountByParent.get(body.name) ?? 0;
    const targetCount = getAsteroidTargetCount(body);
    for (let index = existingCount; index < targetCount; index += 1) {
      generated.push(createAsteroid(body, kind, index));
    }
  });

  if (generated.length === 0) return asteroids;

  const nextAsteroids = [...asteroids, ...generated];
  await writeStoredAsteroids(generated);
  asteroidsPromise = Promise.resolve(nextAsteroids);
  return nextAsteroids;
}

export function advanceAsteroid(asteroid: Asteroid, elapsedSeconds: number) {
  if (elapsedSeconds <= 0) return;

  const position = WorldService.advanceBodyPosition(asteroid, elapsedSeconds);
  asteroid.position.x = BigInt(position.x);
  asteroid.position.y = BigInt(position.y);
  asteroid.cTime = Date.now();
}

export function getAsteroidWorldVelocity(asteroid: Asteroid): Vector {
  const centerVelocity = getBodyWorldVelocity(asteroid.orbitingBodyName);
  const x = Number(asteroid.position.x);
  const y = Number(asteroid.position.y);
  const radius = Math.hypot(x, y);
  if (radius === 0) return centerVelocity;

  const direction = asteroid.clockwise ? 1 : -1;
  const speed = Number(asteroid.speed);
  return {
    x: centerVelocity.x + (direction * -y * speed) / radius,
    y: centerVelocity.y + (direction * x * speed) / radius,
  };
}

export function getAsteroidSurfaceDistance(asteroid: Asteroid) {
  return asteroid.orbitSurfaceDistanceMeters;
}

export function getAsteroidMaterialMassKg(asteroid: Asteroid) {
  return asteroid.materials.reduce(
    (total, material) => total + material.massKg,
    0,
  );
}

export function mineAsteroid(asteroid: Asteroid, requestedMassKg: number) {
  if (!Number.isFinite(requestedMassKg) || requestedMassKg <= 0) return [];

  let remainingRequest = Math.min(
    requestedMassKg,
    getAsteroidMaterialMassKg(asteroid),
  );
  if (remainingRequest <= 0) return [];

  const minedMaterials: AsteroidMaterial[] = [];
  asteroid.materials = asteroid.materials.flatMap((material) => {
    if (remainingRequest <= 0 || material.massKg <= 0) {
      return material.massKg > 0 ? [material] : [];
    }

    const minedMassKg = Math.min(material.massKg, remainingRequest);
    remainingRequest -= minedMassKg;
    minedMaterials.push({ name: material.name, massKg: minedMassKg });

    const remainingMaterialMassKg = material.massKg - minedMassKg;
    return remainingMaterialMassKg > 0
      ? [{ ...material, massKg: remainingMaterialMassKg }]
      : [];
  });

  if (minedMaterials.length === 0) return [];

  const nextMassKg = getAsteroidMaterialMassKg(asteroid);
  asteroid.mass = BigInt(Math.max(0, Math.round(nextMassKg)));
  if (nextMassKg <= 0) {
    asteroid.radius = 0n;
  }
  scheduleAsteroidPersist(asteroid);
  return minedMaterials;
}

export function isClientAsteroidParentNearSpaceship(parent: Planet | Star) {
  const parentPosition = getWorldPosition(parent.position);
  const spaceshipPosition = getWorldPosition(spaceshipState.position);
  const surfaceDistance = Math.max(
    0,
    Math.hypot(
      Number(parentPosition.x - spaceshipPosition.x),
      Number(parentPosition.y - spaceshipPosition.y),
    ) -
      Number(parent.radius) -
      Number(spaceshipState.radius),
  );

  return surfaceDistance <= ASTEROID_SPAWN_RANGE_METERS;
}

function getAsteroidTargetCount(parent: Planet | Star) {
  const mass = Number(parent.mass);
  if (!Number.isFinite(mass) || mass <= 0) return 0;

  return Math.min(
    MAX_ASTEROIDS_PER_PARENT,
    Math.round(
      (mass / EARTH_REFERENCE_MASS_KG) * EARTH_REFERENCE_ASTEROID_COUNT,
    ),
  );
}

function createAsteroid(
  parent: Planet | Star,
  kind: AsteroidParent['kind'],
  index: number,
): Asteroid {
  const random = createSeededRandom(`${parent.name}:${index}`);
  const mass = Math.round(randomBetween(random, MIN_MASS_KG, MAX_MASS_KG));
  const radius = Math.round(
    mapRange(
      mass,
      MIN_MASS_KG,
      MAX_MASS_KG,
      MIN_RADIUS_METERS,
      MAX_RADIUS_METERS,
    ),
  );
  const surfaceDistance = randomBetween(
    random,
    MIN_ORBIT_SURFACE_DISTANCE_METERS,
    MAX_ORBIT_SURFACE_DISTANCE_METERS,
  );
  const orbitalRadius = Number(parent.radius) + radius + surfaceDistance;
  const angle = randomBetween(random, 0, Math.PI * 2);

  return {
    id: `${slugify(parent.name)}-${index}`,
    name: `${parent.name} Asteroid ${index + 1}`,
    orbitingBodyName: parent.name,
    orbitingBodyKind: kind,
    position: {
      x: BigInt(Math.round(Math.cos(angle) * orbitalRadius)),
      y: BigInt(Math.round(Math.sin(angle) * orbitalRadius)),
      relativeTo: parent.name,
    },
    cTime: Date.now(),
    radius: BigInt(radius),
    mass: BigInt(mass),
    orbitalCenter: null,
    speed: BigInt(
      Math.round(
        randomBetween(
          random,
          MIN_ORBIT_SPEED_METERS_PER_SECOND,
          MAX_ORBIT_SPEED_METERS_PER_SECOND,
        ),
      ),
    ),
    clockwise: random() >= 0.5,
    orbitSurfaceDistanceMeters: surfaceDistance,
    materials: createMaterials(random, mass),
  };
}

function createMaterials(random: () => number, mass: number) {
  const count = Math.floor(randomBetween(random, 1, 4));
  const materialPool = [...INVENTORY_MATERIALS];
  const materials: AsteroidMaterial[] = [];
  let remainingMass = mass;

  for (let index = 0; index < count; index += 1) {
    const materialIndex = Math.floor(random() * materialPool.length);
    const [name] = materialPool.splice(materialIndex, 1);
    const massKg =
      index === count - 1
        ? remainingMass
        : Math.max(
            1,
            Math.round(remainingMass * randomBetween(random, 0.18, 0.72)),
          );

    materials.push({ name, massKg });
    remainingMass -= massKg;
  }

  return materials;
}

function mapRange(
  value: number,
  inputMin: number,
  inputMax: number,
  outputMin: number,
  outputMax: number,
) {
  return (
    outputMin +
    ((value - inputMin) / (inputMax - inputMin)) * (outputMax - outputMin)
  );
}

function randomBetween(random: () => number, min: number, max: number) {
  return min + random() * (max - min);
}

function createSeededRandom(seed: string) {
  let value = 2166136261;
  for (let index = 0; index < seed.length; index += 1) {
    value ^= seed.charCodeAt(index);
    value = Math.imul(value, 16777619);
  }

  return () => {
    value += 0x6d2b79f5;
    let result = value;
    result = Math.imul(result ^ (result >>> 15), result | 1);
    result ^= result + Math.imul(result ^ (result >>> 7), result | 61);
    return ((result ^ (result >>> 14)) >>> 0) / 4294967296;
  };
}

function slugify(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
}

async function readStoredAsteroids() {
  const database = await openDatabase();

  return new Promise<Asteroid[]>((resolve, reject) => {
    const request = database
      .transaction(ASTEROIDS_STORE, 'readonly')
      .objectStore(ASTEROIDS_STORE)
      .getAll();

    request.onsuccess = () =>
      resolve(
        (request.result as AsteroidRecord[])
          .map(deserializeAsteroid)
          .map(advanceAsteroidToNow),
      );
    request.onerror = () => reject(request.error);
  });
}

async function writeStoredAsteroids(asteroids: Asteroid[]) {
  const database = await openDatabase();

  return new Promise<void>((resolve, reject) => {
    const transaction = database.transaction(ASTEROIDS_STORE, 'readwrite');
    const store = transaction.objectStore(ASTEROIDS_STORE);
    asteroids.forEach((asteroid) => store.put(serializeAsteroid(asteroid)));
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}

function scheduleAsteroidPersist(asteroid: Asteroid) {
  pendingPersistedAsteroids.set(asteroid.id, asteroid);
  if (asteroidPersistTimer) return;

  asteroidPersistTimer = setTimeout(() => {
    asteroidPersistTimer = undefined;
    const asteroids = [...pendingPersistedAsteroids.values()];
    pendingPersistedAsteroids.clear();
    void writeStoredAsteroids(asteroids).catch((error: unknown) => {
      console.error('Failed to persist asteroids', error);
      asteroids.forEach((pendingAsteroid) => {
        pendingPersistedAsteroids.set(pendingAsteroid.id, pendingAsteroid);
      });
      scheduleAsteroidPersist(asteroids[0]);
    });
  }, 1_000);
}

function openDatabase() {
  databasePromise ??= new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open(DATABASE_NAME, DATABASE_VERSION);
    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains(ASTEROIDS_STORE)) {
        database.createObjectStore(ASTEROIDS_STORE, { keyPath: 'id' });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });

  return databasePromise;
}

function serializeAsteroid(asteroid: Asteroid): AsteroidRecord {
  return {
    ...asteroid,
    position: {
      x: asteroid.position.x.toString(),
      y: asteroid.position.y.toString(),
      relativeTo: asteroid.position.relativeTo,
    },
    radius: asteroid.radius.toString(),
    mass: asteroid.mass.toString(),
    speed: asteroid.speed.toString(),
  };
}

function deserializeAsteroid(record: AsteroidRecord): Asteroid {
  return {
    ...record,
    position: {
      x: BigInt(record.position.x),
      y: BigInt(record.position.y),
      relativeTo: record.position.relativeTo,
    },
    radius: BigInt(record.radius),
    mass: BigInt(record.mass),
    speed: BigInt(record.speed),
  };
}

function advanceAsteroidToNow(asteroid: Asteroid) {
  advanceAsteroid(asteroid, Math.max(0, (Date.now() - asteroid.cTime) / 1_000));
  return asteroid;
}
