import axios from 'axios';
import { atom, getDefaultStore, useAtomValue, useSetAtom } from 'jotai';
import type {
  InventoryMaterial,
  Planet,
  Position,
  SerializedBody,
  SerializedWorldBody,
  SerializedWorldSystems,
  SpaceshipActiveFeature,
  Spaceship,
  SpaceshipDto,
  SpaceshipInventory,
  Star,
  World,
} from '@repo/types';
import {
  MAX_PROPAGATION_STEPS,
  SPACESHIP_MASS_KG,
  TARGET_STEP_SECONDS,
  WorldService,
  type Vector,
} from '@repo/world';
import type {
  SimulationFrameSnapshot,
  SimulationWorkerRequest,
  SimulationWorkerResponse,
} from './world-simulation-protocol';
import {
  getWorldSector,
  getWorldSectorKey,
  markWorldSectorFetched,
  readCachedWorldSectors,
  readFetchedWorldSectorKeys,
  writeCachedWorldSector,
  type WorldSector,
} from './world-sectors';
import { THRUSTER_DURABILITY_DRAIN_PER_SECOND } from './modules';

type Body = Planet | Spaceship | Star;
type SerializedVisiblePlanetBody = SerializedBody<Planet> & {
  type: 'planet' | 'moon' | 'blackhole';
};
type WorldListener = (
  world: World,
  changedBodyNames?: ReadonlySet<string>,
) => void;
type WorldViewportLoader = (
  request: WorldViewportRequest,
) => Promise<SerializedWorldSystems>;
type WorldSectorLoader = (
  sector: WorldSector,
) => Promise<SerializedWorldSystems>;
export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';
export type SpaceshipProximityTelemetry = {
  bodyName: string;
  bodyKind: 'Planet' | 'Star';
  surfaceDistanceMeters: number;
  relativeSpeedMetersPerSecond: number;
};
type SpaceshipClearance = {
  bodyName: string;
  surfaceDistanceMeters: number;
  minimumSurfaceDistanceMeters: number;
};
export type Inventory = SpaceshipInventory;
export const INVENTORY_MATERIALS = [
  'iron',
  'silicates',
  'ice',
  'silver',
  'carbon',
  'gold',
  'hydrogen',
  'nitrogen',
] as const satisfies readonly InventoryMaterial[];

export const INITIAL_SPACESHIP_FUEL_KNS = 1_000_000;
export const MAX_HULL_DURABILITY = 200;
export const HULL_DURABILITY_PER_LEVEL = 50;
export const HULL_DURABILITY_DRAIN_PER_CRASH = 25;
export const HULL_DURABILITY_CONFIG = {
  baseDurability: MAX_HULL_DURABILITY,
  durabilityPerLevel: HULL_DURABILITY_PER_LEVEL,
  usageDrainRatePerCrash: HULL_DURABILITY_DRAIN_PER_CRASH,
} as const;
export const MAX_THRUSTER_DURABILITY = 100;
export const SPACESHIP_THRUSTER_COUNT = 4;
export const SPACESHIP_INVENTORY_CAPACITY_KG = 5_000;
const EARTH_NAME = 'Earth';
const EARTH_RADIUS_METERS = 6_371_000;
const SPACESHIP_RADIUS_METERS = 200;
const DEFAULT_SURFACE_OFFSET = EARTH_RADIUS_METERS + SPACESHIP_RADIUS_METERS;
const PROXIMITY_TELEMETRY_RANGE_METERS = 3_000_000;
const FREE_FLIGHT_BODY_RADIUS_CLEARANCE_RATIO = 0.2;
const DEFAULT_API_BASE_URL = 'http://localhost:3000';
export const WORLD_VIEWPORT_REFRESH_INTERVAL_MS = 5 * 60 * 1_000;
const MIN_RENDER_SHAPE_SCREEN_WIDTH = 16;
const MIN_RENDER_NAME_TO_SHAPE_ZOOM_RATIO = 0.01;
const TARGET_VELOCITY_TOLERANCE_METERS_PER_SECOND = 0.1;
const THRUSTER_SIGNAL_POWER_TOLERANCE_PERCENT = 0.01;
const SIMULATION_TELEMETRY_UPDATE_INTERVAL_MS = 250;
const PLANET_COLORS = [
  0x60a5fa, 0x34d399, 0xf59e0b, 0xf97316, 0xa78bfa, 0x94a3b8, 0x22d3ee,
  0xf472b6,
];
const STAR_COLORS = [0xfef08a, 0xfdba74, 0x93c5fd, 0xfca5a5];

const store = getDefaultStore();

const spaceshipSpeedAtom = atom(0);
const spaceshipAbsoluteSpeedAtom = atom(0);
const spaceshipTargetDirectionAtom = atom<number | undefined>(undefined);
const spaceshipFuelKnsAtom = atom(INITIAL_SPACESHIP_FUEL_KNS);
const spaceshipHullDurabilityAtom = atom(MAX_HULL_DURABILITY);
const spaceshipHullLevelAtom = atom(1);
const spaceshipThrusterDurabilityAtom = atom<number[]>(
  Array(SPACESHIP_THRUSTER_COUNT).fill(MAX_THRUSTER_DURABILITY),
);
const spaceshipActiveThrustersAtom = atom<
  { powerPercent: number; active: boolean }[]
>(createInactiveThrusters());
const spaceshipMotionStateAtom = atom<SpaceshipMotionState>('landed');
const spaceshipActiveFeatureAtom = atom<SpaceshipActiveFeature | undefined>(
  undefined,
);
const inventoryAtom = atom<Inventory>(createEmptyInventory());
let inventoryPersistHandler: ((inventory: Inventory) => void) | undefined;

function createEmptyInventory(): Inventory {
  return Object.fromEntries(
    INVENTORY_MATERIALS.map((material) => [material, 0]),
  ) as Inventory;
}

function normalizeInventory(inventory?: Partial<Inventory>): Inventory {
  return Object.fromEntries(
    INVENTORY_MATERIALS.map((material) => [
      material,
      inventory?.[material] ?? 0,
    ]),
  ) as Inventory;
}

function createInactiveThrusters() {
  return Array.from({ length: SPACESHIP_THRUSTER_COUNT }, () => ({
    powerPercent: 0,
    active: false,
  }));
}

function thrusterSignalsAreEqual(
  current: { powerPercent: number; active: boolean }[],
  next: { powerPercent: number; active: boolean }[],
) {
  if (current.length !== next.length) return false;

  return next.every((nextThruster, index) => {
    const currentThruster = current[index];
    return (
      currentThruster?.active === nextThruster.active &&
      Math.abs(currentThruster.powerPercent - nextThruster.powerPercent) <=
        THRUSTER_SIGNAL_POWER_TOLERANCE_PERCENT
    );
  });
}

function setSpaceshipActiveThrusterSignals(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  const nextThrusters = normalizeThrusterSignals(thrusters);
  const currentThrusters = store.get(spaceshipActiveThrustersAtom);
  if (thrusterSignalsAreEqual(currentThrusters, nextThrusters)) return;

  store.set(spaceshipActiveThrustersAtom, nextThrusters);
}

function persistInventory(inventory: Inventory) {
  inventoryPersistHandler?.(inventory);
}

export function setInventoryPersistHandler(
  handler?: (inventory: Inventory) => void,
) {
  inventoryPersistHandler = handler;
}

export function useSpaceshipSpeed() {
  return useAtomValue(spaceshipSpeedAtom);
}

export function useSetSpaceshipSpeed() {
  return useSetAtom(spaceshipSpeedAtom);
}

export function useSpaceshipAbsoluteSpeed() {
  return useAtomValue(spaceshipAbsoluteSpeedAtom);
}

export function useSetSpaceshipAbsoluteSpeed() {
  return useSetAtom(spaceshipAbsoluteSpeedAtom);
}

export function useSpaceshipTargetDirection() {
  return useAtomValue(spaceshipTargetDirectionAtom);
}

export function useSetSpaceshipTargetDirection() {
  return useSetAtom(spaceshipTargetDirectionAtom);
}

export function setSpaceshipTargetDirection(direction?: number) {
  store.set(spaceshipTargetDirectionAtom, direction);
}

export function useSpaceshipFuelKns() {
  return useAtomValue(spaceshipFuelKnsAtom);
}

export function useSetSpaceshipFuelKns() {
  return useSetAtom(spaceshipFuelKnsAtom);
}

export function getSpaceshipFuelKns() {
  return store.get(spaceshipFuelKnsAtom);
}

export function useSpaceshipHullDurability() {
  return useAtomValue(spaceshipHullDurabilityAtom);
}

export function useSetSpaceshipHullDurability() {
  return useSetAtom(spaceshipHullDurabilityAtom);
}

export function useSpaceshipHullLevel() {
  return useAtomValue(spaceshipHullLevelAtom);
}

export function useSetSpaceshipHullLevel() {
  return useSetAtom(spaceshipHullLevelAtom);
}

export function getSpaceshipHullLevel() {
  return store.get(spaceshipHullLevelAtom);
}

export function getSpaceshipMaxHullDurability(level = getSpaceshipHullLevel()) {
  return (
    MAX_HULL_DURABILITY + Math.max(0, level - 1) * HULL_DURABILITY_PER_LEVEL
  );
}

export function useSpaceshipThrusterDurability() {
  return useAtomValue(spaceshipThrusterDurabilityAtom);
}

export function useSetSpaceshipThrusterDurability() {
  return useSetAtom(spaceshipThrusterDurabilityAtom);
}

export function repairSpaceshipHull() {
  store.set(spaceshipHullDurabilityAtom, getSpaceshipMaxHullDurability());
}

export function repairSpaceshipHullByAmount(amount: number) {
  if (!Number.isFinite(amount) || amount <= 0) return false;

  const currentDurability = store.get(spaceshipHullDurabilityAtom);
  const nextDurability = Math.min(
    getSpaceshipMaxHullDurability(),
    currentDurability + amount,
  );
  if (nextDurability <= currentDurability) return false;

  store.set(spaceshipHullDurabilityAtom, nextDurability);
  return true;
}

export function upgradeSpaceshipHull() {
  const currentLevel = store.get(spaceshipHullLevelAtom);
  store.set(spaceshipHullLevelAtom, currentLevel + 1);
  store.set(
    spaceshipHullDurabilityAtom,
    store.get(spaceshipHullDurabilityAtom) + HULL_DURABILITY_PER_LEVEL,
  );
}

function drainSpaceshipHull(amount: number) {
  if (!Number.isFinite(amount) || amount <= 0) return;

  store.set(
    spaceshipHullDurabilityAtom,
    Math.max(0, store.get(spaceshipHullDurabilityAtom) - amount),
  );
}

export function repairSpaceshipThruster(index: number) {
  if (
    !Number.isInteger(index) ||
    index < 0 ||
    index >= SPACESHIP_THRUSTER_COUNT
  ) {
    return false;
  }

  store.set(
    spaceshipThrusterDurabilityAtom,
    store
      .get(spaceshipThrusterDurabilityAtom)
      .map((durability, thrusterIndex) =>
        thrusterIndex === index ? MAX_THRUSTER_DURABILITY : durability,
      ),
  );
  return true;
}

export function repairSpaceshipThrusterByAmount(index: number, amount: number) {
  if (
    !Number.isInteger(index) ||
    index < 0 ||
    index >= SPACESHIP_THRUSTER_COUNT ||
    !Number.isFinite(amount) ||
    amount <= 0
  ) {
    return false;
  }

  let repaired = false;
  store.set(
    spaceshipThrusterDurabilityAtom,
    store
      .get(spaceshipThrusterDurabilityAtom)
      .map((durability, thrusterIndex) => {
        if (thrusterIndex !== index) return durability;

        const nextDurability = Math.min(
          MAX_THRUSTER_DURABILITY,
          durability + amount,
        );
        repaired = nextDurability > durability;
        return nextDurability;
      }),
  );
  return repaired;
}

export function addSpaceshipFuelKns(amount: number) {
  if (!Number.isFinite(amount) || amount <= 0) return false;

  store.set(spaceshipFuelKnsAtom, store.get(spaceshipFuelKnsAtom) + amount);
  return true;
}

export function useSpaceshipActiveThrusters() {
  return useAtomValue(spaceshipActiveThrustersAtom);
}

export function useSetSpaceshipActiveThrusters() {
  return useSetAtom(spaceshipActiveThrustersAtom);
}

export function useSpaceshipMotionState() {
  return useAtomValue(spaceshipMotionStateAtom);
}

export function useSetSpaceshipMotionState() {
  return useSetAtom(spaceshipMotionStateAtom);
}

export function useSpaceshipActiveFeature() {
  return useAtomValue(spaceshipActiveFeatureAtom);
}

export function useSetSpaceshipActiveFeature() {
  return useSetAtom(spaceshipActiveFeatureAtom);
}

export function useInventory() {
  return useAtomValue(inventoryAtom);
}

export function useSetInventory() {
  return useSetAtom(inventoryAtom);
}

export function spendInventory(cost: Partial<Inventory>) {
  const inventory = store.get(inventoryAtom);
  const entries = Object.entries(cost) as [InventoryMaterial, number][];
  const canSpend = entries.every(
    ([material, amount]) => inventory[material] >= amount,
  );
  if (!canSpend) return false;

  const nextInventory = Object.fromEntries(
    INVENTORY_MATERIALS.map((material) => [
      material,
      inventory[material] - (cost[material] ?? 0),
    ]),
  ) as Inventory;
  store.set(inventoryAtom, nextInventory);
  persistInventory(nextInventory);
  return true;
}

export function addInventory(deposit: Partial<Inventory>) {
  const inventory = store.get(inventoryAtom);
  const nextInventory = Object.fromEntries(
    INVENTORY_MATERIALS.map((material) => [
      material,
      inventory[material] + Math.max(0, deposit[material] ?? 0),
    ]),
  ) as Inventory;

  store.set(inventoryAtom, nextInventory);
  persistInventory(nextInventory);
}

export function getInventoryMassKg(inventory = store.get(inventoryAtom)) {
  return INVENTORY_MATERIALS.reduce(
    (total, material) => total + inventory[material],
    0,
  );
}

export function getSpaceshipMotionState() {
  return store.get(spaceshipMotionStateAtom);
}

export function getSpaceshipAttachedBodyName() {
  return spaceshipAttachedBodyName;
}

export const worldState: World = {
  planets: [],
  stars: [],
};

export const spaceshipState: Spaceship = {
  position: {
    x: BigInt(DEFAULT_SURFACE_OFFSET),
    y: 0n,
    relativeTo: EARTH_NAME,
  },
  name: 'Spaceship',
  radius: BigInt(SPACESHIP_RADIUS_METERS),
  mass: BigInt(SPACESHIP_MASS_KG),
  orbitalCenter: null,
  clockwise: false,
  speed: 0n,
  heading: 0,
};

const listeners = new Set<WorldListener>();
let loadPromise: Promise<World> | undefined;
let worldBodyByName = new Map<string, Body>();
let worldPlanetNames = new Set<string>();
let bodyVelocityByName = new Map<string, Vector>();
let bodyOrbitEpochByName = new Map<string, Planet | Star>();
let spaceshipVelocity: Vector | undefined;
let spaceshipPositionRemainder: Vector = { x: 0, y: 0 };
let spaceshipStoredRelativeVelocity: Vector | undefined;
let spaceshipAttachedBodyName: string | undefined = EARTH_NAME;
let worldElapsedSeconds = 0;
let worldViewportLoader: WorldViewportLoader | undefined;
let worldSectorLoader: WorldSectorLoader | undefined;
let activeWorldBodyNames: ReadonlySet<string> | undefined;
let activeWorldBodies: (Planet | Star)[] | undefined;
let simulationWorker: Worker | undefined;
let simulationWorkerFailed = false;
let simulationAdvancePending = false;
let simulationAdvanceRequestId = 0;
let simulationRequestId = 0;
let lastSimulationTelemetryAtomUpdateMs = 0;
let lastDurabilityDrainElapsedSeconds = 0;
const pendingSpaceshipRequests = new Map<
  number,
  {
    resolve: (spaceship: SpaceshipDto) => void;
    reject: (error: Error) => void;
  }
>();
let latestSimulationSnapshot:
  | Pick<
      SimulationFrameSnapshot,
      'proximityTelemetry' | 'activeThrustVector' | 'activeThrusters'
    >
  | undefined;
const worldSystemsBySectorKey = new Map<string, SerializedWorldSystems>();
const worldReferenceSystemsByBodyName = new Map<
  string,
  SerializedWorldSystems
>();
let fetchedWorldSectorKeys = new Set<string>();

const isBrowserMainThread =
  typeof window !== 'undefined' && typeof Worker !== 'undefined';

type WorldViewportRequest = {
  x1: string;
  y1: string;
  x2: string;
  y2: string;
  zoom?: number;
  requiredBodyNames?: string[];
  signal?: AbortSignal;
};

export function setWorldViewportLoader(loader?: WorldViewportLoader) {
  worldViewportLoader = loader;
}

export function setWorldSectorLoader(loader?: WorldSectorLoader) {
  worldSectorLoader = loader;
}

function getSimulationWorker() {
  if (!isBrowserMainThread || simulationWorkerFailed) return undefined;
  if (simulationWorker) return simulationWorker;

  try {
    simulationWorker = new Worker(
      new URL('./world-simulation.worker.ts', import.meta.url),
      { type: 'module' },
    );
    simulationWorker.onmessage = (
      event: MessageEvent<SimulationWorkerResponse>,
    ) => {
      const message = event.data;
      if (message.type === 'error') {
        console.error('World simulation worker error', message.message);
        rejectPendingSimulationRequests(new Error(message.message));
        simulationWorkerFailed = true;
        simulationAdvancePending = false;
        simulationWorker?.terminate();
        simulationWorker = undefined;
        return;
      }

      if (message.type === 'spaceship') {
        hydrateSpaceship(message.spaceship, false);
        applySimulationFrameSnapshot(message.snapshot);
        const pending = pendingSpaceshipRequests.get(message.requestId);
        pendingSpaceshipRequests.delete(message.requestId);
        pending?.resolve(message.spaceship);
        return;
      }

      applySimulationFrameSnapshot(message.snapshot);
      if (
        message.requestId !== undefined &&
        message.requestId === simulationAdvanceRequestId
      ) {
        simulationAdvancePending = false;
      }
    };
    simulationWorker.onerror = (event) => {
      console.error('World simulation worker failed', event.message);
      rejectPendingSimulationRequests(new Error(event.message));
      simulationWorkerFailed = true;
      simulationAdvancePending = false;
      simulationWorker?.terminate();
      simulationWorker = undefined;
    };
  } catch (error) {
    console.error('Failed to start world simulation worker', error);
    simulationWorkerFailed = true;
  }

  return simulationWorker;
}

function postSimulationMessage(message: SimulationWorkerRequest) {
  getSimulationWorker()?.postMessage(message);
}

function invalidateLatestSimulationSnapshot() {
  latestSimulationSnapshot = undefined;
}

function rejectPendingSimulationRequests(error: Error) {
  pendingSpaceshipRequests.forEach((pending) => pending.reject(error));
  pendingSpaceshipRequests.clear();
}

function shouldPublishSimulationTelemetry(snapshot: SimulationFrameSnapshot) {
  const currentFeature = store.get(spaceshipActiveFeatureAtom);
  if (currentFeature?.type !== snapshot.activeFeature?.type) return true;
  if (store.get(spaceshipMotionStateAtom) !== snapshot.motionState) {
    return true;
  }
  if (
    !thrusterSignalsAreEqual(
      store.get(spaceshipActiveThrustersAtom),
      snapshot.activeThrusters,
    )
  ) {
    return true;
  }

  const now = Date.now();
  if (
    now - lastSimulationTelemetryAtomUpdateMs >=
    SIMULATION_TELEMETRY_UPDATE_INTERVAL_MS
  ) {
    return true;
  }

  return false;
}

function publishSimulationTelemetryAtoms(snapshot: SimulationFrameSnapshot) {
  lastSimulationTelemetryAtomUpdateMs = Date.now();
  store.set(spaceshipMotionStateAtom, snapshot.motionState);
  store.set(spaceshipActiveFeatureAtom, snapshot.activeFeature);
  setSpaceshipActiveThrusterSignals(snapshot.activeThrusters);
  store.set(spaceshipSpeedAtom, Number(snapshot.spaceship.speed));
  store.set(spaceshipAbsoluteSpeedAtom, snapshot.absoluteSpeed);
}

export async function loadWorld(request: WorldViewportRequest) {
  if (loadPromise) return loadPromise;

  loadPromise = initializeWorldSectors(request).catch((error: unknown) => {
    loadPromise = undefined;
    throw error;
  });

  return loadPromise;
}

async function initializeWorldSectors(request: WorldViewportRequest) {
  await loadCachedWorldSectors();
  await hydrateSpaceshipPositionReferences();
  await scanWorldSector(getSpaceshipWorldSector());

  if (worldSystemsBySectorKey.size === 0) {
    return refreshWorldViewport(request);
  }

  return worldState;
}

export async function loadCachedWorldSectors() {
  const [cachedSectors, fetchedSectorKeys] = await Promise.all([
    readCachedWorldSectors(),
    readFetchedWorldSectorKeys(),
  ]);
  fetchedWorldSectorKeys = fetchedSectorKeys;
  cachedSectors.forEach((sector) => {
    worldSystemsBySectorKey.set(getWorldSectorKey(sector), sector.systems);
  });

  if (cachedSectors.length > 0) {
    applyWorldSystems(getCombinedWorldSystems());
  }

  return worldState;
}

export async function scanWorldSector(
  sector: WorldSector,
  signal?: AbortSignal,
) {
  const key = getWorldSectorKey(sector);
  if (
    worldSystemsBySectorKey.has(key) &&
    fetchedWorldSectorKeys.has(key) &&
    hasLoadedBlackHole()
  ) {
    return worldState;
  }

  const systems = worldSectorLoader
    ? await worldSectorLoader(sector)
    : await loadWorldSectorFromRest(sector, signal);

  if (signal?.aborted) return worldState;

  worldSystemsBySectorKey.set(key, systems);
  await Promise.all([
    writeCachedWorldSector(sector, systems),
    markWorldSectorFetched(sector),
  ]);
  fetchedWorldSectorKeys.add(key);
  applyWorldSystems(getCombinedWorldSystems());
  return worldState;
}

export function getLoadedWorldSectorKeys() {
  return new Set(worldSystemsBySectorKey.keys());
}

export function getSpaceshipWorldSector() {
  const position = getWorldPosition(spaceshipState.position);
  return getWorldSector({ x: position.x, y: position.y });
}

export async function refreshWorldViewport({
  x1,
  y1,
  x2,
  y2,
  zoom,
  requiredBodyNames,
  signal,
}: WorldViewportRequest) {
  const request = {
    x1,
    y1,
    x2,
    y2,
    ...(zoom === undefined ? {} : { zoom }),
    ...(requiredBodyNames === undefined ? {} : { requiredBodyNames }),
  };
  const data = worldViewportLoader
    ? await worldViewportLoader(request)
    : await loadWorldViewportFromRest({ ...request, signal });

  applyWorldSystems(data);
  return worldState;
}

export function initializeSpaceshipInSimulation(
  request:
    | { type: 'new' }
    | { type: 'continue'; securityCode: string }
    | { type: 'claim'; securityCode: string },
) {
  const worker = getSimulationWorker();
  if (!worker) return undefined;

  const requestId = ++simulationRequestId;
  const promise = new Promise<SpaceshipDto>((resolve, reject) => {
    pendingSpaceshipRequests.set(requestId, { resolve, reject });
  });
  worker.postMessage({
    type: 'initialize-spaceship',
    requestId,
    request,
  } satisfies SimulationWorkerRequest);
  return promise;
}

async function loadWorldViewportFromRest({
  x1,
  y1,
  x2,
  y2,
  zoom,
  requiredBodyNames,
  signal,
}: WorldViewportRequest) {
  const apiBaseUrl = (
    import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL
  ).replace(/\/+$/, '');

  return axios
    .post<SerializedWorldSystems>(
      `${apiBaseUrl}/world/systems`,
      {
        x1,
        y1,
        x2,
        y2,
        ...(zoom === undefined ? {} : { zoom }),
        ...(requiredBodyNames === undefined ? {} : { requiredBodyNames }),
      },
      { signal },
    )
    .then(({ data }) => data);
}

async function loadWorldSectorFromRest(
  sector: WorldSector,
  signal?: AbortSignal,
) {
  const apiBaseUrl = (
    import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL
  ).replace(/\/+$/, '');

  return axios
    .post<SerializedWorldSystems>(
      `${apiBaseUrl}/world/systems`,
      {
        sectorX: sector.x,
        sectorY: sector.y,
      },
      { signal },
    )
    .then(({ data }) => data);
}

async function hydrateSpaceshipPositionReferences() {
  const referenceName = spaceshipState.position.relativeTo;
  if (!referenceName || getBodyByName(referenceName)) return;

  const systems = await loadWorldReferenceBodiesFromRest([referenceName]);
  worldReferenceSystemsByBodyName.set(referenceName, systems);
  applyWorldSystems(getCombinedWorldSystems());
}

async function loadWorldReferenceBodiesFromRest(requiredBodyNames: string[]) {
  const apiBaseUrl = (
    import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL
  ).replace(/\/+$/, '');

  return axios
    .post<SerializedWorldSystems>(`${apiBaseUrl}/world/systems`, {
      x1: '0',
      y1: '0',
      x2: '0',
      y2: '0',
      requiredBodyNames,
    })
    .then(({ data }) => data);
}

export function subscribeToWorld(listener: WorldListener) {
  listeners.add(listener);

  return () => {
    listeners.delete(listener);
  };
}

export function getSimulationFrameSnapshot(): SimulationFrameSnapshot {
  const velocity = getSpaceshipWorldVelocity();
  const activeThrusters = getSpaceshipActiveThrusters();
  const activeThrustVector = getSpaceshipActiveThrustVector();

  return {
    elapsedSeconds: worldElapsedSeconds,
    bodyPositions: getActiveWorldBodies().map((body) => ({
      name: body.name,
      x: body.position.x,
      y: body.position.y,
    })),
    spaceship: {
      x: spaceshipState.position.x,
      y: spaceshipState.position.y,
      relativeTo: spaceshipState.position.relativeTo,
      heading: spaceshipState.heading,
      speed: spaceshipState.speed,
      velocity,
      attachedBodyName: spaceshipAttachedBodyName,
    },
    motionState: store.get(spaceshipMotionStateAtom),
    activeFeature: store.get(spaceshipActiveFeatureAtom),
    absoluteSpeed: Math.hypot(velocity.x, velocity.y),
    proximityTelemetry: getSpaceshipProximityTelemetry(),
    activeThrustVector,
    activeThrusters,
  };
}

function applySimulationFrameSnapshot(snapshot: SimulationFrameSnapshot) {
  const currentMotionState = store.get(spaceshipMotionStateAtom);
  if (currentMotionState !== 'crashed' && snapshot.motionState === 'crashed') {
    drainSpaceshipHull(HULL_DURABILITY_DRAIN_PER_CRASH);
  }

  const durabilityDrainSeconds = Math.max(
    0,
    snapshot.elapsedSeconds - lastDurabilityDrainElapsedSeconds,
  );
  if (durabilityDrainSeconds > 0) {
    drainActiveThrusterDurability(
      snapshot.activeThrusters,
      durabilityDrainSeconds,
    );
    lastDurabilityDrainElapsedSeconds = snapshot.elapsedSeconds;
  }

  worldElapsedSeconds = snapshot.elapsedSeconds;
  latestSimulationSnapshot = {
    proximityTelemetry: snapshot.proximityTelemetry,
    activeThrustVector: snapshot.activeThrustVector,
    activeThrusters: snapshot.activeThrusters,
  };

  const changedBodyNames = new Set<string>();
  snapshot.bodyPositions.forEach(({ name, x, y }) => {
    const body = getBodyByName(name);
    if (!body) return;
    if (body.position.x === x && body.position.y === y) return;

    body.position.x = x;
    body.position.y = y;
    changedBodyNames.add(name);
  });

  if (
    spaceshipState.position.x !== snapshot.spaceship.x ||
    spaceshipState.position.y !== snapshot.spaceship.y ||
    spaceshipState.position.relativeTo !== snapshot.spaceship.relativeTo ||
    spaceshipState.heading !== snapshot.spaceship.heading ||
    spaceshipState.speed !== snapshot.spaceship.speed
  ) {
    spaceshipState.position = {
      x: snapshot.spaceship.x,
      y: snapshot.spaceship.y,
      relativeTo: snapshot.spaceship.relativeTo,
    };
    spaceshipState.heading = snapshot.spaceship.heading;
    spaceshipState.speed = snapshot.spaceship.speed;
    changedBodyNames.add(spaceshipState.name);
  }

  spaceshipVelocity = snapshot.spaceship.velocity;
  spaceshipAttachedBodyName = snapshot.spaceship.attachedBodyName;
  if (shouldPublishSimulationTelemetry(snapshot)) {
    publishSimulationTelemetryAtoms(snapshot);
  }

  if (changedBodyNames.size > 0) {
    listeners.forEach((listener) => listener(worldState, changedBodyNames));
  }
}

function applyWorldSystems(data: SerializedWorldSystems, syncWorker = true) {
  invalidateLatestSimulationSnapshot();
  const nextVelocities = new Map<string, Vector>();
  const bodies = data.systems.flat();
  const stars = bodies.flatMap((body) => {
    if (body.type !== 'star') return [];
    if (body.velocity) nextVelocities.set(body.name, body.velocity);
    return deserializeBody<Star>(body, getStarVisualDefaults(body.name));
  });
  const planets = bodies.flatMap((body) => {
    if (!isVisiblePlanetBody(body)) return [];

    if (body.velocity) nextVelocities.set(body.name, body.velocity);
    return deserializeBody<Planet>(body, getPlanetVisualDefaults(body.name));
  });

  worldState.stars = stars;
  worldState.planets = planets;
  bodyVelocityByName = nextVelocities;
  bodyOrbitEpochByName = new Map(
    [...stars, ...planets].map((body) => [
      body.name,
      cloneBodyOrbitEpoch(body),
    ]),
  );
  [...stars, ...planets].forEach((body) => {
    advanceBodyPositionToNow(body);
  });
  rebuildWorldBodyByName();
  rebuildActiveWorldBodies();
  normalizeAttachedSpaceshipPosition();
  syncSpaceshipAbsoluteSpeed();
  listeners.forEach((listener) => listener(worldState));
  if (syncWorker) {
    postSimulationMessage({ type: 'hydrate-world', systems: data });
  }
}

function getCombinedWorldSystems(): SerializedWorldSystems {
  return {
    systems: [
      ...worldReferenceSystemsByBodyName.values(),
      ...worldSystemsBySectorKey.values(),
    ].flatMap((systems) => systems.systems),
  };
}

function hasLoadedBlackHole() {
  return worldState.planets.some((planet) => planet.type === 'blackhole');
}

export function hydrateWorldSystems(data: SerializedWorldSystems) {
  applyWorldSystems(data);
  return worldState;
}

function isVisiblePlanetBody(
  body: SerializedWorldBody,
): body is SerializedVisiblePlanetBody {
  return (
    body.type === 'planet' || body.type === 'moon' || body.type === 'blackhole'
  );
}

export function setActiveWorldBodyNames(names?: Iterable<string>) {
  if (!names) {
    activeWorldBodyNames = undefined;
    activeWorldBodies = undefined;
    postSimulationMessage({ type: 'set-active-bodies' });
    return;
  }

  const nextNames = new Set<string>();
  for (const name of names) {
    addActiveWorldBodyName(name, nextNames);
  }
  activeWorldBodyNames = nextNames;
  rebuildActiveWorldBodies();
  postSimulationMessage({
    type: 'set-active-bodies',
    names: [...nextNames],
  });
}

function addActiveWorldBodyName(name: string, names: Set<string>) {
  if (names.has(name)) return;

  names.add(name);
  const body = getWorldBodyByName().get(name);
  const centerName = body?.orbitalCenter ?? body?.position.relativeTo;
  if (centerName) addActiveWorldBodyName(centerName, names);
}

function rebuildActiveWorldBodies() {
  const activeNames = activeWorldBodyNames;
  if (!activeNames) {
    activeWorldBodies = undefined;
    return;
  }

  activeWorldBodies = [...worldState.stars, ...worldState.planets].filter(
    (body) => activeNames.has(body.name),
  );
}

export function setSpaceshipHeading(heading: number) {
  invalidateLatestSimulationSnapshot();
  spaceshipState.heading = heading;
  listeners.forEach((listener) => listener(worldState));
  postSimulationMessage({ type: 'set-heading', heading });
}

export function startSpaceshipThrusters(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  invalidateLatestSimulationSnapshot();
  if (!Array.isArray(thrusters) || thrusters.length === 0) return false;
  if (store.get(spaceshipMotionStateAtom) === 'crashed') return false;
  if (!canDetachSpaceshipFromAttachedBody()) return false;

  normalizeAttachedSpaceshipPosition();
  spaceshipVelocity = getInitialSpaceshipWorldVelocity();
  spaceshipAttachedBodyName = undefined;
  store.set(spaceshipMotionStateAtom, 'flying');
  store.set(spaceshipActiveFeatureAtom, {
    type: 'thrusters',
    thrusters: thrusters.map((thruster) => ({
      powerPercent: thruster.powerPercent,
      active: thruster.active,
    })),
    elapsedSeconds: 0,
  });
  syncSpaceshipAbsoluteSpeed();
  syncSpaceshipActiveThrusters();
  listeners.forEach((listener) => listener(worldState));
  postSimulationMessage({ type: 'start-thrusters', thrusters });
  return true;
}

export function startSpaceshipTargetSpeed(
  targetSpeedMetersPerSecond: number,
  maximumThrustPercent: number,
  targetDirection: number | undefined,
) {
  invalidateLatestSimulationSnapshot();
  if (
    store.get(spaceshipMotionStateAtom) === 'crashed' ||
    targetDirection === undefined
  ) {
    return false;
  }
  if (!canDetachSpaceshipFromAttachedBody()) return false;

  const preview = getSpaceshipTargetSpeedBurnPreview(
    targetSpeedMetersPerSecond,
    maximumThrustPercent,
    targetDirection,
  );
  if (!preview) return false;

  normalizeAttachedSpaceshipPosition();
  spaceshipVelocity = getInitialSpaceshipWorldVelocity();
  spaceshipAttachedBodyName = undefined;
  store.set(spaceshipMotionStateAtom, 'flying');
  store.set(spaceshipActiveFeatureAtom, {
    type: 'target-speed',
    targetSpeedMetersPerSecond,
    maximumThrustPercent,
    targetDirection,
    targetVelocity: {
      x: Math.cos(targetDirection) * targetSpeedMetersPerSecond,
      y: Math.sin(targetDirection) * targetSpeedMetersPerSecond,
    },
    maximumAcceleration: preview.maximumAcceleration,
    durationSeconds: preview.durationSeconds,
    elapsedSeconds: 0,
  });
  syncSpaceshipAbsoluteSpeed();
  syncSpaceshipActiveThrusters();
  listeners.forEach((listener) => listener(worldState));
  postSimulationMessage({
    type: 'start-target-speed',
    targetSpeedMetersPerSecond,
    maximumThrustPercent,
    targetDirection,
  });
  return true;
}

export function stopSpaceshipActiveFeatureLocally() {
  invalidateLatestSimulationSnapshot();
  store.set(spaceshipActiveFeatureAtom, undefined);
  setSpaceshipActiveThrusterSignals(createInactiveThrusters());
  listeners.forEach((listener) => listener(worldState));
  postSimulationMessage({ type: 'stop-active-feature' });
}

export function hydrateSpaceship(dto: SpaceshipDto, syncWorker = true) {
  invalidateLatestSimulationSnapshot();
  spaceshipState.position = {
    x: BigInt(dto.position.x),
    y: BigInt(dto.position.y),
    relativeTo: dto.position.relativeTo,
  };
  spaceshipState.positionCapturedAt = dto.positionCapturedAt ?? dto.simulatedAt;
  spaceshipState.heading = dto.direction;
  spaceshipState.speed = BigInt(dto.speed);
  spaceshipState.orbitalCenter = null;
  const motionState =
    dto.motionState ??
    (dto.speed === '0' && dto.position.relativeTo ? 'landed' : 'flying');
  spaceshipVelocity = undefined;
  spaceshipPositionRemainder = { x: 0, y: 0 };
  spaceshipStoredRelativeVelocity =
    motionState === 'flying' && dto.velocity ? { ...dto.velocity } : undefined;
  lastDurabilityDrainElapsedSeconds = worldElapsedSeconds;
  spaceshipAttachedBodyName =
    motionState === 'flying' ? undefined : dto.position.relativeTo;
  store.set(spaceshipMotionStateAtom, motionState);
  store.set(
    spaceshipActiveFeatureAtom,
    normalizeActiveFeature(dto.activeFeature),
  );
  store.set(spaceshipSpeedAtom, Number(dto.speed));
  syncSpaceshipAbsoluteSpeed();
  syncSpaceshipActiveThrusters();
  store.set(
    spaceshipFuelKnsAtom,
    dto.stats?.fuelKns ?? INITIAL_SPACESHIP_FUEL_KNS,
  );
  const hullLevel = Math.max(1, Math.round(dto.stats?.hullLevel ?? 1));
  store.set(spaceshipHullLevelAtom, hullLevel);
  store.set(
    spaceshipHullDurabilityAtom,
    clampDurability(
      dto.stats?.hullDurability,
      getSpaceshipMaxHullDurability(hullLevel),
    ),
  );
  store.set(
    spaceshipThrusterDurabilityAtom,
    Array.from({ length: SPACESHIP_THRUSTER_COUNT }, (_, index) =>
      clampDurability(
        dto.stats?.thrusterDurability?.[index],
        MAX_THRUSTER_DURABILITY,
      ),
    ),
  );
  store.set(inventoryAtom, normalizeInventory(dto.inventory));
  rebuildWorldBodyByName();
  normalizeAttachedSpaceshipPosition();
  advanceSpaceshipToNow(dto.positionCapturedAt ?? dto.simulatedAt);
  listeners.forEach((listener) => listener(worldState));
  if (syncWorker) {
    postSimulationMessage({ type: 'hydrate-spaceship', spaceship: dto });
  }
}

export function getSpaceshipDto(securityCode: string): SpaceshipDto {
  const worldPosition = toVector(getWorldPosition(spaceshipState.position));
  const worldVelocity = spaceshipVelocity ?? getInitialSpaceshipWorldVelocity();
  const reference = getClosestPersistenceReference(worldPosition);
  const position = reference
    ? {
        x: Math.round(worldPosition.x - reference.position.x).toString(),
        y: Math.round(worldPosition.y - reference.position.y).toString(),
        relativeTo: reference.body.name,
      }
    : {
        x: Math.round(worldPosition.x).toString(),
        y: Math.round(worldPosition.y).toString(),
      };
  const relativeVelocity = reference
    ? {
        x: worldVelocity.x - reference.velocity.x,
        y: worldVelocity.y - reference.velocity.y,
      }
    : worldVelocity;
  const speed =
    store.get(spaceshipMotionStateAtom) === 'flying'
      ? Math.hypot(relativeVelocity.x, relativeVelocity.y)
      : 0;
  const direction =
    speed === 0
      ? spaceshipState.heading
      : ((Math.atan2(relativeVelocity.y, relativeVelocity.x) * 180) / Math.PI +
          450) %
        360;

  return {
    securityCode,
    position,
    direction,
    speed: Math.round(speed).toString(),
    velocity: relativeVelocity,
    motionState: store.get(spaceshipMotionStateAtom),
    positionCapturedAt: new Date().toISOString(),
    stats: {
      fuelKns: getSpaceshipFuelKns(),
      hullDurability: store.get(spaceshipHullDurabilityAtom),
      hullLevel: getSpaceshipHullLevel(),
      thrusterDurability: store.get(spaceshipThrusterDurabilityAtom),
    },
    inventory: store.get(inventoryAtom),
    activeFeature: store.get(spaceshipActiveFeatureAtom),
    simulatedAt: new Date().toISOString(),
  };
}

function getClosestPersistenceReference(spaceshipPosition: Vector) {
  let closest:
    | {
        body: Planet | Star;
        position: Vector;
        velocity: Vector;
        surfaceDistance: number;
      }
    | undefined;

  for (const body of getActiveWorldBodies()) {
    const bodyPosition = toVector(getWorldPosition(body.position));
    const centerDistance = Math.hypot(
      spaceshipPosition.x - bodyPosition.x,
      spaceshipPosition.y - bodyPosition.y,
    );
    const surfaceDistance = Math.max(
      0,
      centerDistance - Number(body.radius) - SPACESHIP_RADIUS_METERS,
    );
    if (
      surfaceDistance > PROXIMITY_TELEMETRY_RANGE_METERS ||
      (closest && surfaceDistance >= closest.surfaceDistance)
    ) {
      continue;
    }

    closest = {
      body,
      position: bodyPosition,
      velocity: getCelestialBodyWorldVelocity(body.name, new Set()),
      surfaceDistance,
    };
  }

  return closest;
}

function canDetachSpaceshipFromAttachedBody() {
  const referenceName = spaceshipState.position.relativeTo;
  if (store.get(spaceshipMotionStateAtom) === 'flying' || !referenceName) {
    return true;
  }

  return getBodyByName(referenceName) !== undefined;
}

function normalizeActiveFeature(
  activeFeature: SpaceshipDto['activeFeature'] | unknown,
): SpaceshipActiveFeature | undefined {
  if (
    activeFeature &&
    typeof activeFeature === 'object' &&
    'type' in activeFeature &&
    activeFeature.type === 'lock-on'
  ) {
    return undefined;
  }

  return activeFeature as SpaceshipActiveFeature | undefined;
}

export function isSpaceshipEngineRunning() {
  return store.get(spaceshipActiveFeatureAtom) !== undefined;
}

export function getSpaceshipSaveBlockReason() {
  if (store.get(spaceshipMotionStateAtom) !== 'flying') {
    return 'Ship can be saved only while free flying.';
  }
  if (store.get(spaceshipActiveFeatureAtom)) {
    return 'Turn off all thrusters before saving.';
  }

  const blockedClearance = getClosestBlockingSpaceshipClearance();
  if (blockedClearance) {
    return `Move at least ${Math.round(
      blockedClearance.minimumSurfaceDistanceMeters,
    ).toLocaleString()} m from ${blockedClearance.bodyName}'s surface before saving.`;
  }

  return undefined;
}

function getClosestBlockingSpaceshipClearance():
  | SpaceshipClearance
  | undefined {
  const spaceshipPosition = toVector(getWorldPosition(spaceshipState.position));
  let closest: SpaceshipClearance | undefined;

  for (const body of [...worldState.planets, ...worldState.stars]) {
    const bodyPosition = toVector(getWorldPosition(body.position));
    const centerDistance = Math.hypot(
      spaceshipPosition.x - bodyPosition.x,
      spaceshipPosition.y - bodyPosition.y,
    );
    const surfaceDistance =
      spaceshipAttachedBodyName === body.name &&
      store.get(spaceshipMotionStateAtom) !== 'flying'
        ? 0
        : Math.max(
            0,
            centerDistance -
              Number(body.radius) -
              Number(spaceshipState.radius),
          );
    const minimumSurfaceDistanceMeters =
      Number(body.radius) * FREE_FLIGHT_BODY_RADIUS_CLEARANCE_RATIO;

    if (surfaceDistance >= minimumSurfaceDistanceMeters) continue;
    if (closest && surfaceDistance >= closest.surfaceDistanceMeters) {
      continue;
    }

    closest = {
      bodyName: body.name,
      surfaceDistanceMeters: surfaceDistance,
      minimumSurfaceDistanceMeters,
    };
  }

  return closest;
}

export type SpaceshipTargetSpeedBurnPreview = {
  maximumAcceleration: number;
  durationSeconds: number;
};

export function getSpaceshipTargetSpeedBurnPreview(
  targetSpeedMetersPerSecond: number,
  maximumThrustPercent: number,
  targetDirection: number | undefined,
): SpaceshipTargetSpeedBurnPreview | undefined {
  if (
    store.get(spaceshipActiveFeatureAtom) ||
    store.get(spaceshipMotionStateAtom) === 'crashed' ||
    targetDirection === undefined ||
    !Number.isFinite(targetDirection) ||
    !Number.isFinite(targetSpeedMetersPerSecond) ||
    targetSpeedMetersPerSecond < 0 ||
    !Number.isFinite(maximumThrustPercent) ||
    maximumThrustPercent <= 0 ||
    maximumThrustPercent > 100
  ) {
    return undefined;
  }

  const motion = {
    position: toVector(getWorldPosition(spaceshipState.position)),
    velocity: getSpaceshipWorldVelocity(),
  };
  const targetVelocity = {
    x: Math.cos(targetDirection) * targetSpeedMetersPerSecond,
    y: Math.sin(targetDirection) * targetSpeedMetersPerSecond,
  };
  const maximumAcceleration =
    WorldService.calculateMaximumEngineAcceleration(maximumThrustPercent);
  const durationSeconds = WorldService.calculateTargetSpeedBurnDuration(
    targetVelocity,
    motion.velocity,
    motion.position,
    maximumAcceleration,
    calculateGravityAcceleration,
  );
  if (durationSeconds === undefined || durationSeconds === 0) return undefined;

  const accelerationValue = WorldService.calculateRequiredBurnAcceleration(
    targetVelocity,
    durationSeconds,
    motion.velocity,
    motion.position,
    calculateGravityAcceleration,
  );
  if (!hasAvailableThrusterForAcceleration(accelerationValue)) return undefined;

  return { maximumAcceleration, durationSeconds };
}

export function getSpaceshipActiveThrustVector() {
  if (latestSimulationSnapshot) {
    return latestSimulationSnapshot.activeThrustVector;
  }

  return getThrusterSignalsAcceleration(getSpaceshipActiveThrusters());
}

export function getSpaceshipActiveThrusters() {
  if (latestSimulationSnapshot) {
    return latestSimulationSnapshot.activeThrusters;
  }

  return calculateSpaceshipActiveThrusterSignals({
    position: toVector(getWorldPosition(spaceshipState.position)),
    velocity: getSpaceshipWorldVelocity(),
  });
}

function calculateSpaceshipActiveThrustAcceleration(motion: {
  position: Vector;
  velocity: Vector;
}) {
  return getThrusterSignalsAcceleration(
    calculateSpaceshipActiveThrusterSignals(motion),
  );
}

function calculateSpaceshipActiveThrusterSignals(motion: {
  position: Vector;
  velocity: Vector;
}) {
  const activeFeature = store.get(spaceshipActiveFeatureAtom);
  if (
    activeFeature?.type === 'thrusters' ||
    activeFeature?.type === 'manual-force'
  ) {
    return normalizeThrusterSignals(activeFeature.thrusters);
  }

  if (activeFeature?.type !== 'target-speed') return createInactiveThrusters();

  const remainingSeconds = Math.max(
    activeFeature.durationSeconds - activeFeature.elapsedSeconds,
    WorldService.calculateTargetSpeedBurnDuration(
      activeFeature.targetVelocity,
      motion.velocity,
      motion.position,
      activeFeature.maximumAcceleration,
      calculateGravityAcceleration,
    ) ?? 0,
  );
  if (remainingSeconds <= 0) return createInactiveThrusters();

  return getBurnThrusterSignals(
    activeFeature.targetVelocity,
    remainingSeconds,
    motion,
    activeFeature.maximumThrustPercent,
  );
}

function getBurnThrusterSignals(
  targetVelocity: Vector,
  remainingSeconds: number,
  motion: { position: Vector; velocity: Vector },
  maximumThrustPercent: number,
) {
  const requestedAcceleration = WorldService.calculateRequiredBurnAcceleration(
    targetVelocity,
    remainingSeconds,
    motion.velocity,
    motion.position,
    calculateGravityAcceleration,
  );
  return getAccelerationThrusterSignals(
    requestedAcceleration,
    maximumThrustPercent,
  );
}

function normalizeThrusterSignals(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  return Array.from({ length: SPACESHIP_THRUSTER_COUNT }, (_, index) => {
    const thruster = thrusters[index];
    const powerPercent =
      thruster && Number.isFinite(thruster.powerPercent)
        ? clampPercent(thruster.powerPercent)
        : 0;

    return {
      powerPercent,
      active: Boolean(thruster?.active) && powerPercent > 0,
    };
  });
}

function getAccelerationThrusterSignals(
  acceleration: Vector,
  maximumThrustPercent = 100,
) {
  const thrusters = createInactiveThrusters();
  setAxisThrusterSignal(
    thrusters,
    acceleration.x < 0 ? 1 : 3,
    acceleration.x,
    maximumThrustPercent,
  );
  setAxisThrusterSignal(
    thrusters,
    acceleration.y < 0 ? 2 : 0,
    acceleration.y,
    maximumThrustPercent,
  );
  return thrusters;
}

function setAxisThrusterSignal(
  thrusters: { powerPercent: number; active: boolean }[],
  index: number,
  acceleration: number,
  maximumThrustPercent: number,
) {
  const fullPowerAcceleration =
    WorldService.calculateMaximumEngineAcceleration(100);
  const powerPercent =
    fullPowerAcceleration > 0
      ? (Math.abs(acceleration) / fullPowerAcceleration) * 100
      : 0;
  const clampedPowerPercent = clampPercent(
    Math.min(powerPercent, maximumThrustPercent),
  );
  thrusters[index] = {
    powerPercent: clampedPowerPercent,
    active: clampedPowerPercent > 0,
  };
}

function clampPercent(value: number) {
  return Math.min(100, Math.max(0, value));
}

function getThrusterSignalsAcceleration(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  const acceleration = { x: 0, y: 0 };
  const thrusterDurability = store.get(spaceshipThrusterDurabilityAtom);
  normalizeThrusterSignals(thrusters).forEach((thruster, index) => {
    if (
      !thruster.active ||
      thruster.powerPercent <= 0 ||
      (thrusterDurability[index] ?? 0) <= 0
    ) {
      return;
    }

    const thrustAcceleration = WorldService.calculateMaximumEngineAcceleration(
      thruster.powerPercent,
    );
    if (index === 0) acceleration.y += thrustAcceleration;
    if (index === 1) acceleration.x -= thrustAcceleration;
    if (index === 2) acceleration.y -= thrustAcceleration;
    if (index === 3) acceleration.x += thrustAcceleration;
  });

  return acceleration.x === 0 && acceleration.y === 0
    ? undefined
    : acceleration;
}

function hasAvailableThrusterForAcceleration(accelerationValue: Vector) {
  const thrusterDurability = store.get(spaceshipThrusterDurabilityAtom);
  const xIndex = accelerationValue.x < 0 ? 1 : 3;
  const yIndex = accelerationValue.y < 0 ? 2 : 0;

  return (
    (Math.abs(accelerationValue.x) > 1e-8 &&
      (thrusterDurability[xIndex] ?? 0) > 0) ||
    (Math.abs(accelerationValue.y) > 1e-8 &&
      (thrusterDurability[yIndex] ?? 0) > 0)
  );
}

function drainActiveThrusterDurability(
  thrusters: { powerPercent: number; active: boolean }[],
  elapsedSeconds: number,
) {
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) return;

  const normalizedThrusters = normalizeThrusterSignals(thrusters);
  if (!normalizedThrusters.some((thruster) => thruster.active)) return;

  store.set(
    spaceshipThrusterDurabilityAtom,
    store.get(spaceshipThrusterDurabilityAtom).map((durability, index) => {
      const thruster = normalizedThrusters[index];
      if (!thruster?.active || thruster.powerPercent <= 0 || durability <= 0) {
        return durability;
      }

      const drain =
        THRUSTER_DURABILITY_DRAIN_PER_SECOND *
        (thruster.powerPercent / 100) *
        elapsedSeconds;
      return Math.max(0, durability - drain);
    }),
  );
}

export function getSpaceshipWorldVelocity() {
  if (spaceshipVelocity) return { ...spaceshipVelocity };

  const relativeVelocity =
    spaceshipStoredRelativeVelocity ?? getSpaceshipVelocity();
  const referenceVelocity = getSpaceshipReferenceVelocity();
  return {
    x: referenceVelocity.x + relativeVelocity.x,
    y: referenceVelocity.y + relativeVelocity.y,
  };
}

export function getSpaceshipVelocity() {
  if (spaceshipVelocity) {
    const referenceVelocity = getSpaceshipReferenceVelocity();
    return {
      x: spaceshipVelocity.x - referenceVelocity.x,
      y: spaceshipVelocity.y - referenceVelocity.y,
    };
  }

  const speed = Number(spaceshipState.speed);
  const centerName = spaceshipState.orbitalCenter;
  if (centerName) {
    const relativePosition = getWorldPositionRelativeTo(
      spaceshipState.position,
      centerName,
    );
    const x = Number(relativePosition.x);
    const y = Number(relativePosition.y);
    const radius = Math.hypot(x, y);
    if (radius > 0) {
      const direction = spaceshipState.clockwise ? 1 : -1;
      return {
        x: (direction * -y * speed) / radius,
        y: (direction * x * speed) / radius,
      };
    }
  }

  const headingRadians = (spaceshipState.heading * Math.PI) / 180;
  return {
    x: Math.sin(headingRadians) * speed,
    y: -Math.cos(headingRadians) * speed,
  };
}

export function getSpaceshipProximityTelemetry():
  | SpaceshipProximityTelemetry
  | undefined {
  if (latestSimulationSnapshot) {
    return latestSimulationSnapshot.proximityTelemetry;
  }

  const spaceshipWorldVelocity = getSpaceshipWorldVelocity();
  let closest: SpaceshipProximityTelemetry | undefined;

  for (const body of [...worldState.planets, ...worldState.stars]) {
    const relativePosition = getWorldPositionRelativeTo(
      spaceshipState.position,
      body.name,
    );
    const centerDistance = Math.hypot(
      Number(relativePosition.x),
      Number(relativePosition.y),
    );
    const surfaceDistance =
      spaceshipAttachedBodyName === body.name &&
      store.get(spaceshipMotionStateAtom) !== 'flying'
        ? 0
        : Math.max(
            0,
            centerDistance -
              Number(body.radius) -
              Number(spaceshipState.radius),
          );
    if (
      surfaceDistance >= PROXIMITY_TELEMETRY_RANGE_METERS ||
      (closest && surfaceDistance >= closest.surfaceDistanceMeters)
    ) {
      continue;
    }

    const bodyVelocity = getCelestialBodyWorldVelocity(body.name, new Set());
    closest = {
      bodyName: body.name,
      bodyKind: worldPlanetNames.has(body.name) ? 'Planet' : 'Star',
      surfaceDistanceMeters: surfaceDistance,
      relativeSpeedMetersPerSecond: Math.hypot(
        spaceshipWorldVelocity.x - bodyVelocity.x,
        spaceshipWorldVelocity.y - bodyVelocity.y,
      ),
    };
  }

  return closest;
}

export function advanceWorld(elapsedSeconds: number) {
  const worker = getSimulationWorker();
  if (worker) {
    if (elapsedSeconds > 0 && !simulationAdvancePending) {
      simulationAdvancePending = true;
      simulationAdvanceRequestId += 1;
      worker.postMessage({
        type: 'advance',
        requestId: simulationAdvanceRequestId,
        elapsedSeconds,
      } satisfies SimulationWorkerRequest);
    }
    return worldElapsedSeconds;
  }

  if (elapsedSeconds > 0) {
    worldElapsedSeconds += elapsedSeconds;
    advanceBodyPositions(elapsedSeconds);
    advanceSpaceshipPosition(elapsedSeconds);
  }
  return worldElapsedSeconds;
}

function clampDurability(value: number | undefined, maximum: number) {
  return Number.isFinite(value)
    ? Math.min(maximum, Math.max(0, value as number))
    : maximum;
}

function getInitialSpaceshipWorldVelocity() {
  const relativeVelocity =
    spaceshipStoredRelativeVelocity ?? getSpaceshipVelocity();
  spaceshipStoredRelativeVelocity = undefined;
  const referenceVelocity = getSpaceshipReferenceVelocity();
  return {
    x: referenceVelocity.x + relativeVelocity.x,
    y: referenceVelocity.y + relativeVelocity.y,
  };
}

function advanceBodyPositions(elapsedSeconds: number) {
  for (const body of getActiveWorldBodies()) {
    advanceBodyPositionByOrbit(body, elapsedSeconds);
  }
}

function getActiveWorldBodies() {
  return activeWorldBodies ?? [...worldState.stars, ...worldState.planets];
}

function advanceBodyPositionToNow(body: Planet | Star) {
  const cTime = getSnapshotTimeMs(body.cTime);
  if (!Number.isFinite(cTime)) return;

  const elapsedSeconds = Math.max(0, (Date.now() - cTime) / 1000);
  if (elapsedSeconds <= 0) return;

  advanceBodyPositionByOrbit(body, elapsedSeconds);
  body.cTime = Date.now();
}

function advanceBodyPositionByOrbit(
  body: Planet | Star,
  elapsedSeconds: number,
) {
  const position = WorldService.advanceBodyPosition(body, elapsedSeconds);
  body.position.x = BigInt(position.x);
  body.position.y = BigInt(position.y);
}

function advanceSpaceshipToNow(positionCapturedAt: string | undefined) {
  if (!positionCapturedAt) return;

  const positionCapturedAtMs = Date.parse(positionCapturedAt);
  if (!Number.isFinite(positionCapturedAtMs)) return;

  const elapsedSeconds = Math.max(
    0,
    (Date.now() - positionCapturedAtMs) / 1000,
  );
  if (elapsedSeconds > 0) advanceSpaceshipPosition(elapsedSeconds);
  spaceshipState.positionCapturedAt = new Date().toISOString();
}

function getSnapshotTimeMs(value: number | string | undefined) {
  if (typeof value === 'number') return value;
  if (typeof value === 'string') return Date.parse(value);
  return Number.NaN;
}

function syncSpaceshipAbsoluteSpeed() {
  const velocity = getSpaceshipWorldVelocity();
  store.set(spaceshipAbsoluteSpeedAtom, Math.hypot(velocity.x, velocity.y));
}

function syncSpaceshipActiveThrusters() {
  setSpaceshipActiveThrusterSignals(getSpaceshipActiveThrusters());
}

function advanceSpaceshipPosition(elapsedSeconds: number) {
  const motionState = store.get(spaceshipMotionStateAtom);
  if (motionState !== 'flying') return;

  if (spaceshipState.position.relativeTo) {
    advanceRelativeSpaceshipPosition(
      spaceshipState.position.relativeTo,
      elapsedSeconds,
    );
    advanceActiveFeature(elapsedSeconds);
    syncSpaceshipAbsoluteSpeed();
    syncSpaceshipActiveThrusters();
    return;
  }

  const motion = integrateSpaceshipMotion(
    {
      position: toVector(getWorldPosition(spaceshipState.position)),
      velocity: getSpaceshipWorldVelocity(),
    },
    elapsedSeconds,
  );
  spaceshipVelocity = motion.velocity;
  const referencePosition = spaceshipState.position.relativeTo
    ? toVector(
        getWorldPosition({
          x: 0n,
          y: 0n,
          relativeTo: spaceshipState.position.relativeTo,
        }),
      )
    : { x: 0, y: 0 };
  spaceshipPositionRemainder = advancePositionByVelocity(
    spaceshipState.position,
    {
      x:
        motion.position.x -
        referencePosition.x -
        Number(spaceshipState.position.x),
      y:
        motion.position.y -
        referencePosition.y -
        Number(spaceshipState.position.y),
    },
    1,
    spaceshipPositionRemainder,
  );
  advanceActiveFeature(elapsedSeconds);
  syncSpaceshipAbsoluteSpeed();
  syncSpaceshipActiveThrusters();
}

function advanceRelativeSpaceshipPosition(
  referenceName: string,
  elapsedSeconds: number,
) {
  const referenceVelocity = getCelestialBodyWorldVelocity(
    referenceName,
    new Set(),
  );
  const worldVelocity = getSpaceshipWorldVelocity();
  const relativeMotion = integrateSpaceshipRelativeMotion(
    referenceName,
    {
      position: toVector(spaceshipState.position),
      velocity: {
        x: worldVelocity.x - referenceVelocity.x,
        y: worldVelocity.y - referenceVelocity.y,
      },
    },
    elapsedSeconds,
  );

  spaceshipVelocity = {
    x: referenceVelocity.x + relativeMotion.velocity.x,
    y: referenceVelocity.y + relativeMotion.velocity.y,
  };
  spaceshipPositionRemainder = advancePositionByVelocity(
    spaceshipState.position,
    {
      x: relativeMotion.position.x - Number(spaceshipState.position.x),
      y: relativeMotion.position.y - Number(spaceshipState.position.y),
    },
    1,
    spaceshipPositionRemainder,
  );
}

function integrateSpaceshipMotion(
  motion: { position: Vector; velocity: Vector },
  elapsedSeconds: number,
) {
  if (elapsedSeconds <= 0) return motion;

  const stepCount = Math.min(
    MAX_PROPAGATION_STEPS,
    Math.max(1, Math.ceil(elapsedSeconds / TARGET_STEP_SECONDS)),
  );
  const stepSeconds = elapsedSeconds / stepCount;
  let nextMotion = motion;

  for (let step = 0; step < stepCount; step += 1) {
    nextMotion = integrateSpaceshipStep(nextMotion, stepSeconds);
  }

  return nextMotion;
}

function integrateSpaceshipRelativeMotion(
  referenceName: string,
  motion: { position: Vector; velocity: Vector },
  elapsedSeconds: number,
) {
  if (elapsedSeconds <= 0) return motion;

  const stepCount = Math.min(
    MAX_PROPAGATION_STEPS,
    Math.max(1, Math.ceil(elapsedSeconds / TARGET_STEP_SECONDS)),
  );
  const stepSeconds = elapsedSeconds / stepCount;
  let nextMotion = motion;

  for (let step = 0; step < stepCount; step += 1) {
    const referencePosition = toVector(
      getWorldPosition({ x: 0n, y: 0n, relativeTo: referenceName }),
    );
    const referenceVelocity = getCelestialBodyWorldVelocity(
      referenceName,
      new Set(),
    );
    const thrustAcceleration = calculateSpaceshipActiveThrustAcceleration({
      position: {
        x: referencePosition.x + nextMotion.position.x,
        y: referencePosition.y + nextMotion.position.y,
      },
      velocity: {
        x: referenceVelocity.x + nextMotion.velocity.x,
        y: referenceVelocity.y + nextMotion.velocity.y,
      },
    });
    nextMotion = WorldService.integrateStep(
      nextMotion,
      stepSeconds,
      (position) =>
        WorldService.calculateAcceleration(
          position,
          (nextPosition) =>
            calculateReferenceGravityAcceleration(nextPosition, referenceName),
          thrustAcceleration,
        ),
    );
  }

  return nextMotion;
}

function integrateSpaceshipStep(
  motion: { position: Vector; velocity: Vector },
  seconds: number,
) {
  const thrustAcceleration = calculateSpaceshipActiveThrustAcceleration(motion);
  return WorldService.integrateStep(motion, seconds, (position) =>
    calculateAcceleration(position, thrustAcceleration),
  );
}

function calculateAcceleration(position: Vector, thrustAcceleration?: Vector) {
  return WorldService.calculateAcceleration(
    position,
    calculateGravityAcceleration,
    thrustAcceleration,
  );
}

function calculateGravityAcceleration(position: Vector) {
  return WorldService.calculateGravityAcceleration(
    position,
    getActiveWorldBodies(),
    (body) => toVector(getWorldPosition(body.position)),
  );
}

function calculateReferenceGravityAcceleration(
  position: Vector,
  referenceName: string,
) {
  const reference = getBodyByName(referenceName);
  if (!reference) return { x: 0, y: 0 };

  return WorldService.calculateGravityAcceleration(
    position,
    [reference],
    () => ({ x: 0, y: 0 }),
  );
}

function advanceActiveFeature(elapsedSeconds: number) {
  const activeFeature = store.get(spaceshipActiveFeatureAtom);
  if (
    activeFeature?.type === 'thrusters' ||
    activeFeature?.type === 'manual-force'
  ) {
    drainActiveThrusterDurability(activeFeature.thrusters, elapsedSeconds);
    const nextElapsedSeconds = activeFeature.elapsedSeconds + elapsedSeconds;
    store.set(spaceshipActiveFeatureAtom, {
      ...activeFeature,
      elapsedSeconds: nextElapsedSeconds,
    });
    return;
  }

  if (activeFeature?.type !== 'target-speed') return;

  const motion = {
    position: toVector(getWorldPosition(spaceshipState.position)),
    velocity: getSpaceshipWorldVelocity(),
  };
  const velocityError = Math.hypot(
    activeFeature.targetVelocity.x - motion.velocity.x,
    activeFeature.targetVelocity.y - motion.velocity.y,
  );
  if (velocityError <= TARGET_VELOCITY_TOLERANCE_METERS_PER_SECOND) {
    store.set(spaceshipActiveFeatureAtom, undefined);
    return;
  }

  const remainingSeconds = WorldService.calculateTargetSpeedBurnDuration(
    activeFeature.targetVelocity,
    motion.velocity,
    motion.position,
    activeFeature.maximumAcceleration,
    calculateGravityAcceleration,
  );
  if (remainingSeconds === undefined || remainingSeconds === 0) {
    store.set(spaceshipActiveFeatureAtom, undefined);
    return;
  }

  const nextElapsedSeconds = activeFeature.elapsedSeconds + elapsedSeconds;
  drainActiveThrusterDurability(getSpaceshipActiveThrusters(), elapsedSeconds);
  store.set(spaceshipActiveFeatureAtom, {
    ...activeFeature,
    durationSeconds: nextElapsedSeconds + remainingSeconds,
    elapsedSeconds: nextElapsedSeconds,
  });
}

function advancePositionByVelocity(
  position: Position,
  velocity: Vector,
  elapsedSeconds: number,
  remainder: Vector,
) {
  const deltaX = velocity.x * elapsedSeconds + remainder.x;
  const deltaY = velocity.y * elapsedSeconds + remainder.y;
  const wholeX = Math.trunc(deltaX);
  const wholeY = Math.trunc(deltaY);

  if (wholeX !== 0) position.x += BigInt(wholeX);
  if (wholeY !== 0) position.y += BigInt(wholeY);

  return {
    x: deltaX - wholeX,
    y: deltaY - wholeY,
  };
}

function getSpaceshipReferenceVelocity() {
  const referenceName = spaceshipState.position.relativeTo;
  return referenceName
    ? getCelestialBodyWorldVelocity(referenceName, new Set())
    : { x: 0, y: 0 };
}

function getCelestialBodyWorldVelocity(
  bodyName: string,
  path: Set<string>,
): Vector {
  if (path.has(bodyName)) return { x: 0, y: 0 };

  const body = getBodyByName(bodyName);
  if (!body) return { x: 0, y: 0 };

  const centerName = body.orbitalCenter ?? body.position.relativeTo;
  if (!centerName || body.speed === 0n) return { x: 0, y: 0 };

  path.add(bodyName);
  const centerVelocity = getCelestialBodyWorldVelocity(centerName, path);
  path.delete(bodyName);
  const relativePosition = getWorldPositionRelativeTo(
    body.position,
    centerName,
  );
  const x = Number(relativePosition.x);
  const y = Number(relativePosition.y);
  const radius = Math.hypot(x, y);
  if (radius === 0) return centerVelocity;

  const direction = body.clockwise ? 1 : -1;
  const speed = Number(body.speed);
  return {
    x: centerVelocity.x + (direction * -y * speed) / radius,
    y: centerVelocity.y + (direction * x * speed) / radius,
  };
}

export function getBodyWorldVelocity(bodyName: string) {
  const snapshotVelocity = bodyVelocityByName.get(bodyName);
  return snapshotVelocity
    ? { ...snapshotVelocity }
    : getCelestialBodyWorldVelocity(bodyName, new Set());
}

export function getBodyWorldPositionAfter(
  bodyName: string,
  elapsedSeconds: number,
) {
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds < 0) {
    return undefined;
  }

  return getBodyWorldPositionAt(bodyName, Date.now() + elapsedSeconds * 1_000);
}

function getBodyWorldPositionAt(bodyName: string, timeMs: number) {
  if (!Number.isFinite(timeMs)) return undefined;

  return getBodyWorldPositionAtWithPath(bodyName, timeMs, new Set());
}

function getBodyWorldPositionAtWithPath(
  bodyName: string,
  timeMs: number,
  path: Set<string>,
): Vector | undefined {
  if (path.has(bodyName)) return undefined;

  const body = bodyOrbitEpochByName.get(bodyName);
  if (!body) return undefined;

  const epochTimeMs = getSnapshotTimeMs(body.cTime);
  const elapsedSeconds = Number.isFinite(epochTimeMs)
    ? (timeMs - epochTimeMs) / 1_000
    : 0;
  const futurePosition = WorldService.advanceBodyPosition(body, elapsedSeconds);
  let position = {
    x: Number(futurePosition.x),
    y: Number(futurePosition.y),
  };
  const referenceName = futurePosition.relativeTo;
  if (!referenceName) return position;

  path.add(bodyName);
  const referencePosition = getBodyWorldPositionAtWithPath(
    referenceName,
    timeMs,
    path,
  );
  path.delete(bodyName);

  if (!referencePosition) return position;

  position = WorldService.add(position, referencePosition);
  return position;
}

function cloneBodyOrbitEpoch<TBody extends Planet | Star>(body: TBody): TBody {
  return {
    ...body,
    position: { ...body.position },
  };
}

function toVector(position: Position): Vector {
  return { x: Number(position.x), y: Number(position.y) };
}

export function getWorldPosition(position: Position): Position {
  return getWorldPositionWithBodyMap(position, getWorldBodyByName());
}

function getWorldPositionWithBodyMap(
  position: Position,
  bodyByName: Map<string, Body>,
): Position {
  const resolvePosition = (
    candidate: Position,
    path: Set<string>,
  ): Position => {
    const relativeTo = candidate.relativeTo;
    if (!relativeTo) return candidate;

    const reference = bodyByName.get(relativeTo);
    if (!reference) {
      return candidate;
    }
    if (path.has(relativeTo)) {
      console.warn(`Invalid position reference "${relativeTo}"`);
      return candidate;
    }

    path.add(relativeTo);
    const referencePosition = resolvePosition(reference.position, path);
    path.delete(relativeTo);
    return {
      x: referencePosition.x + candidate.x,
      y: referencePosition.y + candidate.y,
    };
  };

  return resolvePosition(position, new Set());
}

export function getWorldPositionRelativeTo(
  position: Position,
  referenceName: string,
): Position {
  const worldPosition = getWorldPosition(position);
  const reference = getBodyByName(referenceName);

  if (!reference) {
    console.warn(`Invalid position reference "${referenceName}"`);
    return worldPosition;
  }

  const referencePosition = getWorldPosition(reference.position);
  return {
    x: worldPosition.x - referencePosition.x,
    y: worldPosition.y - referencePosition.y,
  };
}

function getBodyByName(name: string) {
  return getWorldBodyByName().get(name);
}

function getWorldBodyByName() {
  if (worldBodyByName.size === 0) rebuildWorldBodyByName();
  return worldBodyByName;
}

function rebuildWorldBodyByName() {
  worldBodyByName = new Map(
    [...worldState.stars, ...worldState.planets, spaceshipState].map((body) => [
      body.name,
      body,
    ]),
  );
  worldPlanetNames = new Set(worldState.planets.map((planet) => planet.name));
}

function normalizeAttachedSpaceshipPosition() {
  const referenceName = spaceshipState.position.relativeTo;
  if (store.get(spaceshipMotionStateAtom) === 'flying' || !referenceName) {
    return;
  }

  const reference = getBodyByName(referenceName);
  if (!reference) {
    if (referenceName === EARTH_NAME) {
      normalizeAttachedSpaceshipPositionToSurface(DEFAULT_SURFACE_OFFSET);
    }
    return;
  }

  normalizeAttachedSpaceshipPositionToSurface(
    Number(reference.radius) + Number(spaceshipState.radius),
  );
}

function normalizeAttachedSpaceshipPositionToSurface(surfaceOffset: number) {
  const offsetX = Number(spaceshipState.position.x);
  const offsetY = Number(spaceshipState.position.y);
  const distance = Math.hypot(offsetX, offsetY);
  const direction =
    distance === 0
      ? { x: 1, y: 0 }
      : { x: offsetX / distance, y: offsetY / distance };
  spaceshipState.position = {
    ...spaceshipState.position,
    x: BigInt(Math.round(direction.x * surfaceOffset)),
    y: BigInt(Math.round(direction.y * surfaceOffset)),
  };
  spaceshipPositionRemainder = { x: 0, y: 0 };
}

function deserializeBody<T extends Body>(
  body: Partial<Omit<T, 'position' | 'radius' | 'mass' | 'speed'>> & {
    name: string;
    position: {
      x: string;
      y: string;
      relativeTo?: string;
      relativeToId?: string;
    };
    radius: string;
    mass: string;
    speed: string;
    cTime?: number | string;
    minZoomRenderShape?: number;
    minZoomRenderName?: number;
    shapeRenderZoomLevel?: number;
    renderZoomLevel?: number;
  },
  defaults: Partial<T> & {
    minZoomRenderShape?: number;
    minZoomRenderName?: number;
    renderZoomLevel?: number;
  } = {},
): T {
  const radius = BigInt(body.radius);
  const minZoomRenderShape =
    body.minZoomRenderShape ??
    body.shapeRenderZoomLevel ??
    defaults.minZoomRenderShape ??
    getMinZoomRenderShape(radius);

  return {
    ...defaults,
    ...body,
    position: {
      x: BigInt(body.position.x),
      y: BigInt(body.position.y),
      relativeTo: body.position.relativeTo,
      relativeToId: body.position.relativeToId,
    },
    radius,
    mass: BigInt(body.mass),
    speed: BigInt(body.speed),
    minZoomRenderShape,
    minZoomRenderName:
      body.minZoomRenderName ??
      body.renderZoomLevel ??
      defaults.minZoomRenderName ??
      defaults.renderZoomLevel ??
      getMinZoomRenderName(minZoomRenderShape),
  } as T;
}

function getPlanetVisualDefaults(name: string): Partial<Planet> {
  const isEarth = name === EARTH_NAME;

  return {
    color: isEarth ? 0x3b82f6 : pickColor(name, PLANET_COLORS),
    variant: isEarth ? 0 : pickIndex(name, 10),
    rotationDegrees: 0,
    rotationPeriodSeconds: 86_400,
  };
}

function getStarVisualDefaults(name: string): Partial<Star> {
  const isSun = name === 'Sun';

  return {
    color: isSun ? 0xfacc15 : pickColor(name, STAR_COLORS),
    variant: isSun ? 0 : pickIndex(name, 4),
    rotationDegrees: 0,
    rotationPeriodSeconds: 2_160_000,
  };
}

function getMinZoomRenderName(minZoomRenderShape: number) {
  return minZoomRenderShape * MIN_RENDER_NAME_TO_SHAPE_ZOOM_RATIO;
}

function pickColor(name: string, colors: number[]) {
  return colors[pickIndex(name, colors.length)] ?? colors[0];
}

function pickIndex(name: string, length: number) {
  let hash = 0;
  for (let index = 0; index < name.length; index += 1) {
    hash = (hash * 31 + name.charCodeAt(index)) >>> 0;
  }
  return hash % length;
}

function getMinZoomRenderShape(radius: bigint) {
  const radiusNumber = Number(radius);
  if (!Number.isFinite(radiusNumber) || radiusNumber <= 0) return 0;

  return MIN_RENDER_SHAPE_SCREEN_WIDTH / 2 / radiusNumber;
}
