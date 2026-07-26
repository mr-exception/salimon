import type { SerializedWorldSystems } from '@repo/types';

export type WorldSector = {
  x: number;
  y: number;
};

export type WorldSectorBounds = {
  left: bigint;
  right: bigint;
  top: bigint;
  bottom: bigint;
};

type CachedWorldSector = WorldSector & {
  key: string;
  systems: SerializedWorldSystems;
  scannedAt: number;
};

const DATABASE_NAME = 'salimon-world';
const DATABASE_VERSION = 1;
const FETCHED_SECTORS_DATABASE_NAME = 'salimon-world-fetched-sectors';
const FETCHED_SECTORS_DATABASE_VERSION = 1;
const WORLD_SECTORS_STORE = 'world-sectors';
const FETCHED_SECTORS_STORE = 'fetched-sectors';
export const METERS_PER_LIGHT_YEAR = 9_460_730_472_580_800n;
export const WORLD_SECTOR_SIZE_LIGHT_YEARS = 10n;
export const WORLD_SECTOR_SIZE_METERS =
  WORLD_SECTOR_SIZE_LIGHT_YEARS * METERS_PER_LIGHT_YEAR;
const WORLD_SECTOR_HALF_SIZE_METERS = WORLD_SECTOR_SIZE_METERS / 2n;

let databasePromise: Promise<IDBDatabase> | undefined;
let fetchedSectorsDatabasePromise: Promise<IDBDatabase> | undefined;

export function getWorldSectorKey(sector: WorldSector) {
  return `${sector.x}:${sector.y}`;
}

export function getWorldSector(position: { x: bigint; y: bigint }) {
  return {
    x: floorDiv(
      position.x + WORLD_SECTOR_HALF_SIZE_METERS,
      WORLD_SECTOR_SIZE_METERS,
    ),
    y: floorDiv(
      position.y + WORLD_SECTOR_HALF_SIZE_METERS,
      WORLD_SECTOR_SIZE_METERS,
    ),
  };
}

export function getWorldSectorBounds(sector: WorldSector): WorldSectorBounds {
  const left =
    BigInt(sector.x) * WORLD_SECTOR_SIZE_METERS - WORLD_SECTOR_HALF_SIZE_METERS;
  const top =
    BigInt(sector.y) * WORLD_SECTOR_SIZE_METERS - WORLD_SECTOR_HALF_SIZE_METERS;

  return {
    left,
    right: left + WORLD_SECTOR_SIZE_METERS,
    top,
    bottom: top + WORLD_SECTOR_SIZE_METERS,
  };
}

export function getWorldSectorsInBounds(
  bounds: WorldSectorBounds,
  maximumSectorCount = 400,
) {
  const first = getWorldSector({ x: bounds.left, y: bounds.top });
  const last = getWorldSector({ x: bounds.right, y: bounds.bottom });
  const minX = Math.min(first.x, last.x);
  const maxX = Math.max(first.x, last.x);
  const minY = Math.min(first.y, last.y);
  const maxY = Math.max(first.y, last.y);
  const count = (maxX - minX + 1) * (maxY - minY + 1);

  if (count > maximumSectorCount) return [];

  const sectors: WorldSector[] = [];
  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      sectors.push({ x, y });
    }
  }

  return sectors;
}

export async function readCachedWorldSectors() {
  const database = await openDatabase();

  return new Promise<CachedWorldSector[]>((resolve, reject) => {
    const request = database
      .transaction(WORLD_SECTORS_STORE, 'readonly')
      .objectStore(WORLD_SECTORS_STORE)
      .getAll();

    request.onsuccess = () => resolve(request.result as CachedWorldSector[]);
    request.onerror = () => reject(request.error);
  });
}

export async function writeCachedWorldSector(
  sector: WorldSector,
  systems: SerializedWorldSystems,
) {
  const database = await openDatabase();
  const value: CachedWorldSector = {
    ...sector,
    key: getWorldSectorKey(sector),
    systems,
    scannedAt: Date.now(),
  };

  return new Promise<void>((resolve, reject) => {
    const transaction = database.transaction(WORLD_SECTORS_STORE, 'readwrite');
    transaction.objectStore(WORLD_SECTORS_STORE).put(value);
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}

export async function readFetchedWorldSectorKeys() {
  const database = await openFetchedSectorsDatabase();

  return new Promise<Set<string>>((resolve, reject) => {
    const request = database
      .transaction(FETCHED_SECTORS_STORE, 'readonly')
      .objectStore(FETCHED_SECTORS_STORE)
      .getAllKeys();

    request.onsuccess = () =>
      resolve(new Set(request.result.map((key) => String(key))));
    request.onerror = () => reject(request.error);
  });
}

export async function markWorldSectorFetched(sector: WorldSector) {
  const database = await openFetchedSectorsDatabase();
  const key = getWorldSectorKey(sector);

  return new Promise<void>((resolve, reject) => {
    const transaction = database.transaction(
      FETCHED_SECTORS_STORE,
      'readwrite',
    );
    transaction.objectStore(FETCHED_SECTORS_STORE).put(
      {
        key,
        x: sector.x,
        y: sector.y,
        fetchedAt: Date.now(),
      },
      key,
    );
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}

function openDatabase() {
  databasePromise ??= openDatabaseWithStore(
    DATABASE_NAME,
    DATABASE_VERSION,
    WORLD_SECTORS_STORE,
    (database) =>
      database.createObjectStore(WORLD_SECTORS_STORE, { keyPath: 'key' }),
  );

  return databasePromise;
}

function openFetchedSectorsDatabase() {
  fetchedSectorsDatabasePromise ??= openDatabaseWithStore(
    FETCHED_SECTORS_DATABASE_NAME,
    FETCHED_SECTORS_DATABASE_VERSION,
    FETCHED_SECTORS_STORE,
    (database) => database.createObjectStore(FETCHED_SECTORS_STORE),
  );

  return fetchedSectorsDatabasePromise;
}

function openDatabaseWithStore(
  name: string,
  version: number,
  storeName: string,
  createStore: (database: IDBDatabase) => void,
) {
  return new Promise<IDBDatabase>((resolve, reject) => {
    const open = (databaseVersion?: number) => {
      const request =
        databaseVersion === undefined
          ? indexedDB.open(name)
          : indexedDB.open(name, databaseVersion);

      request.onupgradeneeded = () => {
        const database = request.result;
        if (!database.objectStoreNames.contains(storeName)) {
          createStore(database);
        }
      };
      request.onsuccess = () => {
        const database = request.result;
        if (database.objectStoreNames.contains(storeName)) {
          resolve(database);
          return;
        }

        const nextVersion = Math.max(database.version + 1, version + 1);
        database.close();
        open(nextVersion);
      };
      request.onerror = () => reject(request.error);
    };

    open();
  });
}

function floorDiv(value: bigint, divisor: bigint) {
  const quotient = value / divisor;
  const remainder = value % divisor;

  return Number(remainder < 0n ? quotient - 1n : quotient);
}
