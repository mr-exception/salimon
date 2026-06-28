import axios from 'axios';
import { atom, getDefaultStore, useAtomValue, useSetAtom } from 'jotai';
import type {
  Planet,
  Position,
  SerializedWorld,
  Spaceship,
  Star,
  World,
} from '@types';
import { timeSpeedAtom } from './time';

type Body = Planet | Spaceship | Star;
type GravitySource = {
  body: Planet | Star;
  mass: number;
  radius: number;
};
type WorldListener = (
  world: World,
  changedBodyNames?: ReadonlySet<string>,
) => void;
type Vector = { x: number; y: number };
export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';
export type SpaceshipAutoOrbit = {
  active: boolean;
  planetName?: string;
  orbitDistanceMeters?: number;
  speedMetersPerSecond?: number;
};
export type SpaceshipFallingSpeedControl = {
  active: boolean;
  bodyName?: string;
  targetSpeedMetersPerSecond: number;
};
export type SpaceshipProximityTelemetry = {
  bodyName: string;
  bodyKind: 'Planet' | 'Star';
  surfaceDistanceMeters: number;
  relativeSpeedMetersPerSecond: number;
};

export const SPACESHIP_MASS_KG = 10_000;
export const MAX_ENGINE_THRUST_KN = 1_000;
export const INITIAL_SPACESHIP_FUEL_KNS = 1_000_000;
export const BASE_SPACESHIP_CONFIG = {
  crashVelocityThresholdMetersPerSecond: 15,
} as const;
const GRAVITATIONAL_CONSTANT = 6.6743e-11;
const MIN_GRAVITY_ACCELERATION = 1e-8;
const MAX_SIMULATION_STEP_SECONDS = 1;
const MAX_SIMULATION_STEPS = 240;
const EARTH_NAME = 'Earth';
const EARTH_RADIUS_METERS = 6_371_000;
const SPACESHIP_RADIUS_METERS = 200;
const SURFACE_LAUNCH_CLEARANCE_METERS = 1;
const DEFAULT_SURFACE_OFFSET = EARTH_RADIUS_METERS + SPACESHIP_RADIUS_METERS;
const MAX_AUTO_ORBIT_SURFACE_DISTANCE_METERS = 1_000_000;
const AUTO_ORBIT_RADIAL_SPEED_LIMIT_METERS_PER_SECOND = 500;
const AUTO_ORBIT_RADIAL_SPEED_GAIN = 0.0025;
const AUTO_ORBIT_RADIAL_ACCELERATION_GAIN = 0.08;
const AUTO_ORBIT_TANGENTIAL_ACCELERATION_GAIN = 0.05;
const PROXIMITY_TELEMETRY_RANGE_METERS = 3_000_000;

const store = getDefaultStore();

const spaceshipSpeedAtom = atom(0);
const spaceshipTargetDirectionAtom = atom<number | undefined>(undefined);
const spaceshipFuelKnsAtom = atom(INITIAL_SPACESHIP_FUEL_KNS);
const spaceshipMotionStateAtom = atom<SpaceshipMotionState>('landed');
const spaceshipAutoOrbitAtom = atom<SpaceshipAutoOrbit>({ active: false });
const spaceshipFallingSpeedControlAtom = atom<SpaceshipFallingSpeedControl>({
  active: false,
  targetSpeedMetersPerSecond:
    BASE_SPACESHIP_CONFIG.crashVelocityThresholdMetersPerSecond,
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

export function useSpaceshipMotionState() {
  return useAtomValue(spaceshipMotionStateAtom);
}

export function useSetSpaceshipMotionState() {
  return useSetAtom(spaceshipMotionStateAtom);
}

export function getSpaceshipMotionState() {
  return store.get(spaceshipMotionStateAtom);
}

export function getSpaceshipAttachedBodyName() {
  return spaceshipAttachedBodyName;
}

export function useSpaceshipAutoOrbit() {
  return useAtomValue(spaceshipAutoOrbitAtom);
}

export function useSetSpaceshipAutoOrbit() {
  return useSetAtom(spaceshipAutoOrbitAtom);
}

export function getSpaceshipAutoOrbit() {
  return store.get(spaceshipAutoOrbitAtom);
}

export function useSpaceshipFallingSpeedControl() {
  return useAtomValue(spaceshipFallingSpeedControlAtom);
}

export function useSetSpaceshipFallingSpeedControl() {
  return useSetAtom(spaceshipFallingSpeedControlAtom);
}

export function getSpaceshipFallingSpeedControl() {
  return store.get(spaceshipFallingSpeedControlAtom);
}

export function resolveSpaceshipPlanetCollision(planetName: string) {
  if (
    store.get(spaceshipMotionStateAtom) !== 'flying' ||
    !spaceshipWorldPosition ||
    !spaceshipVelocity
  ) {
    return false;
  }

  const planet = getBodyByName(planetName);
  if (!planet || !worldState.planets.includes(planet as Planet)) return false;

  const planetPosition = toVector(getWorldPosition(planet.position));
  const relativePosition = {
    x: spaceshipWorldPosition.x - planetPosition.x,
    y: spaceshipWorldPosition.y - planetPosition.y,
  };
  const surfaceDirection = normalize(relativePosition);
  if (!surfaceDirection) return false;

  const collisionRadius = Number(planet.radius + spaceshipState.radius);
  attachSpaceshipToBody({
    body: planet as Planet,
    bodyVelocity: getCelestialBodyWorldVelocity(planet.name, new Set()),
    surfaceOffset: {
      x: surfaceDirection.x * collisionRadius,
      y: surfaceDirection.y * collisionRadius,
    },
    time: 0,
  });
  return true;
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
let gravitySources: GravitySource[] = [];
let activeWorldBodyNames = new Set<string>();
const suspendedSimulationSeconds = new Map<string, number>();
let spaceshipWorldPosition: Vector | undefined;
let spaceshipVelocity: Vector | undefined;
let spaceshipAttachedBodyName: string | undefined = EARTH_NAME;
let spaceshipSurfaceOffset: Vector | undefined = {
  x: DEFAULT_SURFACE_OFFSET,
  y: 0,
};
let spaceshipBurn:
  | {
      acceleration: Vector;
      elapsedSeconds: number;
      maximumAcceleration: number;
      durationSeconds: number;
      targetVelocity: Vector;
    }
  | undefined;
let lastSpaceshipBurnReachedTarget = false;
let spaceshipManualAcceleration: Vector | undefined;
let spaceshipAutoOrbitAcceleration: Vector | undefined;
let spaceshipAutoOrbitClockwise = false;
let spaceshipFallingSpeedAcceleration: Vector | undefined;
let worldElapsedSeconds = 0;

export async function loadWorld() {
  loadPromise ??= axios
    .get<SerializedWorld>('/world.json')
    .then(({ data }) => {
      worldState.planets = data.planets.map(deserializeBody<Planet>);
      worldState.stars = data.stars.map(deserializeBody<Star>);
      rebuildWorldBodyByName();
      return worldState;
    })
    .catch((error: unknown) => {
      loadPromise = undefined;
      throw error;
    });

  return loadPromise;
}

export function subscribeToWorld(listener: WorldListener) {
  listeners.add(listener);

  return () => {
    listeners.delete(listener);
  };
}

export function setActiveWorldBodyNames(names: Iterable<string>) {
  activeWorldBodyNames = new Set(names);
}

export function setSpaceshipHeading(heading: number) {
  spaceshipState.heading = heading;
  listeners.forEach((listener) => listener(worldState));
}

export function getRequiredSpaceshipBurnAcceleration(
  targetSpeed: number,
  durationSeconds: number,
  targetDirection?: number,
  currentVelocity: Vector = getSpaceshipVelocity(),
) {
  if (
    !Number.isFinite(targetSpeed) ||
    targetSpeed < 0 ||
    !Number.isFinite(durationSeconds) ||
    durationSeconds <= 0
  ) {
    return undefined;
  }

  const direction =
    targetDirection ?? Math.atan2(currentVelocity.y, currentVelocity.x);
  const targetRelativeVelocity = {
    x: Math.cos(direction) * targetSpeed,
    y: Math.sin(direction) * targetSpeed,
  };
  return calculateRequiredBurnAcceleration(
    targetRelativeVelocity,
    durationSeconds,
    currentVelocity,
  );
}

export function getSpaceshipBurnPlan(
  targetSpeed: number,
  maximumThrustPercent: number,
  targetDirection?: number,
  currentVelocity: Vector = getSpaceshipVelocity(),
) {
  if (
    !Number.isFinite(targetSpeed) ||
    targetSpeed < 0 ||
    !Number.isFinite(maximumThrustPercent) ||
    maximumThrustPercent <= 0 ||
    maximumThrustPercent > 100
  ) {
    return undefined;
  }

  const direction =
    targetDirection ?? Math.atan2(currentVelocity.y, currentVelocity.x);
  const targetVelocity = {
    x: Math.cos(direction) * targetSpeed,
    y: Math.sin(direction) * targetSpeed,
  };
  const velocityChange = {
    x: targetVelocity.x - currentVelocity.x,
    y: targetVelocity.y - currentVelocity.y,
  };
  const velocityChangeSquared = velocityChange.x ** 2 + velocityChange.y ** 2;
  if (velocityChangeSquared === 0) return undefined;

  const position =
    spaceshipWorldPosition ??
    toVector(getWorldPosition(spaceshipState.position));
  const gravityAcceleration = calculateGravityAcceleration(
    position,
    getGravitySources(),
  );
  const referenceAcceleration = getSpaceshipReferenceAcceleration();
  const compensationAcceleration = {
    x: referenceAcceleration.x - gravityAcceleration.x,
    y: referenceAcceleration.y - gravityAcceleration.y,
  };
  const maximumAcceleration =
    ((MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG) *
    (maximumThrustPercent / 100);

  // Solve |velocityChange / duration + compensation| = maximumAcceleration.
  const linearCoefficient =
    2 *
    (velocityChange.x * compensationAcceleration.x +
      velocityChange.y * compensationAcceleration.y);
  const constantCoefficient =
    compensationAcceleration.x ** 2 +
    compensationAcceleration.y ** 2 -
    maximumAcceleration ** 2;
  const discriminant =
    linearCoefficient ** 2 - 4 * velocityChangeSquared * constantCoefficient;
  if (discriminant < 0) return undefined;

  const root = Math.sqrt(discriminant);
  const reciprocalDurations = [
    (-linearCoefficient + root) / (2 * velocityChangeSquared),
    (-linearCoefficient - root) / (2 * velocityChangeSquared),
  ].filter((value) => Number.isFinite(value) && value > 0);
  const reciprocalDuration = Math.max(...reciprocalDurations);
  if (!Number.isFinite(reciprocalDuration)) return undefined;

  const durationSeconds = 1 / reciprocalDuration;
  const acceleration = calculateRequiredBurnAcceleration(
    targetVelocity,
    durationSeconds,
    currentVelocity,
  );

  return {
    acceleration,
    durationSeconds,
    maximumAcceleration,
  };
}

function calculateRequiredBurnAcceleration(
  targetRelativeVelocity: Vector,
  remainingSeconds: number,
  currentVelocity: Vector,
) {
  const desiredAcceleration = {
    x: (targetRelativeVelocity.x - currentVelocity.x) / remainingSeconds,
    y: (targetRelativeVelocity.y - currentVelocity.y) / remainingSeconds,
  };
  const position =
    spaceshipWorldPosition ??
    toVector(getWorldPosition(spaceshipState.position));
  const gravityAcceleration = calculateGravityAcceleration(
    position,
    getGravitySources(),
  );
  const referenceAcceleration = getSpaceshipReferenceAcceleration();

  return {
    x: desiredAcceleration.x + referenceAcceleration.x - gravityAcceleration.x,
    y: desiredAcceleration.y + referenceAcceleration.y - gravityAcceleration.y,
  };
}

export function startSpaceshipEngines(
  targetSpeed: number,
  maximumThrustPercent: number,
  targetDirection?: number,
) {
  if (
    spaceshipBurn ||
    store.get(spaceshipMotionStateAtom) === 'crashed' ||
    !Number.isFinite(targetSpeed) ||
    targetSpeed < 0 ||
    !Number.isFinite(maximumThrustPercent) ||
    maximumThrustPercent <= 0 ||
    maximumThrustPercent > 100
  ) {
    return false;
  }

  stopSpaceshipAutoOrbit();
  stopSpaceshipFallingSpeedControl();
  spaceshipManualAcceleration = undefined;
  const currentVelocity = getSpaceshipVelocity();
  const direction =
    targetDirection ?? Math.atan2(currentVelocity.y, currentVelocity.x);
  const targetVelocity = {
    x: Math.cos(direction) * targetSpeed,
    y: Math.sin(direction) * targetSpeed,
  };
  const burnPlan = getSpaceshipBurnPlan(
    targetSpeed,
    maximumThrustPercent,
    targetDirection,
    currentVelocity,
  );
  if (!burnPlan) return false;

  const { acceleration, durationSeconds, maximumAcceleration } = burnPlan;
  const requiredThrustKilonewtons =
    (Math.hypot(acceleration.x, acceleration.y) * SPACESHIP_MASS_KG) / 1_000;
  if (
    requiredThrustKilonewtons > MAX_ENGINE_THRUST_KN ||
    store.get(spaceshipFuelKnsAtom) <= 0
  ) {
    return false;
  }

  spaceshipVelocity ??= getInitialSpaceshipWorldVelocity();
  spaceshipWorldPosition ??= toVector(
    getWorldPosition(spaceshipState.position),
  );
  if (store.get(spaceshipMotionStateAtom) === 'landed') {
    spaceshipAttachedBodyName = undefined;
    spaceshipSurfaceOffset = undefined;
    store.set(spaceshipMotionStateAtom, 'flying');
  }
  spaceshipState.orbitalCenter = null;
  spaceshipBurn = {
    acceleration,
    elapsedSeconds: 0,
    maximumAcceleration,
    durationSeconds,
    targetVelocity,
  };
  lastSpaceshipBurnReachedTarget = false;

  return true;
}

export function startSpaceshipAutoOrbit(
  speedMetersPerSecond: number,
  orbitDistanceMeters: number,
) {
  if (
    spaceshipBurn ||
    spaceshipManualAcceleration ||
    store.get(spaceshipFallingSpeedControlAtom).active ||
    store.get(spaceshipMotionStateAtom) === 'crashed' ||
    !Number.isFinite(speedMetersPerSecond) ||
    speedMetersPerSecond <= 0 ||
    !Number.isFinite(orbitDistanceMeters) ||
    orbitDistanceMeters < 0
  ) {
    return false;
  }

  const orbitTarget = findAutoOrbitTarget();
  if (!orbitTarget) return false;

  const currentVelocity = getSpaceshipVelocity();
  const unit = normalize(orbitTarget.relativePosition);
  if (!unit) return false;

  const counterClockwiseTangent = { x: -unit.y, y: unit.x };
  const clockwise =
    currentVelocity.x * counterClockwiseTangent.x +
      currentVelocity.y * counterClockwiseTangent.y <
    0;

  spaceshipWorldPosition ??= toVector(
    getWorldPosition(spaceshipState.position),
  );
  spaceshipVelocity ??= getInitialSpaceshipWorldVelocity();
  spaceshipAttachedBodyName = undefined;
  spaceshipSurfaceOffset = undefined;
  spaceshipBurn = undefined;
  spaceshipAutoOrbitAcceleration = undefined;
  spaceshipAutoOrbitClockwise = clockwise;
  lastSpaceshipBurnReachedTarget = false;
  spaceshipState.position = {
    x: BigInt(Math.round(spaceshipWorldPosition.x)),
    y: BigInt(Math.round(spaceshipWorldPosition.y)),
  };
  spaceshipState.orbitalCenter = null;
  store.set(spaceshipMotionStateAtom, 'flying');
  store.set(spaceshipAutoOrbitAtom, {
    active: true,
    planetName: orbitTarget.planet.name,
    orbitDistanceMeters,
    speedMetersPerSecond,
  });
  setSpaceshipTargetDirection(undefined);
  listeners.forEach((listener) =>
    listener(worldState, new Set([spaceshipState.name])),
  );

  return true;
}

export function stopSpaceshipAutoOrbit() {
  if (!store.get(spaceshipAutoOrbitAtom).active) return false;

  spaceshipWorldPosition ??= toVector(
    getWorldPosition(spaceshipState.position),
  );
  spaceshipVelocity ??= getInitialSpaceshipWorldVelocity();
  spaceshipState.position = {
    x: BigInt(Math.round(spaceshipWorldPosition.x)),
    y: BigInt(Math.round(spaceshipWorldPosition.y)),
  };
  spaceshipState.orbitalCenter = null;
  spaceshipAutoOrbitAcceleration = undefined;
  store.set(spaceshipAutoOrbitAtom, { active: false });
  listeners.forEach((listener) =>
    listener(worldState, new Set([spaceshipState.name])),
  );

  return true;
}

export function startSpaceshipFallingSpeedControl(
  bodyName: string,
  targetSpeedMetersPerSecond: number,
) {
  if (
    spaceshipBurn ||
    spaceshipManualAcceleration ||
    store.get(spaceshipMotionStateAtom) !== 'flying' ||
    store.get(spaceshipFuelKnsAtom) <= 0 ||
    !worldBodyByName.has(bodyName) ||
    !Number.isFinite(targetSpeedMetersPerSecond) ||
    targetSpeedMetersPerSecond < 0
  ) {
    return false;
  }

  stopSpaceshipAutoOrbit();
  spaceshipWorldPosition ??= toVector(
    getWorldPosition(spaceshipState.position),
  );
  spaceshipVelocity ??= getInitialSpaceshipWorldVelocity();
  spaceshipAttachedBodyName = undefined;
  spaceshipSurfaceOffset = undefined;
  spaceshipState.orbitalCenter = null;
  spaceshipFallingSpeedAcceleration = undefined;
  store.set(spaceshipMotionStateAtom, 'flying');
  store.set(spaceshipFallingSpeedControlAtom, {
    active: true,
    bodyName,
    targetSpeedMetersPerSecond,
  });
  setSpaceshipTargetDirection(undefined);
  listeners.forEach((listener) =>
    listener(worldState, new Set([spaceshipState.name])),
  );
  return true;
}

export function setSpaceshipTargetFallingSpeed(
  targetSpeedMetersPerSecond: number,
) {
  const control = store.get(spaceshipFallingSpeedControlAtom);
  if (
    !control.active ||
    !Number.isFinite(targetSpeedMetersPerSecond) ||
    targetSpeedMetersPerSecond < 0
  ) {
    return false;
  }

  store.set(spaceshipFallingSpeedControlAtom, {
    ...control,
    targetSpeedMetersPerSecond,
  });
  return true;
}

export function stopSpaceshipFallingSpeedControl() {
  if (!store.get(spaceshipFallingSpeedControlAtom).active) return false;

  spaceshipFallingSpeedAcceleration = undefined;
  store.set(spaceshipFallingSpeedControlAtom, {
    active: false,
    targetSpeedMetersPerSecond:
      BASE_SPACESHIP_CONFIG.crashVelocityThresholdMetersPerSecond,
  });
  listeners.forEach((listener) =>
    listener(worldState, new Set([spaceshipState.name])),
  );
  return true;
}

export function isSpaceshipEngineRunning() {
  return (
    spaceshipBurn !== undefined ||
    spaceshipManualAcceleration !== undefined ||
    store.get(spaceshipAutoOrbitAtom).active ||
    (store.get(spaceshipFallingSpeedControlAtom).active &&
      spaceshipFallingSpeedAcceleration !== undefined)
  );
}

export function stopSpaceshipEngines() {
  if (spaceshipManualAcceleration) {
    spaceshipManualAcceleration = undefined;
    return true;
  }

  if (store.get(spaceshipAutoOrbitAtom).active) {
    stopSpaceshipAutoOrbit();
    return true;
  }

  if (store.get(spaceshipFallingSpeedControlAtom).active) {
    stopSpaceshipFallingSpeedControl();
    return true;
  }

  if (!spaceshipBurn) return false;

  spaceshipBurn = undefined;
  lastSpaceshipBurnReachedTarget = false;
  return true;
}

export function getSpaceshipBurnAcceleration() {
  const acceleration =
    spaceshipBurn?.acceleration ??
    spaceshipManualAcceleration ??
    spaceshipAutoOrbitAcceleration ??
    spaceshipFallingSpeedAcceleration;
  return acceleration ? { ...acceleration } : undefined;
}

export function setSpaceshipManualThrust(direction?: Vector, powerPercent = 0) {
  if (!direction || powerPercent <= 0) {
    spaceshipManualAcceleration = undefined;
    return false;
  }

  const magnitude = Math.hypot(direction.x, direction.y);
  if (
    magnitude === 0 ||
    !Number.isFinite(powerPercent) ||
    powerPercent > 100 ||
    store.get(spaceshipMotionStateAtom) === 'crashed' ||
    store.get(spaceshipFuelKnsAtom) <= 0
  ) {
    spaceshipManualAcceleration = undefined;
    return false;
  }

  stopSpaceshipAutoOrbit();
  stopSpaceshipFallingSpeedControl();
  spaceshipBurn = undefined;
  const acceleration =
    ((MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG) * (powerPercent / 100);
  spaceshipManualAcceleration = {
    x: (direction.x / magnitude) * acceleration,
    y: (direction.y / magnitude) * acceleration,
  };
  spaceshipVelocity ??= getInitialSpaceshipWorldVelocity();
  spaceshipWorldPosition ??= toVector(
    getWorldPosition(spaceshipState.position),
  );
  spaceshipState.orbitalCenter = null;
  return true;
}

export function getSpaceshipBurnRemainingSeconds() {
  if (!spaceshipBurn) return undefined;

  return Math.max(
    0,
    spaceshipBurn.durationSeconds - spaceshipBurn.elapsedSeconds,
  );
}

export function didSpaceshipBurnReachTarget() {
  return lastSpaceshipBurnReachedTarget;
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
  const simulationSeconds = elapsedSeconds * store.get(timeSpeedAtom);
  if (simulationSeconds <= 0) return worldElapsedSeconds;
  worldElapsedSeconds += simulationSeconds;

  const stepCount = Math.min(
    MAX_SIMULATION_STEPS,
    Math.max(1, Math.ceil(simulationSeconds / MAX_SIMULATION_STEP_SECONDS)),
  );
  const stepSeconds = simulationSeconds / stepCount;
  for (let step = 0; step < stepCount; step += 1) {
    tickWorld(stepSeconds);
  }
  return worldElapsedSeconds;
}

export function tickWorld(elapsedSeconds = 1) {
  if (
    elapsedSeconds <= 0 ||
    (worldState.planets.length === 0 && worldState.stars.length === 0)
  ) {
    return;
  }

  rotateAttachedSpaceshipSurfaceOffset(elapsedSeconds);
  const spaceshipPosition = advanceSpaceshipMotion(elapsedSeconds);

  const bodies = [...worldState.stars, ...worldState.planets, spaceshipState];
  const bodyByName = new Map(bodies.map((body) => [body.name, body]));
  const simulatedBodyNames = getSimulatedBodyNames(bodyByName);
  const previousPositions = new Map<string, Position>();
  const nextPositions = new Map<string, Position>();
  const changedBodyNames = new Set<string>();

  bodies.forEach((body) => {
    if (body === spaceshipState || body.speed === 0n) return;
    if (simulatedBodyNames.has(body.name)) return;

    suspendedSimulationSeconds.set(
      body.name,
      (suspendedSimulationSeconds.get(body.name) ?? 0) + elapsedSeconds,
    );
  });

  const getPreviousPosition = (name: string) => {
    const cachedPosition = previousPositions.get(name);
    if (cachedPosition) return cachedPosition;

    const body = bodyByName.get(name);
    if (!body) return undefined;

    const position = getWorldPositionWithBodyMap(body.position, bodyByName);
    previousPositions.set(name, position);
    return position;
  };

  const calculatePosition = (
    body: Body,
    path: Set<string>,
  ): Body['position'] => {
    const cachedPosition = nextPositions.get(body.name);
    if (cachedPosition) return cachedPosition;

    if (
      body === spaceshipState &&
      spaceshipAttachedBodyName &&
      spaceshipSurfaceOffset
    ) {
      const attachedBody = bodyByName.get(spaceshipAttachedBodyName);
      if (attachedBody) {
        const attachedBodyPosition = calculatePosition(attachedBody, path);
        const position = {
          x:
            attachedBodyPosition.x +
            BigInt(Math.round(spaceshipSurfaceOffset.x)),
          y:
            attachedBodyPosition.y +
            BigInt(Math.round(spaceshipSurfaceOffset.y)),
        };
        spaceshipWorldPosition = toVector(position);
        nextPositions.set(body.name, position);
        return position;
      }
    }

    if (body === spaceshipState && spaceshipPosition) {
      const position = {
        x: BigInt(Math.round(spaceshipPosition.x)),
        y: BigInt(Math.round(spaceshipPosition.y)),
      };
      nextPositions.set(body.name, position);
      return position;
    }

    const centerName = body.orbitalCenter;
    if (!centerName || body.speed === 0n) {
      const reference = body.position.relativeTo
        ? bodyByName.get(body.position.relativeTo)
        : undefined;
      let referencePosition: Position | undefined;
      if (reference && !path.has(reference.name) && reference !== body) {
        path.add(body.name);
        referencePosition = calculatePosition(reference, path);
        path.delete(body.name);
      } else if (body.position.relativeTo) {
        console.warn(
          `Invalid position reference "${body.position.relativeTo}" for ${body.name}`,
        );
      }
      const position = referencePosition
        ? {
            x: referencePosition.x + body.position.x,
            y: referencePosition.y + body.position.y,
          }
        : { x: body.position.x, y: body.position.y };
      nextPositions.set(body.name, position);
      return position;
    }

    const center = bodyByName.get(centerName);
    const previousCenterPosition = getPreviousPosition(centerName);
    if (!center || !previousCenterPosition || path.has(body.name)) {
      console.warn(`Invalid orbital center "${centerName}" for ${body.name}`);
      return getPreviousPosition(body.name) ?? body.position;
    }

    path.add(body.name);
    const nextCenterPosition = calculatePosition(center, path);
    path.delete(body.name);

    const previousPosition = getPreviousPosition(body.name) ?? body.position;
    const relativeX = Number(previousPosition.x - previousCenterPosition.x);
    const relativeY = Number(previousPosition.y - previousCenterPosition.y);
    const orbitalRadius = Math.hypot(relativeX, relativeY);
    if (orbitalRadius === 0) return previousPosition;

    const bodyElapsedSeconds =
      elapsedSeconds + (suspendedSimulationSeconds.get(body.name) ?? 0);
    const direction = body.clockwise ? 1 : -1;
    const angle =
      (direction * Number(body.speed) * bodyElapsedSeconds) / orbitalRadius;
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    const position = {
      x:
        nextCenterPosition.x +
        BigInt(Math.round(relativeX * cos - relativeY * sin)),
      y:
        nextCenterPosition.y +
        BigInt(Math.round(relativeX * sin + relativeY * cos)),
    };

    suspendedSimulationSeconds.delete(body.name);
    nextPositions.set(body.name, position);
    return position;
  };

  bodies.forEach((body) => {
    if (!simulatedBodyNames.has(body.name)) return;
    calculatePosition(body, new Set());
  });
  bodies.forEach((body) => {
    if (!simulatedBodyNames.has(body.name)) return;
    if (body !== spaceshipState && body.speed === 0n) return;

    const position = nextPositions.get(body.name);
    if (!position) return;

    const relativeTo = body.position.relativeTo;
    const referencePosition = relativeTo
      ? nextPositions.get(relativeTo)
      : undefined;
    body.position = referencePosition
      ? {
          x: position.x - referencePosition.x,
          y: position.y - referencePosition.y,
          relativeTo,
        }
      : position;
    changedBodyNames.add(body.name);
  });
  listeners.forEach((listener) => listener(worldState, changedBodyNames));
}

function getSimulatedBodyNames(bodyByName: Map<string, Body>) {
  const simulatedBodyNames = new Set<string>(activeWorldBodyNames);
  simulatedBodyNames.add(spaceshipState.name);

  if (spaceshipAttachedBodyName) {
    simulatedBodyNames.add(spaceshipAttachedBodyName);
  }

  const includeOrbitalCenters = (
    bodyName: string,
    path = new Set<string>(),
  ) => {
    if (path.has(bodyName)) return;

    const body = bodyByName.get(bodyName);
    const centerName = body?.orbitalCenter;
    if (!centerName) return;

    path.add(bodyName);
    simulatedBodyNames.add(centerName);
    includeOrbitalCenters(centerName, path);
    path.delete(bodyName);
  };

  [...simulatedBodyNames].forEach((bodyName) =>
    includeOrbitalCenters(bodyName),
  );

  return simulatedBodyNames;
}

function advanceSpaceshipMotion(elapsedSeconds: number) {
  const sources = getGravitySources();
  if (sources.length === 0) return undefined;

  if (
    store.get(spaceshipMotionStateAtom) !== 'flying' &&
    spaceshipAttachedBodyName
  ) {
    updateSpaceshipBurnAcceleration();
    const surfaceThrustAcceleration =
      spaceshipBurn?.acceleration ?? spaceshipManualAcceleration;
    const bodyVelocity = getCelestialBodyWorldVelocity(
      spaceshipAttachedBodyName,
      new Set(),
    );
    const surfaceVelocity = getAttachedSpaceshipSurfaceVelocity();
    spaceshipVelocity = {
      x: bodyVelocity.x + surfaceVelocity.x,
      y: bodyVelocity.y + surfaceVelocity.y,
    };
    spaceshipState.speed = 0n;
    store.set(spaceshipSpeedAtom, 0);
    if (
      store.get(spaceshipMotionStateAtom) !== 'landed' ||
      !surfaceThrustAcceleration ||
      !tryLaunchSpaceshipFromSurface(sources, surfaceThrustAcceleration)
    ) {
      consumeSurfaceThrusterFuel(elapsedSeconds, surfaceThrustAcceleration);
      return spaceshipWorldPosition;
    }
  }

  spaceshipWorldPosition ??= toVector(
    getWorldPosition(spaceshipState.position),
  );
  spaceshipVelocity ??= getInitialSpaceshipWorldVelocity();

  updateSpaceshipBurnAcceleration();
  spaceshipAutoOrbitAcceleration = calculateAutoOrbitThrustAcceleration();
  spaceshipFallingSpeedAcceleration =
    calculateFallingSpeedControlAcceleration();
  const thrustAcceleration = spaceshipBurn?.acceleration ??
    spaceshipManualAcceleration ??
    spaceshipAutoOrbitAcceleration ??
    spaceshipFallingSpeedAcceleration ?? { x: 0, y: 0 };
  const thrustKilonewtons =
    (Math.hypot(thrustAcceleration.x, thrustAcceleration.y) *
      SPACESHIP_MASS_KG) /
    1_000;
  const availableFuelKns = store.get(spaceshipFuelKnsAtom);
  const fuelSeconds =
    thrustKilonewtons > 0
      ? availableFuelKns / thrustKilonewtons
      : Number.POSITIVE_INFINITY;
  const requestedBurnSeconds = spaceshipBurn
    ? Math.min(
        elapsedSeconds,
        spaceshipBurn.durationSeconds - spaceshipBurn.elapsedSeconds,
        fuelSeconds,
      )
    : spaceshipManualAcceleration ||
        store.get(spaceshipAutoOrbitAtom).active ||
        store.get(spaceshipFallingSpeedControlAtom).active
      ? Math.min(elapsedSeconds, fuelSeconds)
      : 0;
  const burnSeconds = integrateSpaceship(
    requestedBurnSeconds,
    thrustAcceleration,
    sources,
  );

  const remainingFuelKns = Math.max(
    0,
    availableFuelKns - thrustKilonewtons * burnSeconds,
  );
  if (remainingFuelKns !== availableFuelKns) {
    store.set(spaceshipFuelKnsAtom, remainingFuelKns);
  }

  if (spaceshipBurn) {
    spaceshipBurn.elapsedSeconds += burnSeconds;
    if (spaceshipBurn.elapsedSeconds >= spaceshipBurn.durationSeconds) {
      const targetVelocity = spaceshipBurn.targetVelocity;
      spaceshipBurn = undefined;
      lastSpaceshipBurnReachedTarget =
        isSpaceshipTargetVelocityReached(targetVelocity);
    } else if (remainingFuelKns <= 0) {
      spaceshipBurn = undefined;
      lastSpaceshipBurnReachedTarget = false;
    }
  } else if (
    (spaceshipManualAcceleration ||
      store.get(spaceshipAutoOrbitAtom).active ||
      store.get(spaceshipFallingSpeedControlAtom).active) &&
    remainingFuelKns <= 0
  ) {
    spaceshipManualAcceleration = undefined;
    stopSpaceshipAutoOrbit();
    stopSpaceshipFallingSpeedControl();
  }

  const coastSeconds = elapsedSeconds - burnSeconds;
  integrateSpaceship(coastSeconds, { x: 0, y: 0 }, sources);

  const relativeVelocity = getSpaceshipVelocity();
  const speed = Math.hypot(relativeVelocity.x, relativeVelocity.y);
  spaceshipState.speed = BigInt(Math.round(speed));
  if (speed > 0) {
    spaceshipState.heading =
      ((Math.atan2(relativeVelocity.y, relativeVelocity.x) * 180) / Math.PI +
        450) %
      360;
  }
  store.set(spaceshipSpeedAtom, Number(spaceshipState.speed));
  return spaceshipWorldPosition;
}

function rotateAttachedSpaceshipSurfaceOffset(elapsedSeconds: number) {
  if (
    !spaceshipAttachedBodyName ||
    !spaceshipSurfaceOffset ||
    store.get(spaceshipMotionStateAtom) === 'flying'
  ) {
    return;
  }

  const body = getBodyByName(spaceshipAttachedBodyName);
  if (!body || !('rotationPeriodSeconds' in body)) return;

  const angle = (Math.PI * 2 * elapsedSeconds) / body.rotationPeriodSeconds;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const { x, y } = spaceshipSurfaceOffset;
  const contactRadius = Number(body.radius) + SPACESHIP_RADIUS_METERS;
  const rotatedX = x * cos - y * sin;
  const rotatedY = x * sin + y * cos;
  const rotatedRadius = Math.hypot(rotatedX, rotatedY);
  const radiusCorrection =
    rotatedRadius > 0 ? contactRadius / rotatedRadius : 1;
  spaceshipSurfaceOffset = {
    x: rotatedX * radiusCorrection,
    y: rotatedY * radiusCorrection,
  };
}

function getAttachedSpaceshipSurfaceVelocity(): Vector {
  if (!spaceshipAttachedBodyName || !spaceshipSurfaceOffset) {
    return { x: 0, y: 0 };
  }

  const body = getBodyByName(spaceshipAttachedBodyName);
  if (!body || !('rotationPeriodSeconds' in body)) {
    return { x: 0, y: 0 };
  }

  const angularVelocity = (Math.PI * 2) / body.rotationPeriodSeconds;
  return {
    x: -spaceshipSurfaceOffset.y * angularVelocity,
    y: spaceshipSurfaceOffset.x * angularVelocity,
  };
}

function tryLaunchSpaceshipFromSurface(
  sources: GravitySource[],
  thrustAcceleration: Vector,
) {
  if (
    !spaceshipAttachedBodyName ||
    !spaceshipSurfaceOffset ||
    !spaceshipWorldPosition
  ) {
    return false;
  }

  const surfaceNormal = normalize(spaceshipSurfaceOffset);
  if (!surfaceNormal) return false;

  const gravityAcceleration = calculateGravityAcceleration(
    spaceshipWorldPosition,
    sources,
  );
  const referenceAcceleration = getSpaceshipReferenceAcceleration();
  const relativeGravity = {
    x: gravityAcceleration.x - referenceAcceleration.x,
    y: gravityAcceleration.y - referenceAcceleration.y,
  };
  const outwardThrust = dot(thrustAcceleration, surfaceNormal);
  const inwardGravity = Math.max(0, -dot(relativeGravity, surfaceNormal));
  if (outwardThrust <= inwardGravity) return false;

  spaceshipSurfaceOffset = {
    x:
      spaceshipSurfaceOffset.x +
      surfaceNormal.x * SURFACE_LAUNCH_CLEARANCE_METERS,
    y:
      spaceshipSurfaceOffset.y +
      surfaceNormal.y * SURFACE_LAUNCH_CLEARANCE_METERS,
  };
  const attachedBody = getBodyByName(spaceshipAttachedBodyName);
  if (attachedBody) {
    const bodyPosition = toVector(getWorldPosition(attachedBody.position));
    spaceshipWorldPosition = {
      x: bodyPosition.x + spaceshipSurfaceOffset.x,
      y: bodyPosition.y + spaceshipSurfaceOffset.y,
    };
  }
  spaceshipAttachedBodyName = undefined;
  spaceshipSurfaceOffset = undefined;
  spaceshipState.position = {
    x: BigInt(Math.round(spaceshipWorldPosition.x)),
    y: BigInt(Math.round(spaceshipWorldPosition.y)),
  };
  store.set(spaceshipMotionStateAtom, 'flying');
  return true;
}

function consumeSurfaceThrusterFuel(
  elapsedSeconds: number,
  thrustAcceleration?: Vector,
) {
  if (!thrustAcceleration) return;
  const thrustKilonewtons =
    (Math.hypot(thrustAcceleration.x, thrustAcceleration.y) *
      SPACESHIP_MASS_KG) /
    1_000;
  const availableFuelKns = store.get(spaceshipFuelKnsAtom);
  const remainingFuelKns = Math.max(
    0,
    availableFuelKns - thrustKilonewtons * elapsedSeconds,
  );
  store.set(spaceshipFuelKnsAtom, remainingFuelKns);
  if (remainingFuelKns <= 0) {
    spaceshipBurn = undefined;
    spaceshipManualAcceleration = undefined;
    lastSpaceshipBurnReachedTarget = false;
  }
}

function calculateAutoOrbitThrustAcceleration() {
  const autoOrbit = store.get(spaceshipAutoOrbitAtom);
  if (
    !autoOrbit.active ||
    !autoOrbit.planetName ||
    autoOrbit.speedMetersPerSecond === undefined ||
    autoOrbit.orbitDistanceMeters === undefined ||
    !spaceshipWorldPosition ||
    !spaceshipVelocity
  ) {
    spaceshipAutoOrbitAcceleration = undefined;
    return undefined;
  }

  const planet = getBodyByName(autoOrbit.planetName);
  if (!planet || !worldState.planets.includes(planet as Planet)) {
    stopSpaceshipAutoOrbit();
    return undefined;
  }

  const planetPosition = toVector(getWorldPosition(planet.position));
  const planetVelocity = getCelestialBodyWorldVelocity(planet.name, new Set());
  const planetAcceleration = getCelestialBodyWorldAcceleration(
    planet.name,
    new Set(),
  );
  const relativePosition = {
    x: spaceshipWorldPosition.x - planetPosition.x,
    y: spaceshipWorldPosition.y - planetPosition.y,
  };
  const radialUnit = normalize(relativePosition);
  if (!radialUnit) return undefined;

  const direction = spaceshipAutoOrbitClockwise ? 1 : -1;
  const tangentUnit = {
    x: direction * -radialUnit.y,
    y: direction * radialUnit.x,
  };
  const relativeVelocity = {
    x: spaceshipVelocity.x - planetVelocity.x,
    y: spaceshipVelocity.y - planetVelocity.y,
  };
  const radius = Math.hypot(relativePosition.x, relativePosition.y);
  const targetRadius =
    Number(planet.radius) +
    SPACESHIP_RADIUS_METERS +
    autoOrbit.orbitDistanceMeters;
  const radialError = targetRadius - radius;
  const radialSpeed = dot(relativeVelocity, radialUnit);
  const tangentialSpeed = dot(relativeVelocity, tangentUnit);
  const desiredRadialSpeed = clamp(
    radialError * AUTO_ORBIT_RADIAL_SPEED_GAIN,
    -AUTO_ORBIT_RADIAL_SPEED_LIMIT_METERS_PER_SECOND,
    AUTO_ORBIT_RADIAL_SPEED_LIMIT_METERS_PER_SECOND,
  );
  const radialAcceleration =
    (desiredRadialSpeed - radialSpeed) * AUTO_ORBIT_RADIAL_ACCELERATION_GAIN;
  const tangentialAcceleration =
    (autoOrbit.speedMetersPerSecond - tangentialSpeed) *
    AUTO_ORBIT_TANGENTIAL_ACCELERATION_GAIN;
  const targetRelativeAcceleration = {
    x:
      radialUnit.x *
        (radialAcceleration -
          autoOrbit.speedMetersPerSecond ** 2 / Math.max(targetRadius, 1)) +
      tangentUnit.x * tangentialAcceleration,
    y:
      radialUnit.y *
        (radialAcceleration -
          autoOrbit.speedMetersPerSecond ** 2 / Math.max(targetRadius, 1)) +
      tangentUnit.y * tangentialAcceleration,
  };
  const gravityAcceleration = calculateGravityAcceleration(
    spaceshipWorldPosition,
    getGravitySources(),
  );
  const requiredThrustAcceleration = {
    x:
      planetAcceleration.x +
      targetRelativeAcceleration.x -
      gravityAcceleration.x,
    y:
      planetAcceleration.y +
      targetRelativeAcceleration.y -
      gravityAcceleration.y,
  };
  const maximumAcceleration =
    (MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG;
  const magnitude = Math.hypot(
    requiredThrustAcceleration.x,
    requiredThrustAcceleration.y,
  );
  const scale =
    magnitude > maximumAcceleration ? maximumAcceleration / magnitude : 1;

  return {
    x: requiredThrustAcceleration.x * scale,
    y: requiredThrustAcceleration.y * scale,
  };
}

/*
 * Auto-landing controller removed. This block is retained temporarily in
 * history only and is excluded from the compiled game.
function calculateAutoLandingThrustAcceleration() {
  const autoLanding = store.get(spaceshipAutoLandingAtom);
  if (
    !autoLanding.active ||
    !autoLanding.planetName ||
    !spaceshipWorldPosition ||
    !spaceshipVelocity
  ) {
    spaceshipAutoLandingAcceleration = undefined;
    return undefined;
  }
  const planet = getBodyByName(autoLanding.planetName);
  if (!planet || !worldState.planets.includes(planet as Planet)) {
    stopSpaceshipAutoLanding();
    return undefined;
  }

  const planetPosition = toVector(getWorldPosition(planet.position));
  const planetVelocity = getCelestialBodyWorldVelocity(planet.name, new Set());
  const planetAcceleration = getCelestialBodyWorldAcceleration(
    planet.name,
    new Set(),
  );
  const relativePosition = {
    x: spaceshipWorldPosition.x - planetPosition.x,
    y: spaceshipWorldPosition.y - planetPosition.y,
  };
  const radialUnit = normalize(relativePosition);
  if (!radialUnit) return undefined;

  const relativeVelocity = {
    x: spaceshipVelocity.x - planetVelocity.x,
    y: spaceshipVelocity.y - planetVelocity.y,
  };
  const surfaceDistance = Math.max(
    0,
    Math.hypot(relativePosition.x, relativePosition.y) -
      Number(planet.radius) -
      SPACESHIP_RADIUS_METERS,
  );
  const radialSpeed = dot(relativeVelocity, radialUnit);
  const tangentialVelocity = {
    x: relativeVelocity.x - radialUnit.x * radialSpeed,
    y: relativeVelocity.y - radialUnit.y * radialSpeed,
  };
  const gravityAcceleration = calculateGravityAcceleration(
    spaceshipWorldPosition,
    getGravitySources(),
  );
  const landingSpeedThreshold =
    BASE_SPACESHIP_CONFIG.crashVelocityThresholdMetersPerSecond;
  const targetLandingSpeed = landingSpeedThreshold * 0.9;
  const maximumAcceleration =
    (MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG;
  const relativeGravity = {
    x: gravityAcceleration.x - planetAcceleration.x,
    y: gravityAcceleration.y - planetAcceleration.y,
  };
  const inwardGravityAcceleration = Math.max(
    0,
    -dot(relativeGravity, radialUnit),
  );
  const inwardSpeed = Math.max(0, -radialSpeed);
  const tangentialSpeed = Math.hypot(
    tangentialVelocity.x,
    tangentialVelocity.y,
  );
  const landingSpeedSquared = landingSpeedThreshold ** 2;
  const planetGravitationalParameter =
    GRAVITATIONAL_CONSTANT * Number(planet.mass);
  const currentRadius = Math.hypot(relativePosition.x, relativePosition.y);
  const contactRadius = Number(planet.radius) + SPACESHIP_RADIUS_METERS;
  const gravitySpeedGainSquared = Math.max(
    0,
    2 * planetGravitationalParameter * (1 / contactRadius - 1 / currentRadius),
  );
  const predictedFreeFallImpactSpeedSquared =
    inwardSpeed ** 2 + tangentialSpeed ** 2 + gravitySpeedGainSquared;

  if (
    !spaceshipAutoLandingFreeFallStarted &&
    predictedFreeFallImpactSpeedSquared <= landingSpeedSquared
  ) {
    spaceshipAutoLandingFreeFallStarted = true;
  }

  const predictedRadialImpactSpeed = Math.sqrt(
    inwardSpeed ** 2 + gravitySpeedGainSquared,
  );
  const remainingSeconds =
    surfaceDistance /
    Math.max(
      targetLandingSpeed,
      (inwardSpeed + predictedRadialImpactSpeed) / 2,
    );
  store.set(spaceshipAutoLandingAtom, {
    ...autoLanding,
    remainingSeconds: Math.max(0, remainingSeconds),
  });

  if (
    !spaceshipAutoLandingBrakingStarted &&
    remainingSeconds > AUTO_LANDING_BRAKING_START_SECONDS
  ) {
    return { x: 0, y: 0 };
  }
  spaceshipAutoLandingBrakingStarted = true;

  if (spaceshipAutoLandingFreeFallStarted) {
    return { x: 0, y: 0 };
  }

  const safeFreeFallDistance = Math.max(
    0,
    (landingSpeedSquared - tangentialSpeed ** 2) /
      (2 * Math.max(inwardGravityAcceleration, 0.001)),
  );
  const brakingDistance = Math.max(surfaceDistance - safeFreeFallDistance, 1);
  const requiredOutwardNetAcceleration =
    (inwardSpeed ** 2 - targetLandingSpeed ** 2) / (2 * brakingDistance);
  const tangentialBrakingSeconds = Math.max(remainingSeconds, 1);
  const targetRelativeAcceleration = {
    x:
      radialUnit.x * requiredOutwardNetAcceleration -
      tangentialVelocity.x / tangentialBrakingSeconds,
    y:
      radialUnit.y * requiredOutwardNetAcceleration -
      tangentialVelocity.y / tangentialBrakingSeconds,
  };
  let requiredThrustAcceleration = {
    x:
      planetAcceleration.x +
      targetRelativeAcceleration.x -
      gravityAcceleration.x,
    y:
      planetAcceleration.y +
      targetRelativeAcceleration.y -
      gravityAcceleration.y,
  };
  const outwardThrustAcceleration = dot(requiredThrustAcceleration, radialUnit);
  if (outwardThrustAcceleration < 0) {
    requiredThrustAcceleration = {
      x:
        requiredThrustAcceleration.x - radialUnit.x * outwardThrustAcceleration,
      y:
        requiredThrustAcceleration.y - radialUnit.y * outwardThrustAcceleration,
    };
  }
  const magnitude = Math.hypot(
    requiredThrustAcceleration.x,
    requiredThrustAcceleration.y,
  );
  const scale =
    magnitude > maximumAcceleration ? maximumAcceleration / magnitude : 1;

  return {
    x: requiredThrustAcceleration.x * scale,
    y: requiredThrustAcceleration.y * scale,
  };
}

*/

function updateSpaceshipBurnAcceleration() {
  if (!spaceshipBurn) return;

  const remainingSeconds =
    spaceshipBurn.durationSeconds - spaceshipBurn.elapsedSeconds;
  if (remainingSeconds <= 0) return;

  const acceleration = calculateRequiredBurnAcceleration(
    spaceshipBurn.targetVelocity,
    remainingSeconds,
    getSpaceshipVelocity(),
  );
  const magnitude = Math.hypot(acceleration.x, acceleration.y);
  const maximumAcceleration = spaceshipBurn.maximumAcceleration;
  const scale =
    magnitude > maximumAcceleration ? maximumAcceleration / magnitude : 1;
  spaceshipBurn.acceleration = {
    x: acceleration.x * scale,
    y: acceleration.y * scale,
  };
}

function calculateFallingSpeedControlAcceleration() {
  const control = store.get(spaceshipFallingSpeedControlAtom);
  if (
    !control.active ||
    !control.bodyName ||
    !spaceshipWorldPosition ||
    !spaceshipVelocity
  ) {
    return undefined;
  }

  const body = getBodyByName(control.bodyName);
  if (!body || body.name === spaceshipState.name) {
    stopSpaceshipFallingSpeedControl();
    return undefined;
  }

  const bodyPosition = toVector(getWorldPosition(body.position));
  const inwardUnit = normalize({
    x: bodyPosition.x - spaceshipWorldPosition.x,
    y: bodyPosition.y - spaceshipWorldPosition.y,
  });
  if (!inwardUnit) return undefined;

  const bodyVelocity = getCelestialBodyWorldVelocity(body.name, new Set());
  const bodyAcceleration = getCelestialBodyWorldAcceleration(
    body.name,
    new Set(),
  );
  const targetVelocity = {
    x: bodyVelocity.x + inwardUnit.x * control.targetSpeedMetersPerSecond,
    y: bodyVelocity.y + inwardUnit.y * control.targetSpeedMetersPerSecond,
  };
  const velocityError = {
    x: targetVelocity.x - spaceshipVelocity.x,
    y: targetVelocity.y - spaceshipVelocity.y,
  };
  const gravityAcceleration = calculateGravityAcceleration(
    spaceshipWorldPosition,
    getGravitySources(),
  );
  const requiredAcceleration = {
    x: velocityError.x + bodyAcceleration.x - gravityAcceleration.x,
    y: velocityError.y + bodyAcceleration.y - gravityAcceleration.y,
  };
  const maximumAcceleration =
    (MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG;
  const magnitude = Math.hypot(requiredAcceleration.x, requiredAcceleration.y);
  if (magnitude <= 1e-8) return undefined;

  const scale =
    magnitude > maximumAcceleration ? maximumAcceleration / magnitude : 1;
  return {
    x: requiredAcceleration.x * scale,
    y: requiredAcceleration.y * scale,
  };
}

function isSpaceshipTargetVelocityReached(targetVelocity: Vector) {
  const velocity = getSpaceshipVelocity();
  return (
    Math.hypot(targetVelocity.x - velocity.x, targetVelocity.y - velocity.y) <=
    0.1
  );
}

function integrateSpaceship(
  elapsedSeconds: number,
  thrustAcceleration: Vector,
  sources: GravitySource[],
) {
  if (elapsedSeconds <= 0 || !spaceshipWorldPosition || !spaceshipVelocity) {
    return 0;
  }
  if (store.get(spaceshipMotionStateAtom) !== 'flying') return 0;

  const startPosition = { ...spaceshipWorldPosition };
  const gravityAcceleration = calculateGravityAcceleration(
    spaceshipWorldPosition,
    sources,
  );
  const totalAcceleration = {
    x: gravityAcceleration.x + thrustAcceleration.x,
    y: gravityAcceleration.y + thrustAcceleration.y,
  };
  const endVelocity = {
    x: spaceshipVelocity.x + totalAcceleration.x * elapsedSeconds,
    y: spaceshipVelocity.y + totalAcceleration.y * elapsedSeconds,
  };
  const endPosition = {
    x: startPosition.x + endVelocity.x * elapsedSeconds,
    y: startPosition.y + endVelocity.y * elapsedSeconds,
  };
  const collision = findCelestialBodyCollision(
    startPosition,
    endPosition,
    elapsedSeconds,
  );
  const integratedSeconds = elapsedSeconds * (collision?.time ?? 1);

  spaceshipVelocity.x += totalAcceleration.x * integratedSeconds;
  spaceshipVelocity.y += totalAcceleration.y * integratedSeconds;
  if (!collision) {
    spaceshipWorldPosition.x = endPosition.x;
    spaceshipWorldPosition.y = endPosition.y;
    return elapsedSeconds;
  }

  attachSpaceshipToBody(collision);
  return integratedSeconds;
}

type CelestialBodyCollision = {
  body: Planet | Star;
  bodyVelocity: Vector;
  surfaceOffset: Vector;
  time: number;
};

function findCelestialBodyCollision(
  startPosition: Vector,
  endPosition: Vector,
  elapsedSeconds: number,
): CelestialBodyCollision | undefined {
  let firstCollision: CelestialBodyCollision | undefined;

  [...worldState.planets, ...worldState.stars].forEach((body) => {
    const bodyPosition = toVector(getWorldPosition(body.position));
    const bodyVelocity = getCelestialBodyWorldVelocity(body.name, new Set());
    const endBodyPosition = {
      x: bodyPosition.x + bodyVelocity.x * elapsedSeconds,
      y: bodyPosition.y + bodyVelocity.y * elapsedSeconds,
    };
    const relativeStart = {
      x: startPosition.x - bodyPosition.x,
      y: startPosition.y - bodyPosition.y,
    };
    const relativeEnd = {
      x: endPosition.x - endBodyPosition.x,
      y: endPosition.y - endBodyPosition.y,
    };
    const movement = {
      x: relativeEnd.x - relativeStart.x,
      y: relativeEnd.y - relativeStart.y,
    };
    const collisionRadius = Number(body.radius + spaceshipState.radius);
    const startDistance = Math.hypot(relativeStart.x, relativeStart.y);
    const startsInsideSurface = startDistance < collisionRadius - 0.001;
    const startsOnSurfaceMovingInward =
      Math.abs(startDistance - collisionRadius) < 0.001 &&
      dot(relativeStart, movement) < 0;

    if (
      startDistance > 0 &&
      (startsInsideSurface || startsOnSurfaceMovingInward) &&
      (firstCollision?.time ?? 1) >= 0
    ) {
      firstCollision = {
        body,
        bodyVelocity,
        surfaceOffset: {
          x: (relativeStart.x * collisionRadius) / startDistance,
          y: (relativeStart.y * collisionRadius) / startDistance,
        },
        time: 0,
      };
      return;
    }

    const a = movement.x ** 2 + movement.y ** 2;
    const b = 2 * (relativeStart.x * movement.x + relativeStart.y * movement.y);
    const c =
      relativeStart.x ** 2 + relativeStart.y ** 2 - collisionRadius ** 2;
    const discriminant = b ** 2 - 4 * a * c;
    if (a === 0 || discriminant < 0) return;

    const time = (-b - Math.sqrt(discriminant)) / (2 * a);
    if (time < 0 || time > 1 || time > (firstCollision?.time ?? 1)) return;

    const contactVector = {
      x: relativeStart.x + movement.x * time,
      y: relativeStart.y + movement.y * time,
    };
    const contactDistance = Math.hypot(contactVector.x, contactVector.y);
    if (contactDistance === 0) return;

    firstCollision = {
      body,
      bodyVelocity,
      surfaceOffset: {
        x: (contactVector.x * collisionRadius) / contactDistance,
        y: (contactVector.y * collisionRadius) / contactDistance,
      },
      time,
    };
  });

  return firstCollision;
}

function attachSpaceshipToBody(collision: CelestialBodyCollision) {
  if (!spaceshipVelocity) return;

  const angularVelocity = (Math.PI * 2) / collision.body.rotationPeriodSeconds;
  const surfaceVelocity = {
    x: collision.bodyVelocity.x - collision.surfaceOffset.y * angularVelocity,
    y: collision.bodyVelocity.y + collision.surfaceOffset.x * angularVelocity,
  };
  const impactSpeed = Math.hypot(
    spaceshipVelocity.x - surfaceVelocity.x,
    spaceshipVelocity.y - surfaceVelocity.y,
  );
  const motionState: SpaceshipMotionState =
    impactSpeed > BASE_SPACESHIP_CONFIG.crashVelocityThresholdMetersPerSecond
      ? 'crashed'
      : 'landed';

  spaceshipAttachedBodyName = collision.body.name;
  spaceshipSurfaceOffset = collision.surfaceOffset;
  spaceshipVelocity = collision.bodyVelocity;
  const bodyPosition = toVector(getWorldPosition(collision.body.position));
  spaceshipWorldPosition = {
    x: bodyPosition.x + collision.surfaceOffset.x,
    y: bodyPosition.y + collision.surfaceOffset.y,
  };
  spaceshipState.position = {
    x: BigInt(Math.round(collision.surfaceOffset.x)),
    y: BigInt(Math.round(collision.surfaceOffset.y)),
    relativeTo: collision.body.name,
  };
  spaceshipState.orbitalCenter = null;
  spaceshipState.speed = 0n;
  if (motionState === 'crashed') {
    spaceshipBurn = undefined;
    spaceshipManualAcceleration = undefined;
  }
  lastSpaceshipBurnReachedTarget = false;
  store.set(spaceshipAutoOrbitAtom, { active: false });
  store.set(spaceshipFallingSpeedControlAtom, {
    active: false,
    targetSpeedMetersPerSecond:
      BASE_SPACESHIP_CONFIG.crashVelocityThresholdMetersPerSecond,
  });
  spaceshipFallingSpeedAcceleration = undefined;
  store.set(spaceshipSpeedAtom, 0);
  store.set(spaceshipMotionStateAtom, motionState);
  if (motionState === 'crashed') setSpaceshipTargetDirection(undefined);
}

function findNearbyPlanetTarget() {
  const spaceshipPosition = toVector(getWorldPosition(spaceshipState.position));
  let target:
    | {
        planet: Planet;
        centerDistance: number;
        relativePosition: Vector;
        surfaceDistance: number;
      }
    | undefined;

  worldState.planets.forEach((planet) => {
    const planetPosition = toVector(getWorldPosition(planet.position));
    const relativePosition = {
      x: spaceshipPosition.x - planetPosition.x,
      y: spaceshipPosition.y - planetPosition.y,
    };
    const centerDistance = Math.hypot(relativePosition.x, relativePosition.y);
    if (centerDistance === 0) return;

    const surfaceDistance =
      centerDistance - Number(planet.radius) - SPACESHIP_RADIUS_METERS;
    if (
      surfaceDistance < 0 ||
      surfaceDistance >= MAX_AUTO_ORBIT_SURFACE_DISTANCE_METERS
    ) {
      return;
    }

    if (!target || surfaceDistance < target.surfaceDistance) {
      target = { planet, centerDistance, relativePosition, surfaceDistance };
    }
  });

  return target;
}

function findAutoOrbitTarget() {
  return findNearbyPlanetTarget();
}

function calculateGravityAcceleration(
  spaceshipPosition: Vector,
  sources: GravitySource[],
) {
  const acceleration = { x: 0, y: 0 };

  sources.forEach((source) => {
    const bodyPosition = getWorldPosition(source.body.position);
    const deltaX = Number(bodyPosition.x) - spaceshipPosition.x;
    const deltaY = Number(bodyPosition.y) - spaceshipPosition.y;
    const distanceSquared = deltaX ** 2 + deltaY ** 2;
    if (distanceSquared === 0) return;

    // Inside a body, model its mass as a uniform sphere to avoid a singularity.
    const distance = Math.sqrt(distanceSquared);
    const effectiveDistance = Math.max(distance, source.radius);
    const accelerationScale =
      (GRAVITATIONAL_CONSTANT * source.mass) / effectiveDistance ** 3;
    if (distance * accelerationScale < MIN_GRAVITY_ACCELERATION) return;

    acceleration.x += deltaX * accelerationScale;
    acceleration.y += deltaY * accelerationScale;
  });

  return acceleration;
}

function getInitialSpaceshipWorldVelocity() {
  const relativeVelocity = getSpaceshipVelocity();
  const referenceVelocity = getSpaceshipReferenceVelocity();
  return {
    x: referenceVelocity.x + relativeVelocity.x,
    y: referenceVelocity.y + relativeVelocity.y,
  };
}

function getSpaceshipReferenceVelocity() {
  const referenceName = spaceshipState.position.relativeTo;
  return referenceName
    ? getCelestialBodyWorldVelocity(referenceName, new Set())
    : { x: 0, y: 0 };
}

function getSpaceshipReferenceAcceleration() {
  const referenceName = spaceshipState.position.relativeTo;
  return referenceName
    ? getCelestialBodyWorldAcceleration(referenceName, new Set())
    : { x: 0, y: 0 };
}

function getCelestialBodyWorldAcceleration(
  bodyName: string,
  path: Set<string>,
): Vector {
  if (path.has(bodyName)) return { x: 0, y: 0 };

  const body = getBodyByName(bodyName);
  if (!body) return { x: 0, y: 0 };

  const centerName = body.orbitalCenter;
  if (!centerName || body.speed === 0n) return { x: 0, y: 0 };

  path.add(bodyName);
  const centerAcceleration = getCelestialBodyWorldAcceleration(
    centerName,
    path,
  );
  path.delete(bodyName);
  const relativePosition = getWorldPositionRelativeTo(
    body.position,
    centerName,
  );
  const x = Number(relativePosition.x);
  const y = Number(relativePosition.y);
  const radiusSquared = x ** 2 + y ** 2;
  if (radiusSquared === 0) return centerAcceleration;

  const centripetalScale = Number(body.speed) ** 2 / radiusSquared;
  return {
    x: centerAcceleration.x - x * centripetalScale,
    y: centerAcceleration.y - y * centripetalScale,
  };
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

function toVector(position: Position): Vector {
  return { x: Number(position.x), y: Number(position.y) };
}

function normalize(vector: Vector) {
  const magnitude = Math.hypot(vector.x, vector.y);
  if (magnitude === 0) return undefined;

  return {
    x: vector.x / magnitude,
    y: vector.y / magnitude,
  };
}

function dot(first: Vector, second: Vector) {
  return first.x * second.x + first.y * second.y;
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.max(minimum, Math.min(maximum, value));
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

function getGravitySources() {
  if (
    gravitySources.length === 0 &&
    (worldState.stars.length > 0 || worldState.planets.length > 0)
  ) {
    rebuildGravitySources();
  }
  return gravitySources;
}

function rebuildWorldBodyByName() {
  worldBodyByName = new Map(
    [...worldState.stars, ...worldState.planets, spaceshipState].map((body) => [
      body.name,
      body,
    ]),
  );
  rebuildGravitySources();
}

function rebuildGravitySources() {
  gravitySources = [...worldState.stars, ...worldState.planets].map((body) => ({
    body,
    mass: Number(body.mass),
    radius: Number(body.radius),
  }));
}

function deserializeBody<T extends Body>(
  body: Omit<T, 'position' | 'radius' | 'mass' | 'speed'> & {
    position: { x: string; y: string; relativeTo?: string };
    radius: string;
    mass: string;
    speed: string;
  },
): T {
  return {
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
