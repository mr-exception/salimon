import type { SpaceshipDocument, WorldBodyDocument } from '@models';
import type { SpaceshipActiveFeature } from '@repo/types';
import {
  MAX_PROPAGATION_STEPS,
  TARGET_STEP_SECONDS,
  WorldService,
  type Vector,
} from '@repo/world';
import { RepositoryService } from '../repository.service';
import { getSpaceshipVelocity } from './get-spaceship-velocity';

type WorldSnapshot = {
  bodies: WorldBodyDocument[];
  bodiesByName: Map<string, WorldBodyDocument>;
};

const TARGET_VELOCITY_TOLERANCE_METERS_PER_SECOND = 0.1;

export async function propagateSpaceshipToNow(
  spaceship: SpaceshipDocument,
): Promise<SpaceshipDocument> {
  const motionState =
    spaceship.motionState ??
    (spaceship.speed === '0' && spaceship.position.relativeTo
      ? 'landed'
      : 'flying');
  if (motionState !== 'flying') return spaceship;

  const capturedAt = spaceship.simulatedAt ?? spaceship.updatedAt;
  const now = new Date();
  const elapsedSeconds = (now.getTime() - capturedAt.getTime()) / 1_000;
  if (!Number.isFinite(elapsedSeconds) || elapsedSeconds <= 0) {
    return spaceship;
  }

  const worldData = await RepositoryService.getWorldData();
  const bodies = [...worldData.planets, ...worldData.moons, ...worldData.stars];
  const world = {
    bodies,
    bodiesByName: new Map(bodies.map((body) => [body.name, body])),
  };

  const motion = integrateSpaceshipMotion(
    {
      position: getSpaceshipWorldPosition(spaceship, world, capturedAt),
      velocity: getSpaceshipWorldVelocity(spaceship, world, capturedAt),
    },
    spaceship.activeFeature,
    capturedAt,
    elapsedSeconds,
    world,
  );
  const referenceName = getSpaceshipPositionReference(spaceship, world);
  const referencePosition = referenceName
    ? WorldService.getBodyPositions(world, now).get(referenceName)
    : undefined;
  const referenceVelocity = referenceName
    ? WorldService.getBodyVelocity(world, referenceName, now)
    : { x: 0, y: 0 };
  const relativePosition = referencePosition
    ? WorldService.add(motion.position, referencePosition, -1)
    : motion.position;
  const relativeVelocity = WorldService.add(
    motion.velocity,
    referenceVelocity,
    -1,
  );
  const speed = Math.hypot(relativeVelocity.x, relativeVelocity.y);
  const propagated: SpaceshipDocument = {
    ...spaceship,
    position: serializePosition(relativePosition, referenceName),
    velocity: relativeVelocity,
    speed: Math.round(speed).toString(),
    direction:
      speed === 0
        ? spaceship.direction
        : ((Math.atan2(relativeVelocity.y, relativeVelocity.x) * 180) /
            Math.PI +
            450) %
          360,
    motionState,
    activeFeature: advanceActiveFeature(
      spaceship.activeFeature,
      motion.position,
      motion.velocity,
      capturedAt,
      elapsedSeconds,
      world,
    ),
    simulatedAt: now,
    updatedAt: now,
  };

  return (
    (await RepositoryService.updatePropagatedSpaceship(
      spaceship,
      propagated,
    )) ?? propagated
  );
}

function getSpaceshipWorldPosition(
  spaceship: SpaceshipDocument,
  world: WorldSnapshot,
  time: Date,
): Vector {
  const position = toVector(spaceship.position);
  if (!spaceship.position.relativeTo) return position;

  const referencePosition = WorldService.getBodyPositions(world, time).get(
    spaceship.position.relativeTo,
  );
  return referencePosition
    ? WorldService.add(position, referencePosition)
    : position;
}

function getSpaceshipWorldVelocity(
  spaceship: SpaceshipDocument,
  world: WorldSnapshot,
  time: Date,
): Vector {
  const relativeVelocity = getSpaceshipVelocity(spaceship);
  const referenceVelocity = spaceship.position.relativeTo
    ? WorldService.getBodyVelocity(world, spaceship.position.relativeTo, time)
    : { x: 0, y: 0 };

  return WorldService.add(relativeVelocity, referenceVelocity);
}

function integrateSpaceshipMotion(
  motion: { position: Vector; velocity: Vector },
  activeFeature: SpaceshipActiveFeature | undefined,
  capturedAt: Date,
  elapsedSeconds: number,
  world: WorldSnapshot,
) {
  const stepCount = Math.min(
    MAX_PROPAGATION_STEPS,
    Math.max(1, Math.ceil(elapsedSeconds / TARGET_STEP_SECONDS)),
  );
  const stepSeconds = elapsedSeconds / stepCount;
  let nextMotion = motion;

  for (let step = 0; step < stepCount; step += 1) {
    nextMotion = WorldService.integrateStep(
      nextMotion,
      stepSeconds,
      (position, offsetSeconds) =>
        WorldService.calculateAcceleration(
          position,
          (nextPosition) =>
            calculateGravityAcceleration(
              nextPosition,
              capturedAt,
              step * stepSeconds + offsetSeconds,
              world,
            ),
          calculateSpaceshipActiveThrustAcceleration(
            activeFeature,
            nextMotion,
            capturedAt,
            step * stepSeconds,
            world,
          ),
        ),
    );
  }

  return nextMotion;
}

function calculateGravityAcceleration(
  position: Vector,
  capturedAt: Date,
  offsetSeconds: number,
  world: WorldSnapshot,
) {
  const bodyPositions = WorldService.getBodyPositions(
    world,
    new Date(capturedAt.getTime() + offsetSeconds * 1_000),
  );

  return WorldService.calculateGravityAcceleration(
    position,
    world.bodies,
    (body) => bodyPositions.get(body.name),
  );
}

function calculateSpaceshipActiveThrustAcceleration(
  activeFeature: SpaceshipActiveFeature | undefined,
  motion: { position: Vector; velocity: Vector },
  capturedAt: Date,
  elapsedSeconds: number,
  world: WorldSnapshot,
) {
  if (
    activeFeature?.type === 'thrusters' ||
    activeFeature?.type === 'manual-force'
  ) {
    const acceleration = { x: 0, y: 0 };
    activeFeature.thrusters.forEach((thruster, index) => {
      if (!thruster.active || thruster.powerPercent <= 0) return;

      const thrustAcceleration =
        WorldService.calculateMaximumEngineAcceleration(thruster.powerPercent);
      if (index === 0) acceleration.y += thrustAcceleration;
      if (index === 1) acceleration.x -= thrustAcceleration;
      if (index === 2) acceleration.y -= thrustAcceleration;
      if (index === 3) acceleration.x += thrustAcceleration;
    });

    return acceleration.x === 0 && acceleration.y === 0
      ? undefined
      : acceleration;
  }

  if (activeFeature?.type !== 'target-speed') return undefined;

  const remainingSeconds = Math.max(
    activeFeature.durationSeconds -
      activeFeature.elapsedSeconds -
      elapsedSeconds,
    WorldService.calculateTargetSpeedBurnDuration(
      activeFeature.targetVelocity,
      motion.velocity,
      motion.position,
      activeFeature.maximumAcceleration,
      (position) =>
        calculateGravityAcceleration(
          position,
          capturedAt,
          elapsedSeconds,
          world,
        ),
    ) ?? 0,
  );
  if (remainingSeconds <= 0) return undefined;

  const requestedAcceleration = WorldService.calculateRequiredBurnAcceleration(
    activeFeature.targetVelocity,
    remainingSeconds,
    motion.velocity,
    motion.position,
    (position) =>
      calculateGravityAcceleration(position, capturedAt, elapsedSeconds, world),
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

function advanceActiveFeature(
  activeFeature: SpaceshipActiveFeature | undefined,
  position: Vector,
  velocity: Vector,
  capturedAt: Date,
  elapsedSeconds: number,
  world: WorldSnapshot,
): SpaceshipActiveFeature | undefined {
  if (
    activeFeature?.type === 'thrusters' ||
    activeFeature?.type === 'manual-force'
  ) {
    return {
      ...activeFeature,
      elapsedSeconds: activeFeature.elapsedSeconds + elapsedSeconds,
    };
  }

  if (activeFeature?.type !== 'target-speed') return activeFeature;

  const velocityError = Math.hypot(
    activeFeature.targetVelocity.x - velocity.x,
    activeFeature.targetVelocity.y - velocity.y,
  );
  if (velocityError <= TARGET_VELOCITY_TOLERANCE_METERS_PER_SECOND) {
    return undefined;
  }

  const remainingSeconds = WorldService.calculateTargetSpeedBurnDuration(
    activeFeature.targetVelocity,
    velocity,
    position,
    activeFeature.maximumAcceleration,
    (nextPosition) =>
      calculateGravityAcceleration(
        nextPosition,
        capturedAt,
        elapsedSeconds,
        world,
      ),
  );
  if (remainingSeconds === undefined || remainingSeconds === 0) {
    return undefined;
  }

  const nextElapsedSeconds = activeFeature.elapsedSeconds + elapsedSeconds;
  return {
    ...activeFeature,
    durationSeconds: nextElapsedSeconds + remainingSeconds,
    elapsedSeconds: nextElapsedSeconds,
  };
}

function getSpaceshipPositionReference(
  spaceship: SpaceshipDocument,
  world: WorldSnapshot,
) {
  const referenceName = spaceship.position.relativeTo;
  return referenceName && world.bodiesByName.has(referenceName)
    ? referenceName
    : undefined;
}

function toVector(position: { x: string; y: string }): Vector {
  return { x: Number(position.x), y: Number(position.y) };
}

function serializePosition(position: Vector, relativeTo?: string) {
  return {
    x: Math.round(position.x).toString(),
    y: Math.round(position.y).toString(),
    ...(relativeTo ? { relativeTo } : {}),
  };
}
