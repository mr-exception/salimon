import type {
  SpaceshipDocument,
  SpaceshipMotionState,
  SpaceshipVelocity,
} from '@models';
import { PhysicsService } from '../physics.service';
import { SpaceshipService } from '../spaceship.service';
import {
  MAX_PROPAGATION_STEPS,
  SPACESHIP_RADIUS_METERS,
  TARGET_STEP_SECONDS,
} from './constants';
import type {
  Impact,
  Motion,
  TargetSpeedBurnPlan,
  WorldSnapshot,
} from './types';

const TARGET_VELOCITY_TOLERANCE_METERS_PER_SECOND = 0.1;

export function getBodyPositions(world: WorldSnapshot, time: Date) {
  return PhysicsService.getBodyPositions(world, time);
}

export function getBodyVelocity(
  world: WorldSnapshot,
  bodyName: string,
  time: Date,
) {
  const sampleSeconds = 0.5;
  const before = getBodyPositions(
    world,
    new Date(time.getTime() - sampleSeconds * 1_000),
  ).get(bodyName);
  const after = getBodyPositions(
    world,
    new Date(time.getTime() + sampleSeconds * 1_000),
  ).get(bodyName);
  if (!before || !after) return { x: 0, y: 0 };
  return {
    x: (after.x - before.x) / (sampleSeconds * 2),
    y: (after.y - before.y) / (sampleSeconds * 2),
  };
}

function getTargetSpeedBurnAcceleration(
  feature: TargetSpeedBurnPlan,
  motion: Motion,
  world: WorldSnapshot,
  time: Date,
) {
  const remainingSeconds = feature.durationSeconds - feature.elapsedSeconds;
  if (remainingSeconds <= 0) return undefined;

  const requestedAcceleration =
    PhysicsService.calculateRequiredBurnAcceleration(
      feature.targetVelocity,
      remainingSeconds,
      motion.velocity,
      motion.position,
      world,
      time,
    );
  const magnitude = Math.hypot(
    requestedAcceleration.x,
    requestedAcceleration.y,
  );
  const scale =
    magnitude > feature.maximumAcceleration
      ? feature.maximumAcceleration / magnitude
      : 1;
  return {
    x: requestedAcceleration.x * scale,
    y: requestedAcceleration.y * scale,
  };
}

export function createTargetSpeedFeature(
  spaceship: SpaceshipDocument,
  simulatedAt: Date,
  world: WorldSnapshot,
  targetSpeedMetersPerSecond: number,
  maximumThrustPercent: number,
  targetDirection?: number,
  referenceName?: string,
): TargetSpeedBurnPlan | undefined {
  if (
    spaceship.activeFeature ||
    spaceship.motionState === 'crashed' ||
    !Number.isFinite(targetSpeedMetersPerSecond) ||
    targetSpeedMetersPerSecond < 0 ||
    !Number.isFinite(maximumThrustPercent) ||
    maximumThrustPercent <= 0 ||
    maximumThrustPercent > 100 ||
    (targetDirection !== undefined && !Number.isFinite(targetDirection))
  ) {
    return undefined;
  }

  const currentPosition = {
    x: Number(spaceship.position.x),
    y: Number(spaceship.position.y),
  };
  const referenceVelocity = referenceName
    ? getBodyVelocity(world, referenceName, simulatedAt)
    : undefined;
  const currentVelocity = SpaceshipService.getSpaceshipVelocity(spaceship);
  const direction =
    targetDirection ?? Math.atan2(currentVelocity.y, currentVelocity.x);
  const targetRelativeVelocity = {
    x: Math.cos(direction) * targetSpeedMetersPerSecond,
    y: Math.sin(direction) * targetSpeedMetersPerSecond,
  };
  const targetVelocity = {
    x: (referenceVelocity?.x ?? 0) + targetRelativeVelocity.x,
    y: (referenceVelocity?.y ?? 0) + targetRelativeVelocity.y,
  };
  const maximumAcceleration =
    PhysicsService.calculateMaximumEngineAcceleration(maximumThrustPercent);
  const durationSeconds = PhysicsService.calculateTargetSpeedBurnDuration(
    targetVelocity,
    currentVelocity,
    currentPosition,
    maximumAcceleration,
    world,
    simulatedAt,
  );
  if (durationSeconds === undefined || durationSeconds === 0) return undefined;

  const accelerationValue = PhysicsService.calculateRequiredBurnAcceleration(
    targetVelocity,
    durationSeconds,
    currentVelocity,
    currentPosition,
    world,
    simulatedAt,
  );
  if (!PhysicsService.getActiveThrusters(accelerationValue, spaceship.stats))
    return undefined;

  return {
    type: 'target-speed',
    targetSpeedMetersPerSecond,
    maximumThrustPercent,
    ...(targetDirection === undefined ? {} : { targetDirection }),
    targetVelocity,
    maximumAcceleration,
    durationSeconds,
    elapsedSeconds: 0,
  };
}

function shouldFinishTargetSpeedBurn(
  feature: TargetSpeedBurnPlan,
  motion: Motion,
) {
  if (feature.elapsedSeconds < feature.durationSeconds) return false;

  return isTargetVelocityReached(feature, motion);
}

function isTargetVelocityReached(feature: TargetSpeedBurnPlan, motion: Motion) {
  return (
    Math.hypot(
      feature.targetVelocity.x - motion.velocity.x,
      feature.targetVelocity.y - motion.velocity.y,
    ) <= TARGET_VELOCITY_TOLERANCE_METERS_PER_SECOND
  );
}

function findImpact(
  start: SpaceshipVelocity,
  end: SpaceshipVelocity,
  collisionRadius: number,
) {
  const delta = { x: end.x - start.x, y: end.y - start.y };
  const a = delta.x ** 2 + delta.y ** 2;
  const b = 2 * (start.x * delta.x + start.y * delta.y);
  const c = start.x ** 2 + start.y ** 2 - collisionRadius ** 2;
  if (c <= 0) return { fraction: 0, position: start };
  if (a === 0) return undefined;

  const discriminant = b ** 2 - 4 * a * c;
  if (discriminant < 0) return undefined;
  const fraction = (-b - Math.sqrt(discriminant)) / (2 * a);
  if (fraction < 0 || fraction > 1) return undefined;

  return {
    fraction,
    position: PhysicsService.add(start, delta, fraction),
  };
}

function findFirstImpact(
  motion: Motion,
  nextMotion: Motion,
  world: WorldSnapshot,
  startedAt: Date,
  seconds: number,
): Impact | undefined {
  const startPositions = getBodyPositions(world, startedAt);
  const endPositions = getBodyPositions(
    world,
    new Date(startedAt.getTime() + seconds * 1_000),
  );
  let firstImpact: Impact | undefined;

  for (const body of world.bodies) {
    const startPosition = startPositions.get(body.name);
    const endPosition = endPositions.get(body.name);
    if (!startPosition || !endPosition) continue;
    const impact = findImpact(
      {
        x: motion.position.x - startPosition.x,
        y: motion.position.y - startPosition.y,
      },
      {
        x: nextMotion.position.x - endPosition.x,
        y: nextMotion.position.y - endPosition.y,
      },
      Number(body.radius) + SPACESHIP_RADIUS_METERS,
    );
    if (impact && (!firstImpact || impact.fraction < firstImpact.fraction)) {
      firstImpact = {
        body,
        fraction: impact.fraction,
        relativePosition: impact.position,
      };
    }
  }

  return firstImpact;
}

function findClosestBody(
  position: SpaceshipVelocity,
  world: WorldSnapshot,
  time: Date,
) {
  const positions = getBodyPositions(world, time);
  let closest:
    | {
        body: WorldSnapshot['bodies'][number];
        position: SpaceshipVelocity;
        surfaceDistance: number;
      }
    | undefined;

  for (const body of world.bodies) {
    const bodyPosition = positions.get(body.name);
    if (!bodyPosition) continue;

    const centerDistance = Math.hypot(
      position.x - bodyPosition.x,
      position.y - bodyPosition.y,
    );
    const surfaceDistance =
      centerDistance - Number(body.radius) - SPACESHIP_RADIUS_METERS;
    if (
      closest &&
      Math.abs(surfaceDistance) >= Math.abs(closest.surfaceDistance)
    ) {
      continue;
    }

    closest = { body, position: bodyPosition, surfaceDistance };
  }

  return closest;
}

function serializeMotion(
  spaceship: SpaceshipDocument,
  motionState: SpaceshipMotionState,
  motion: Motion,
  simulatedAt: Date,
  relativeTo?: string,
) {
  const speed =
    motionState === 'flying'
      ? Math.hypot(motion.velocity.x, motion.velocity.y)
      : 0;
  const direction =
    speed > 0
      ? ((Math.atan2(motion.velocity.y, motion.velocity.x) * 180) / Math.PI +
          450) %
        360
      : spaceship.direction;

  return {
    position: {
      x: Math.round(motion.position.x).toString(),
      y: Math.round(motion.position.y).toString(),
      ...(relativeTo ? { relativeTo } : {}),
    },
    velocity: motionState === 'flying' ? motion.velocity : { x: 0, y: 0 },
    speed: Math.round(speed).toString(),
    direction,
    motionState,
    simulatedAt,
    updatedAt: simulatedAt,
  };
}

export function getSpaceshipUpdate(
  spaceship: SpaceshipDocument,
  simulatedAt: Date,
  world: WorldSnapshot,
): Partial<SpaceshipDocument> | undefined {
  const previousSimulationTime = spaceship.simulatedAt ?? spaceship.updatedAt;
  const elapsedSeconds = Math.max(
    0,
    (simulatedAt.getTime() - previousSimulationTime.getTime()) / 1_000,
  );
  if (elapsedSeconds === 0) return undefined;

  const referenceName = spaceship.position.relativeTo;
  const referenceBody = referenceName
    ? world.bodiesByName.get(referenceName)
    : undefined;
  if (referenceName && !referenceBody) return undefined;

  const relativePosition = {
    x: Number(spaceship.position.x),
    y: Number(spaceship.position.y),
  };
  const relativeVelocity = SpaceshipService.getSpaceshipVelocity(spaceship);
  const motionState =
    spaceship.motionState ?? (spaceship.speed === '0' ? 'landed' : 'flying');
  let update: Partial<SpaceshipDocument> | undefined;
  let activeFeature = spaceship.activeFeature;

  if (motionState !== 'flying') {
    const previousPosition = referenceBody
      ? PhysicsService.add(
          getBodyPositions(world, previousSimulationTime).get(referenceName!) ??
            { x: 0, y: 0 },
          relativePosition,
        )
      : relativePosition;
    const closestBody = findClosestBody(
      previousPosition,
      world,
      previousSimulationTime,
    );
    if (!closestBody) {
      update = {
        position: spaceship.position,
        velocity: { x: 0, y: 0 },
        speed: '0',
        direction: spaceship.direction,
        motionState,
        simulatedAt,
        updatedAt: simulatedAt,
      };
    } else {
      const attachedPosition = PhysicsService.rotateAttachedPosition(
        {
          x: previousPosition.x - closestBody.position.x,
          y: previousPosition.y - closestBody.position.y,
        },
        elapsedSeconds,
        closestBody.body.rotationPeriodSeconds,
        Number(closestBody.body.radius) + SPACESHIP_RADIUS_METERS,
      );
      update = {
        position: {
          x: Math.round(attachedPosition.x).toString(),
          y: Math.round(attachedPosition.y).toString(),
          relativeTo: closestBody.body.name,
        },
        velocity: { x: 0, y: 0 },
        speed: '0',
        direction: spaceship.direction,
        motionState,
        simulatedAt,
        updatedAt: simulatedAt,
      };
    }
  } else {
    const initialReferencePosition = referenceName
      ? getBodyPositions(world, previousSimulationTime).get(referenceName)
      : undefined;
    const initialReferenceVelocity = referenceName
      ? getBodyVelocity(world, referenceName, previousSimulationTime)
      : undefined;
    let motion: Motion = {
      position: initialReferencePosition
        ? PhysicsService.add(initialReferencePosition, relativePosition)
        : relativePosition,
      velocity: initialReferenceVelocity
        ? PhysicsService.add(initialReferenceVelocity, relativeVelocity)
        : relativeVelocity,
    };
    const stepCount = Math.min(
      MAX_PROPAGATION_STEPS,
      Math.max(1, Math.ceil(elapsedSeconds / TARGET_STEP_SECONDS)),
    );
    const stepSeconds = elapsedSeconds / stepCount;
    let impact: Impact | undefined;
    let impactState: SpaceshipMotionState | undefined;
    let impactTime: Date | undefined;
    let stats = SpaceshipService.normalizeSpaceshipStats(spaceship.stats);

    for (let step = 0; step < stepCount; step += 1) {
      const stepStartedAt = new Date(
        previousSimulationTime.getTime() + step * stepSeconds * 1_000,
      );
      let burnSeconds = 0;
      let thrustAcceleration: SpaceshipVelocity | undefined;
      let targetSpeedFeature =
        activeFeature?.type === 'target-speed' ? activeFeature : undefined;
      if (targetSpeedFeature) {
        if (isTargetVelocityReached(targetSpeedFeature, motion)) {
          activeFeature = undefined;
          targetSpeedFeature = undefined;
        }
      }
      if (targetSpeedFeature) {
        const remainingSeconds =
          PhysicsService.calculateTargetSpeedBurnDuration(
            targetSpeedFeature.targetVelocity,
            motion.velocity,
            motion.position,
            targetSpeedFeature.maximumAcceleration,
            world,
            stepStartedAt,
          );
        if (remainingSeconds === undefined) {
          activeFeature = undefined;
          targetSpeedFeature = undefined;
        } else if (remainingSeconds === 0) {
          activeFeature = undefined;
          targetSpeedFeature = undefined;
        } else {
          targetSpeedFeature = {
            ...targetSpeedFeature,
            durationSeconds:
              targetSpeedFeature.elapsedSeconds +
              Math.max(remainingSeconds, stepSeconds),
          };
        }
      }
      if (targetSpeedFeature) {
        const requestedAcceleration = getTargetSpeedBurnAcceleration(
          targetSpeedFeature,
          motion,
          world,
          stepStartedAt,
        );
        const activeThrusters = PhysicsService.getActiveThrusters(
          requestedAcceleration,
          stats,
        );
        if (!activeThrusters || stats.fuelKns <= 0) {
          activeFeature = undefined;
        } else {
          const fuelSeconds = stats.fuelKns / activeThrusters.totalKilonewtons;
          burnSeconds = Math.min(
            stepSeconds,
            targetSpeedFeature.durationSeconds -
              targetSpeedFeature.elapsedSeconds,
            fuelSeconds,
            activeThrusters.availableSeconds,
          );
          thrustAcceleration = activeThrusters.effectiveAcceleration;
          stats = PhysicsService.wearThrusters(
            stats,
            activeThrusters.thrustByIndex,
            burnSeconds,
          );
          stats.fuelKns = Math.max(
            0,
            stats.fuelKns - activeThrusters.totalKilonewtons * burnSeconds,
          );
        }
      }
      const nextMotion = PhysicsService.integrateStep(
        motion,
        stepStartedAt,
        burnSeconds > 0 ? burnSeconds : stepSeconds,
        world,
        burnSeconds > 0 ? thrustAcceleration : undefined,
      );
      impact = findFirstImpact(
        motion,
        nextMotion,
        world,
        stepStartedAt,
        burnSeconds > 0 ? burnSeconds : stepSeconds,
      );
      if (impact) {
        impactTime = new Date(
          stepStartedAt.getTime() +
            impact.fraction *
              (burnSeconds > 0 ? burnSeconds : stepSeconds) *
              1_000,
        );
        const bodyVelocity = getBodyVelocity(
          world,
          impact.body.name,
          impactTime,
        );
        const surfaceVelocity = PhysicsService.add(
          bodyVelocity,
          PhysicsService.getSurfaceVelocity(
            impact.relativePosition,
            impact.body.rotationPeriodSeconds,
          ),
        );
        const impactVelocity = {
          x:
            motion.velocity.x +
            (nextMotion.velocity.x - motion.velocity.x) * impact.fraction,
          y:
            motion.velocity.y +
            (nextMotion.velocity.y - motion.velocity.y) * impact.fraction,
        };
        const impactSpeed = Math.hypot(
          impactVelocity.x - surfaceVelocity.x,
          impactVelocity.y - surfaceVelocity.y,
        );
        impactState = PhysicsService.getImpactMotionState(impactSpeed);
        activeFeature = undefined;
        break;
      }
      motion = nextMotion;

      if (targetSpeedFeature && burnSeconds > 0) {
        activeFeature = {
          ...targetSpeedFeature,
          elapsedSeconds: targetSpeedFeature.elapsedSeconds + burnSeconds,
        };
        if (
          shouldFinishTargetSpeedBurn(activeFeature, motion) ||
          stats.fuelKns <= 0
        ) {
          activeFeature = undefined;
        }
      }

      if (burnSeconds > 0 && burnSeconds < stepSeconds && !impact) {
        const coastStartedAt = new Date(
          stepStartedAt.getTime() + burnSeconds * 1_000,
        );
        const coastSeconds = stepSeconds - burnSeconds;
        const coastMotion = PhysicsService.integrateStep(
          motion,
          coastStartedAt,
          coastSeconds,
          world,
        );
        impact = findFirstImpact(
          motion,
          coastMotion,
          world,
          coastStartedAt,
          coastSeconds,
        );
        if (impact) {
          impactTime = new Date(
            coastStartedAt.getTime() + impact.fraction * coastSeconds * 1_000,
          );
          const bodyVelocity = getBodyVelocity(
            world,
            impact.body.name,
            impactTime,
          );
          const surfaceVelocity = PhysicsService.add(
            bodyVelocity,
            PhysicsService.getSurfaceVelocity(
              impact.relativePosition,
              impact.body.rotationPeriodSeconds,
            ),
          );
          const impactVelocity = {
            x:
              motion.velocity.x +
              (coastMotion.velocity.x - motion.velocity.x) * impact.fraction,
            y:
              motion.velocity.y +
              (coastMotion.velocity.y - motion.velocity.y) * impact.fraction,
          };
          const impactSpeed = Math.hypot(
            impactVelocity.x - surfaceVelocity.x,
            impactVelocity.y - surfaceVelocity.y,
          );
          impactState = PhysicsService.getImpactMotionState(impactSpeed);
          activeFeature = undefined;
          break;
        }
        motion = coastMotion;
      }
    }

    update =
      impact && impactState
        ? serializeMotion(
            spaceship,
            impactState,
            {
              position: PhysicsService.rotateAttachedPosition(
                impact.relativePosition,
                impactTime
                  ? (simulatedAt.getTime() - impactTime.getTime()) / 1_000
                  : 0,
                impact.body.rotationPeriodSeconds,
                Number(impact.body.radius) + SPACESHIP_RADIUS_METERS,
              ),
              velocity: { x: 0, y: 0 },
            },
            simulatedAt,
            impact.body.name,
          )
        : serializeMotion(spaceship, 'flying', motion, simulatedAt);

    update.stats = stats;
  }

  if (!update) return undefined;

  const stats =
    update.stats ?? SpaceshipService.normalizeSpaceshipStats(spaceship.stats);
  stats.hullDurability =
    update.motionState === 'crashed'
      ? 0
      : Math.max(0, stats.hullDurability - (elapsedSeconds / (30 * 60)) * 0.01);

  return { ...update, stats, activeFeature };
}
