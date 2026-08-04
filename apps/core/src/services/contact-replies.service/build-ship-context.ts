import type { SpaceshipDocument, WorldBodyDocument } from '@models';
import type { InventoryMaterial, SerializedPosition } from '@repo/types';
import { WorldService, type Vector } from '@repo/world';
import type { ContactShipContext } from '../contacts.service';
import { RepositoryService } from '../repository.service';
import { getSpaceshipVelocity } from '../spaceship.service/get-spaceship-velocity';
import { normalizeSpaceshipInventory } from '../spaceship.service/normalize-spaceship-inventory';
import { normalizeSpaceshipStats } from '../spaceship.service/normalize-spaceship-stats';

type ShipSnapshot = ContactShipContext;
type WorldSnapshot = {
  bodies: WorldBodyDocument[];
  bodiesByName: Map<string, WorldBodyDocument>;
};
type ClosestBody = {
  body: WorldBodyDocument;
  kind: 'star' | 'planet' | 'moon';
  surfaceDistanceMeters: number;
};

const INVENTORY_MATERIALS = [
  'iron',
  'silicates',
  'ice',
  'silver',
  'carbon',
  'gold',
  'hydrogen',
  'nitrogen',
] as const satisfies readonly InventoryMaterial[];
const SHIP_RADIUS_METERS = 200;

export async function buildShipContext(
  spaceshipSecurityCode: string,
  clientShipContext?: ContactShipContext,
) {
  const storedSpaceship = await RepositoryService.findSpaceshipBySecurityCode(
    spaceshipSecurityCode,
  );
  const snapshot = clientShipContext ?? toShipSnapshot(storedSpaceship);
  if (!snapshot) return '';

  const world = await getWorldSnapshot();
  const snapshotTime = getSnapshotTime(snapshot, storedSpaceship);
  const shipPosition = getShipWorldPosition(snapshot, world, snapshotTime);
  const closestBodies = shipPosition
    ? getClosestBodies(shipPosition, world, snapshotTime)
    : [];
  const closestSystem = shipPosition
    ? getClosestSystem(shipPosition, world, snapshotTime)
    : undefined;

  return [
    'Ship telemetry snapshot (context only; does not change game state):',
    ...formatShipSnapshot(snapshot, storedSpaceship),
    ...formatPosition(snapshot, shipPosition),
    ...formatClosestSystem(closestSystem),
    ...formatClosestBodies(closestBodies),
    ...formatProximity(snapshot),
  ].join('\n');
}

function toShipSnapshot(
  spaceship: SpaceshipDocument | undefined,
): ShipSnapshot | undefined {
  if (!spaceship) return undefined;

  return {
    position: spaceship.position,
    direction: spaceship.direction,
    speed: spaceship.speed,
    velocity: getSpaceshipVelocity(spaceship),
    motionState:
      spaceship.motionState ??
      (spaceship.speed === '0' && spaceship.position.relativeTo
        ? 'landed'
        : 'flying'),
    stats: normalizeSpaceshipStats(spaceship.stats),
    inventory: normalizeSpaceshipInventory(spaceship.inventory),
    activeFeature: spaceship.activeFeature,
    positionCapturedAt: (
      spaceship.simulatedAt ?? spaceship.updatedAt
    ).toISOString(),
    simulatedAt: (spaceship.simulatedAt ?? spaceship.updatedAt).toISOString(),
  };
}

async function getWorldSnapshot(): Promise<WorldSnapshot> {
  const worldData = await RepositoryService.getWorldData();
  const bodies = [...worldData.stars, ...worldData.planets, ...worldData.moons];
  return {
    bodies,
    bodiesByName: new Map(bodies.map((body) => [body.name, body])),
  };
}

function getSnapshotTime(
  snapshot: ShipSnapshot,
  storedSpaceship?: SpaceshipDocument,
) {
  const value =
    snapshot.simulatedAt ??
    snapshot.positionCapturedAt ??
    storedSpaceship?.simulatedAt?.toISOString() ??
    storedSpaceship?.updatedAt?.toISOString();
  const date = value ? new Date(value) : new Date();
  return Number.isFinite(date.getTime()) ? date : new Date();
}

function getShipWorldPosition(
  snapshot: ShipSnapshot,
  world: WorldSnapshot,
  time: Date,
): Vector | undefined {
  if (!snapshot.position) return undefined;

  const position = toVector(snapshot.position);
  const referenceName = snapshot.position.relativeTo;
  if (!referenceName) return position;

  const referencePosition = WorldService.getBodyPositions(world, time).get(
    referenceName,
  );
  return referencePosition
    ? WorldService.add(position, referencePosition)
    : position;
}

function getClosestBodies(
  shipPosition: Vector,
  world: WorldSnapshot,
  time: Date,
) {
  const bodyPositions = WorldService.getBodyPositions(world, time);
  return world.bodies
    .map((body) => {
      const position = bodyPositions.get(body.name);
      if (!position) return undefined;
      const centerDistanceMeters = getDistance(shipPosition, position);
      return {
        body,
        kind: getBodyKind(body, world),
        surfaceDistanceMeters: Math.max(
          0,
          centerDistanceMeters - Number(body.radius) - SHIP_RADIUS_METERS,
        ),
      };
    })
    .filter((candidate): candidate is ClosestBody => candidate !== undefined)
    .sort(
      (left, right) => left.surfaceDistanceMeters - right.surfaceDistanceMeters,
    )
    .slice(0, 2);
}

function getClosestSystem(
  shipPosition: Vector,
  world: WorldSnapshot,
  time: Date,
) {
  const bodyPositions = WorldService.getBodyPositions(world, time);
  return world.bodies
    .filter((body) => getBodyKind(body, world) === 'star')
    .map((body) => {
      const position = bodyPositions.get(body.name);
      return position
        ? { body, distanceMeters: getDistance(shipPosition, position) }
        : undefined;
    })
    .filter(
      (
        candidate,
      ): candidate is { body: WorldBodyDocument; distanceMeters: number } =>
        candidate !== undefined,
    )
    .sort((left, right) => left.distanceMeters - right.distanceMeters)[0];
}

function getBodyKind(
  body: WorldBodyDocument,
  world: WorldSnapshot,
): 'star' | 'planet' | 'moon' {
  if (!body.position.relativeTo) return 'star';
  const reference = world.bodiesByName.get(body.position.relativeTo);
  return reference?.position.relativeTo ? 'moon' : 'planet';
}

function formatShipSnapshot(
  snapshot: ShipSnapshot,
  storedSpaceship?: SpaceshipDocument,
) {
  const stats = normalizeSpaceshipStats(
    snapshot.stats ?? storedSpaceship?.stats,
  );
  const inventory =
    snapshot.inventory ??
    normalizeSpaceshipInventory(storedSpaceship?.inventory);
  const inventoryMassKg = INVENTORY_MATERIALS.reduce(
    (total, material) => total + (inventory[material] ?? 0),
    0,
  );
  const nonEmptyInventory = INVENTORY_MATERIALS.flatMap((material) => {
    const amount = inventory[material] ?? 0;
    return amount > 0 ? [`${material} ${formatNumber(amount)}kg`] : [];
  });

  return [
    `- State ${snapshot.motionState ?? 'unknown'}; speed ${formatNumber(
      Number(snapshot.speed ?? 0),
    )}m/s; dir ${formatNumber(snapshot.direction)}deg.`,
    snapshot.velocity
      ? `- Velocity x ${formatNumber(snapshot.velocity.x)}m/s, y ${formatNumber(
          snapshot.velocity.y,
        )}m/s.`
      : '- Velocity unknown.',
    `- Fuel ${formatNumber(stats.fuelKns)}kNs; hull ${formatNumber(
      stats.hullDurability,
    )}${stats.hullLevel ? ` L${stats.hullLevel}` : ''}; thrusters ${stats.thrusterDurability
      .map((durability, index) => `T${index + 1}:${formatNumber(durability)}`)
      .join(', ')}.`,
    `- Inventory ${formatNumber(inventoryMassKg)}kg/${
      nonEmptyInventory.length > 0 ? nonEmptyInventory.join(', ') : 'empty'
    }.`,
    `- Active: ${formatActiveFeature(snapshot.activeFeature)}; time ${
      snapshot.simulatedAt ?? snapshot.positionCapturedAt ?? 'unknown'
    }.`,
  ];
}

function formatPosition(snapshot: ShipSnapshot, shipPosition?: Vector) {
  return [
    snapshot.position
      ? `- Position x ${snapshot.position.x}, y ${snapshot.position.y}${
          snapshot.position.relativeTo
            ? ` rel ${snapshot.position.relativeTo}`
            : ''
        }.`
      : '- Position unknown.',
    shipPosition
      ? `- World x ${formatNumber(shipPosition.x)}m, y ${formatNumber(
          shipPosition.y,
        )}m.`
      : '- World position unknown.',
  ];
}

function formatClosestSystem(
  closestSystem:
    | { body: WorldBodyDocument; distanceMeters: number }
    | undefined,
) {
  return closestSystem
    ? [
        `- Closest star: ${closestSystem.body.name} at ${formatNumber(
          closestSystem.distanceMeters,
        )}m.`,
      ]
    : ['- Closest star unknown.'];
}

function formatClosestBodies(closestBodies: ClosestBody[]) {
  if (closestBodies.length === 0) return ['- Nearest bodies unknown.'];

  return [
    `- Nearest bodies: ${closestBodies
      .map(
        ({ body, kind, surfaceDistanceMeters }) =>
          `${body.name} ${kind} ${formatNumber(surfaceDistanceMeters)}m surface`,
      )
      .join('; ')}.`,
  ];
}

function formatProximity(snapshot: ShipSnapshot) {
  const telemetry = snapshot.proximityTelemetry;
  return telemetry
    ? [
        `- Proximity: ${telemetry.bodyName} ${telemetry.bodyKind}, surface ${formatNumber(
          telemetry.surfaceDistanceMeters,
        )}m, relative ${formatNumber(
          telemetry.relativeSpeedMetersPerSecond,
        )}m/s.`,
      ]
    : ['- Proximity unavailable.'];
}

function formatActiveFeature(activeFeature: ShipSnapshot['activeFeature']) {
  if (!activeFeature) return 'none';
  if (activeFeature.type === 'target-speed') {
    return `target-speed ${formatNumber(
      activeFeature.targetSpeedMetersPerSecond,
    )}m/s max ${formatNumber(activeFeature.maximumThrustPercent)}%`;
  }
  if (
    activeFeature.type === 'thrusters' ||
    activeFeature.type === 'manual-force'
  ) {
    return `${activeFeature.type} ${activeFeature.thrusters
      .map((thruster, index) =>
        thruster.active
          ? `T${index + 1}:${formatNumber(thruster.powerPercent)}%`
          : `T${index + 1} off`,
      )
      .join(', ')}`;
  }
  return 'unknown active feature';
}

function toVector(position: SerializedPosition): Vector {
  return {
    x: Number(position.x),
    y: Number(position.y),
  };
}

function getDistance(left: Vector, right: Vector) {
  return Math.hypot(left.x - right.x, left.y - right.y);
}

function formatNumber(value: number | undefined) {
  return Number.isFinite(value)
    ? Number(value).toLocaleString('en-US')
    : 'unknown';
}
