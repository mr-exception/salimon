import axios from 'axios';
import { atom, getDefaultStore, useAtomValue, useSetAtom } from 'jotai';
import type {
  Planet,
  Position,
  SerializedWorldSystems,
  SpaceshipActiveFeature,
  Spaceship,
  SpaceshipDto,
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

type Body = Planet | Spaceship | Star;
type WorldListener = (
  world: World,
  changedBodyNames?: ReadonlySet<string>,
) => void;
type WorldViewportLoader = (
  request: Required<WorldViewportRequest>,
) => Promise<SerializedWorldSystems>;
export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';
export type SpaceshipProximityTelemetry = {
  bodyName: string;
  bodyKind: 'Planet' | 'Star';
  surfaceDistanceMeters: number;
  relativeSpeedMetersPerSecond: number;
};
export type InventoryMaterial = 'iron' | 'silicates' | 'ice';
export type Inventory = Record<InventoryMaterial, number>;

export const INITIAL_SPACESHIP_FUEL_KNS = 1_000_000;
export const MAX_HULL_DURABILITY = 200;
export const MAX_THRUSTER_DURABILITY = 100;
export const SPACESHIP_THRUSTER_COUNT = 4;
const EARTH_NAME = 'Earth';
const EARTH_RADIUS_METERS = 6_371_000;
const SPACESHIP_RADIUS_METERS = 200;
const DEFAULT_SURFACE_OFFSET = EARTH_RADIUS_METERS + SPACESHIP_RADIUS_METERS;
const PROXIMITY_TELEMETRY_RANGE_METERS = 3_000_000;
const DEFAULT_API_BASE_URL = 'http://localhost:3000';
export const WORLD_VIEWPORT_REFRESH_INTERVAL_MS = 5 * 60 * 1_000;
const DEFAULT_PLANET_SHAPE_RENDER_ZOOM_LEVEL = 0.000001;
const DEFAULT_PLANET_RENDER_ZOOM_LEVEL = 0.0000001;
const DEFAULT_STAR_SHAPE_RENDER_ZOOM_LEVEL = 0.0000000001;
const DEFAULT_STAR_RENDER_ZOOM_LEVEL = 0.00000000001;
const TARGET_VELOCITY_TOLERANCE_METERS_PER_SECOND = 0.1;
const PLANET_COLORS = [
  0x60a5fa, 0x34d399, 0xf59e0b, 0xf97316, 0xa78bfa, 0x94a3b8, 0x22d3ee,
  0xf472b6,
];
const STAR_COLORS = [0xfef08a, 0xfdba74, 0x93c5fd, 0xfca5a5];

const store = getDefaultStore();

const spaceshipSpeedAtom = atom(0);
const spaceshipTargetDirectionAtom = atom<number | undefined>(undefined);
const spaceshipFuelKnsAtom = atom(INITIAL_SPACESHIP_FUEL_KNS);
const spaceshipHullDurabilityAtom = atom(MAX_HULL_DURABILITY);
const spaceshipThrusterDurabilityAtom = atom<number[]>(
  Array(SPACESHIP_THRUSTER_COUNT).fill(MAX_THRUSTER_DURABILITY),
);
const spaceshipMotionStateAtom = atom<SpaceshipMotionState>('landed');
const spaceshipActiveFeatureAtom = atom<SpaceshipActiveFeature | undefined>(
  undefined,
);
const inventoryAtom = atom<Inventory>({
  iron: 0,
  silicates: 0,
  ice: 0,
});

export function useSpaceshipSpeed() {
  return useAtomValue(spaceshipSpeedAtom);
}

export function useSetSpaceshipSpeed() {
  return useSetAtom(spaceshipSpeedAtom);
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

export function useSpaceshipThrusterDurability() {
  return useAtomValue(spaceshipThrusterDurabilityAtom);
}

export function useSetSpaceshipThrusterDurability() {
  return useSetAtom(spaceshipThrusterDurabilityAtom);
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

export function addInventoryMaterial(
  material: InventoryMaterial,
  amount: number,
) {
  if (!Number.isFinite(amount) || amount <= 0) return;

  const inventory = store.get(inventoryAtom);
  store.set(inventoryAtom, {
    ...inventory,
    [material]: inventory[material] + Math.round(amount),
  });
}

export function spendInventory(cost: Partial<Inventory>) {
  const inventory = store.get(inventoryAtom);
  const entries = Object.entries(cost) as [InventoryMaterial, number][];
  const canSpend = entries.every(
    ([material, amount]) => inventory[material] >= amount,
  );
  if (!canSpend) return false;

  store.set(inventoryAtom, {
    iron: inventory.iron - (cost.iron ?? 0),
    silicates: inventory.silicates - (cost.silicates ?? 0),
    ice: inventory.ice - (cost.ice ?? 0),
  });
  return true;
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
let bodyVelocityByName = new Map<string, Vector>();
let spaceshipVelocity: Vector | undefined;
let spaceshipPositionRemainder: Vector = { x: 0, y: 0 };
let spaceshipStoredRelativeVelocity: Vector | undefined;
let spaceshipAttachedBodyName: string | undefined = EARTH_NAME;
let worldElapsedSeconds = 0;
let worldViewportLoader: WorldViewportLoader | undefined;

type WorldViewportRequest = {
  x: string;
  y: string;
  radius: string;
};

export function setWorldViewportLoader(loader?: WorldViewportLoader) {
  worldViewportLoader = loader;
}

export async function loadWorld(request: WorldViewportRequest) {
  if (loadPromise) return loadPromise;

  loadPromise = refreshWorldViewport(request).catch((error: unknown) => {
    loadPromise = undefined;
    throw error;
  });

  return loadPromise;
}

export async function refreshWorldViewport({
  x,
  y,
  radius,
}: WorldViewportRequest) {
  const request = { x, y, radius };
  const data = worldViewportLoader
    ? await worldViewportLoader(request)
    : await loadWorldViewportFromRest(request);

  applyWorldSystems(data);
  return worldState;
}

async function loadWorldViewportFromRest({
  x,
  y,
  radius,
}: Required<WorldViewportRequest>) {
  const apiBaseUrl = (
    import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL
  ).replace(/\/+$/, '');

  const { data } = await axios.get<SerializedWorldSystems>(
    `${apiBaseUrl}/world/systems`,
    {
      params: { x, y, radius },
    },
  );
  return data;
}

export function subscribeToWorld(listener: WorldListener) {
  listeners.add(listener);

  return () => {
    listeners.delete(listener);
  };
}

function applyWorldSystems(data: SerializedWorldSystems) {
  const nextVelocities = new Map<string, Vector>();
  const stars = data.systems.map(({ star }) => {
    if (star.velocity) nextVelocities.set(star.name, star.velocity);
    return deserializeBody<Star>(star, getStarVisualDefaults(star.name));
  });
  const planets = data.systems.flatMap(({ planets: planetSystems }) =>
    planetSystems.flatMap(({ planet, moons }) =>
      [planet, ...moons].map((body) => {
        if (body.velocity) nextVelocities.set(body.name, body.velocity);
        return deserializeBody<Planet>(
          body,
          getPlanetVisualDefaults(body.name),
        );
      }),
    ),
  );

  worldState.stars = mergeBodies(worldState.stars, stars);
  worldState.planets = mergeBodies(worldState.planets, planets);
  bodyVelocityByName = new Map([...bodyVelocityByName, ...nextVelocities]);
  [...stars, ...planets].forEach((body) => {
    advanceBodyPositionToNow(body);
  });
  rebuildWorldBodyByName();
  listeners.forEach((listener) => listener(worldState));
}

export function hydrateWorldSystems(data: SerializedWorldSystems) {
  applyWorldSystems(data);
  return worldState;
}

function mergeBodies<T extends Planet | Star>(current: T[], incoming: T[]) {
  const currentByName = new Map(current.map((body) => [body.name, body]));
  const merged = [...current];

  incoming.forEach((body) => {
    const existing = currentByName.get(body.name);
    if (existing) {
      Object.assign(existing, body);
      return;
    }

    merged.push(body);
  });
  return merged;
}

export function setActiveWorldBodyNames(names?: Iterable<string>) {
  void names;
  return;
}

export function setSpaceshipHeading(heading: number) {
  spaceshipState.heading = heading;
  listeners.forEach((listener) => listener(worldState));
}

export function hydrateSpaceship(dto: SpaceshipDto) {
  spaceshipState.position = {
    x: BigInt(dto.position.x),
    y: BigInt(dto.position.y),
    relativeTo: dto.position.relativeTo,
  };
  spaceshipState.positionCapturedAt = dto.positionCapturedAt ?? dto.simulatedAt;
  spaceshipState.heading = dto.direction;
  spaceshipState.speed = BigInt(dto.speed);
  spaceshipState.orbitalCenter = null;
  spaceshipVelocity = undefined;
  spaceshipPositionRemainder = { x: 0, y: 0 };
  spaceshipStoredRelativeVelocity = dto.velocity
    ? { ...dto.velocity }
    : undefined;
  const motionState =
    dto.motionState ??
    (dto.speed === '0' && dto.position.relativeTo ? 'landed' : 'flying');
  spaceshipAttachedBodyName =
    motionState === 'flying' ? undefined : dto.position.relativeTo;
  store.set(spaceshipMotionStateAtom, motionState);
  store.set(spaceshipActiveFeatureAtom, dto.activeFeature);
  store.set(spaceshipSpeedAtom, Number(dto.speed));
  store.set(
    spaceshipFuelKnsAtom,
    dto.stats?.fuelKns ?? INITIAL_SPACESHIP_FUEL_KNS,
  );
  store.set(
    spaceshipHullDurabilityAtom,
    clampDurability(dto.stats?.hullDurability, MAX_HULL_DURABILITY),
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
  rebuildWorldBodyByName();
  advanceSpaceshipToNow(dto.positionCapturedAt ?? dto.simulatedAt);
  listeners.forEach((listener) => listener(worldState));
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
      thrusterDurability: store.get(spaceshipThrusterDurabilityAtom),
    },
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

  for (const body of [...worldState.planets, ...worldState.stars]) {
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

export function isSpaceshipEngineRunning() {
  return store.get(spaceshipActiveFeatureAtom) !== undefined;
}

export function getSpaceshipActiveThrustVector() {
  return calculateSpaceshipActiveThrustAcceleration({
    position: toVector(getWorldPosition(spaceshipState.position)),
    velocity: getSpaceshipWorldVelocity(),
  });
}

function calculateSpaceshipActiveThrustAcceleration(motion: {
  position: Vector;
  velocity: Vector;
}) {
  const activeFeature = store.get(spaceshipActiveFeatureAtom);
  if (activeFeature?.type !== 'target-speed') return undefined;

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
  if (remainingSeconds <= 0) return undefined;

  const requestedAcceleration = WorldService.calculateRequiredBurnAcceleration(
    activeFeature.targetVelocity,
    remainingSeconds,
    motion.velocity,
    motion.position,
    calculateGravityAcceleration,
  );
  const magnitude = Math.hypot(
    requestedAcceleration.x,
    requestedAcceleration.y,
  );
  const scale =
    magnitude > activeFeature.maximumAcceleration
      ? activeFeature.maximumAcceleration / magnitude
      : 1;

  return {
    x: requestedAcceleration.x * scale,
    y: requestedAcceleration.y * scale,
  };
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
  const spaceshipPosition = toVector(getWorldPosition(spaceshipState.position));
  const spaceshipWorldVelocity =
    spaceshipVelocity ?? getInitialSpaceshipWorldVelocity();
  let closest: SpaceshipProximityTelemetry | undefined;

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
    if (
      surfaceDistance >= PROXIMITY_TELEMETRY_RANGE_METERS ||
      (closest && surfaceDistance >= closest.surfaceDistanceMeters)
    ) {
      continue;
    }

    const bodyVelocity = getCelestialBodyWorldVelocity(body.name, new Set());
    closest = {
      bodyName: body.name,
      bodyKind: worldState.planets.includes(body as Planet) ? 'Planet' : 'Star',
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
  for (const body of [...worldState.stars, ...worldState.planets]) {
    advanceBodyPositionByOrbit(body, elapsedSeconds);
  }
}

function advanceBodyPositionToNow(body: Planet | Star) {
  const positionCapturedAtMs = getSnapshotTimeMs(body.positionCapturedAt);
  if (!Number.isFinite(positionCapturedAtMs)) return;

  const elapsedSeconds = Math.max(
    0,
    (Date.now() - positionCapturedAtMs) / 1000,
  );
  if (elapsedSeconds <= 0) return;

  advanceBodyPositionByOrbit(body, elapsedSeconds);
  body.positionCapturedAt = Date.now();
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

function advanceSpaceshipPosition(elapsedSeconds: number) {
  const motionState = store.get(spaceshipMotionStateAtom);
  if (motionState !== 'flying') return;

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
    [...worldState.stars, ...worldState.planets],
    (body) => toVector(getWorldPosition(body.position)),
  );
}

function advanceActiveFeature(elapsedSeconds: number) {
  const activeFeature = store.get(spaceshipActiveFeatureAtom);
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

  const centerName = body.orbitalCenter;
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
      if (worldState.planets.length > 0 || worldState.stars.length > 0) {
        console.warn(`Invalid position reference "${relativeTo}"`);
      }
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
}

function deserializeBody<T extends Body>(
  body: Partial<Omit<T, 'position' | 'radius' | 'mass' | 'speed'>> & {
    name: string;
    position: { x: string; y: string; relativeTo?: string };
    radius: string;
    mass: string;
    speed: string;
    positionCapturedAt?: number | string;
  },
  defaults: Partial<T> = {},
): T {
  return {
    ...defaults,
    ...body,
    position: {
      x: BigInt(body.position.x),
      y: BigInt(body.position.y),
      relativeTo: body.position.relativeTo,
    },
    radius: BigInt(body.radius),
    mass: BigInt(body.mass),
    speed: BigInt(body.speed),
  } as T;
}

function getPlanetVisualDefaults(name: string): Partial<Planet> {
  const isEarth = name === EARTH_NAME;

  return {
    color: isEarth ? 0x3b82f6 : pickColor(name, PLANET_COLORS),
    variant: isEarth ? 0 : pickIndex(name, 10),
    shapeRenderZoomLevel: DEFAULT_PLANET_SHAPE_RENDER_ZOOM_LEVEL,
    renderZoomLevel: DEFAULT_PLANET_RENDER_ZOOM_LEVEL,
    rotationDegrees: 0,
    rotationPeriodSeconds: 86_400,
  };
}

function getStarVisualDefaults(name: string): Partial<Star> {
  const isSun = name === 'Sun';

  return {
    color: isSun ? 0xfacc15 : pickColor(name, STAR_COLORS),
    variant: isSun ? 0 : pickIndex(name, 4),
    shapeRenderZoomLevel: DEFAULT_STAR_SHAPE_RENDER_ZOOM_LEVEL,
    renderZoomLevel: DEFAULT_STAR_RENDER_ZOOM_LEVEL,
    rotationDegrees: 0,
    rotationPeriodSeconds: 2_160_000,
  };
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
