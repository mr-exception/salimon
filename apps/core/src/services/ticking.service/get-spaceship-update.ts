import type {
  SpaceshipDocument,
  SpaceshipMotionState,
  SpaceshipVelocity,
  WorldBodyDocument,
} from '@models';
import {
  SPACESHIP_THRUSTER_COUNT,
  SpaceshipService,
} from '../spaceship.service';
import {
  CRASH_SPEED_METERS_PER_SECOND,
  GRAVITATIONAL_CONSTANT,
  MAX_ENGINE_THRUST_KN,
  MAX_PROPAGATION_STEPS,
  SPACESHIP_MASS_KG,
  SPACESHIP_RADIUS_METERS,
  TARGET_STEP_SECONDS,
  THRUSTER_DURABILITY_DRAIN_RATE,
} from './constants';
import type { Impact, Motion, TargetSpeedBurnPlan, WorldSnapshot } from './types';

function add(value: SpaceshipVelocity, change: SpaceshipVelocity, scale = 1) {
  return {
    x: value.x + change.x * scale,
    y: value.y + change.y * scale,
  };
}

export function getBodyPositions(world: WorldSnapshot, time: Date) {
  const positions = new Map<string, SpaceshipVelocity>();

  function resolve(
    body: WorldBodyDocument,
    path: Set<string>,
  ): SpaceshipVelocity {
    const cached = positions.get(body.name);
    if (cached) return cached;
    if (path.has(body.name)) {
      throw new Error(`Circular position reference involving ${body.name}`);
    }

    const initialX = Number(body.position.x);
    const initialY = Number(body.position.y);
    const radius = Math.hypot(initialX, initialY);
    const speed = Number(body.speed);
    const elapsedSeconds = (time.getTime() - body.updatedAt.getTime()) / 1_000;
    let localPosition = { x: initialX, y: initialY };

    if (body.orbitalCenter && radius > 0 && speed !== 0) {
      const direction = body.clockwise ? 1 : -1;
      const angle = (direction * speed * elapsedSeconds) / radius;
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      localPosition = {
        x: initialX * cos - initialY * sin,
        y: initialX * sin + initialY * cos,
      };
    }

    const referenceName = body.position.relativeTo;
    if (referenceName) {
      const reference = world.bodiesByName.get(referenceName);
      if (!reference) {
        throw new Error(
          `Position reference ${referenceName} for ${body.name} was not found`,
        );
      }
      const nextPath = new Set(path).add(body.name);
      localPosition = add(localPosition, resolve(reference, nextPath));
    }

    positions.set(body.name, localPosition);
    return localPosition;
  }

  world.bodies.forEach((body) => resolve(body, new Set()));
  return positions;
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

function acceleration(
  position: SpaceshipVelocity,
  world: WorldSnapshot,
  time: Date,
  thrustAcceleration?: SpaceshipVelocity,
) {
  const bodyPositions = getBodyPositions(world, time);
  let x = thrustAcceleration?.x ?? 0;
  let y = thrustAcceleration?.y ?? 0;

  for (const body of world.bodies) {
    const bodyPosition = bodyPositions.get(body.name);
    if (!bodyPosition) continue;
    const deltaX = bodyPosition.x - position.x;
    const deltaY = bodyPosition.y - position.y;
    const radiusSquared = deltaX ** 2 + deltaY ** 2;
    if (radiusSquared === 0) continue;
    const scale =
      (GRAVITATIONAL_CONSTANT * Number(body.mass)) /
      (radiusSquared * Math.sqrt(radiusSquared));
    x += deltaX * scale;
    y += deltaY * scale;
  }

  return { x, y };
}

function integrateStep(
  motion: Motion,
  startedAt: Date,
  seconds: number,
  world: WorldSnapshot,
  thrustAcceleration?: SpaceshipVelocity,
): Motion {
  const midpoint = new Date(startedAt.getTime() + (seconds * 1_000) / 2);
  const finishedAt = new Date(startedAt.getTime() + seconds * 1_000);
  const position1 = motion.velocity;
  const velocity1 = acceleration(
    motion.position,
    world,
    startedAt,
    thrustAcceleration,
  );
  const position2 = add(motion.velocity, velocity1, seconds / 2);
  const velocity2 = acceleration(
    add(motion.position, position1, seconds / 2),
    world,
    midpoint,
    thrustAcceleration,
  );
  const position3 = add(motion.velocity, velocity2, seconds / 2);
  const velocity3 = acceleration(
    add(motion.position, position2, seconds / 2),
    world,
    midpoint,
    thrustAcceleration,
  );
  const position4 = add(motion.velocity, velocity3, seconds);
  const velocity4 = acceleration(
    add(motion.position, position3, seconds),
    world,
    finishedAt,
    thrustAcceleration,
  );

  return {
    position: {
      x:
        motion.position.x +
        (seconds / 6) *
          (position1.x + 2 * position2.x + 2 * position3.x + position4.x),
      y:
        motion.position.y +
        (seconds / 6) *
          (position1.y + 2 * position2.y + 2 * position3.y + position4.y),
    },
    velocity: {
      x:
        motion.velocity.x +
        (seconds / 6) *
          (velocity1.x + 2 * velocity2.x + 2 * velocity3.x + velocity4.x),
      y:
        motion.velocity.y +
        (seconds / 6) *
          (velocity1.y + 2 * velocity2.y + 2 * velocity3.y + velocity4.y),
    },
  };
}

function getActiveThrusters(
  accelerationValue: SpaceshipVelocity | undefined,
  stats: SpaceshipDocument['stats'],
) {
  if (!accelerationValue) return undefined;

  const normalizedStats = SpaceshipService.normalizeSpaceshipStats(stats);
  const thrustByIndex = Array<number>(SPACESHIP_THRUSTER_COUNT).fill(0);
  const effectiveAcceleration = { x: 0, y: 0 };
  const xIndex = accelerationValue.x < 0 ? 1 : 3;
  const yIndex = accelerationValue.y < 0 ? 2 : 0;

  if (
    Math.abs(accelerationValue.x) > 1e-8 &&
    normalizedStats.thrusterDurability[xIndex] > 0
  ) {
    effectiveAcceleration.x = accelerationValue.x;
    thrustByIndex[xIndex] =
      (Math.abs(accelerationValue.x) * SPACESHIP_MASS_KG) / 1_000;
  }
  if (
    Math.abs(accelerationValue.y) > 1e-8 &&
    normalizedStats.thrusterDurability[yIndex] > 0
  ) {
    effectiveAcceleration.y = accelerationValue.y;
    thrustByIndex[yIndex] =
      (Math.abs(accelerationValue.y) * SPACESHIP_MASS_KG) / 1_000;
  }

  const activeIndexes = thrustByIndex
    .map((thrust, index) => ({ index, thrust }))
    .filter(({ thrust }) => thrust > 0);
  if (activeIndexes.length === 0) return undefined;

  return {
    effectiveAcceleration,
    thrustByIndex,
    totalKilonewtons: activeIndexes.reduce(
      (total, { thrust }) => total + thrust,
      0,
    ),
    availableSeconds: Math.min(
      ...activeIndexes.map(
        ({ index, thrust }) =>
          normalizedStats.thrusterDurability[index] /
          ((thrust / 100) * THRUSTER_DURABILITY_DRAIN_RATE),
      ),
    ),
  };
}

function wearThrusters(
  stats: ReturnType<typeof SpaceshipService.normalizeSpaceshipStats>,
  thrustByIndex: readonly number[],
  elapsedSeconds: number,
) {
  if (elapsedSeconds <= 0) return stats;

  return {
    ...stats,
    thrusterDurability: stats.thrusterDurability.map((durability, index) =>
      Math.max(
        0,
        durability -
          ((thrustByIndex[index] ?? 0) / 100) *
            THRUSTER_DURABILITY_DRAIN_RATE *
            elapsedSeconds,
      ),
    ),
  };
}

function calculateRequiredBurnAcceleration(
  targetVelocity: SpaceshipVelocity,
  remainingSeconds: number,
  currentVelocity: SpaceshipVelocity,
  position: SpaceshipVelocity,
  world: WorldSnapshot,
  time: Date,
) {
  const desiredAcceleration = {
    x: (targetVelocity.x - currentVelocity.x) / remainingSeconds,
    y: (targetVelocity.y - currentVelocity.y) / remainingSeconds,
  };
  const gravityAcceleration = acceleration(position, world, time);

  return {
    x: desiredAcceleration.x - gravityAcceleration.x,
    y: desiredAcceleration.y - gravityAcceleration.y,
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

  const requestedAcceleration = calculateRequiredBurnAcceleration(
    feature.targetVelocity,
    remainingSeconds,
    motion.velocity,
    motion.position,
    world,
    time,
  );
  const magnitude = Math.hypot(requestedAcceleration.x, requestedAcceleration.y);
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

  const relativePosition = {
    x: Number(spaceship.position.x),
    y: Number(spaceship.position.y),
  };
  const referenceName = spaceship.position.relativeTo;
  const referencePosition = referenceName
    ? getBodyPositions(world, simulatedAt).get(referenceName)
    : undefined;
  const referenceVelocity = referenceName
    ? getBodyVelocity(world, referenceName, simulatedAt)
    : undefined;
  const currentPosition = referencePosition
    ? add(referencePosition, relativePosition)
    : relativePosition;
  const currentVelocity = referenceVelocity
    ? add(referenceVelocity, SpaceshipService.getSpaceshipVelocity(spaceship))
    : SpaceshipService.getSpaceshipVelocity(spaceship);
  const direction =
    targetDirection ?? Math.atan2(currentVelocity.y, currentVelocity.x);
  const targetVelocity = {
    x: Math.cos(direction) * targetSpeedMetersPerSecond,
    y: Math.sin(direction) * targetSpeedMetersPerSecond,
  };
  const velocityChange = {
    x: targetVelocity.x - currentVelocity.x,
    y: targetVelocity.y - currentVelocity.y,
  };
  const velocityChangeSquared = velocityChange.x ** 2 + velocityChange.y ** 2;
  if (velocityChangeSquared === 0) return undefined;

  const gravityAcceleration = acceleration(currentPosition, world, simulatedAt);
  const compensationAcceleration = {
    x: -gravityAcceleration.x,
    y: -gravityAcceleration.y,
  };
  const maximumAcceleration =
    ((MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG) *
    (maximumThrustPercent / 100);
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
  const accelerationValue = calculateRequiredBurnAcceleration(
    targetVelocity,
    durationSeconds,
    currentVelocity,
    currentPosition,
    world,
    simulatedAt,
  );
  if (!getActiveThrusters(accelerationValue, spaceship.stats)) return undefined;

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

  return (
    Math.hypot(
      feature.targetVelocity.x - motion.velocity.x,
      feature.targetVelocity.y - motion.velocity.y,
    ) <= 0.1
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
    position: add(start, delta, fraction),
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

function rotateAttachedPosition(
  position: SpaceshipVelocity,
  elapsedSeconds: number,
  rotationPeriodSeconds: number | undefined,
  collisionRadius: number,
) {
  if (!rotationPeriodSeconds || rotationPeriodSeconds <= 0) return position;
  const angle = (2 * Math.PI * elapsedSeconds) / rotationPeriodSeconds;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const x = position.x * cos - position.y * sin;
  const y = position.x * sin + position.y * cos;
  const radius = Math.hypot(x, y);
  const scale = radius > 0 ? collisionRadius / radius : 1;
  return { x: x * scale, y: y * scale };
}

function getSurfaceVelocity(
  position: SpaceshipVelocity,
  rotationPeriodSeconds: number | undefined,
) {
  if (!rotationPeriodSeconds || rotationPeriodSeconds <= 0) {
    return { x: 0, y: 0 };
  }
  const angularVelocity = (2 * Math.PI) / rotationPeriodSeconds;
  return {
    x: -position.y * angularVelocity,
    y: position.x * angularVelocity,
  };
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

  if (motionState !== 'flying' && referenceBody) {
    const collisionRadius =
      Number(referenceBody.radius) + SPACESHIP_RADIUS_METERS;
    update = serializeMotion(
      spaceship,
      motionState,
      {
        position: rotateAttachedPosition(
          relativePosition,
          elapsedSeconds,
          referenceBody.rotationPeriodSeconds,
          collisionRadius,
        ),
        velocity: { x: 0, y: 0 },
      },
      simulatedAt,
      referenceName,
    );
  } else if (motionState !== 'flying') {
    update = serializeMotion(
      spaceship,
      motionState,
      { position: relativePosition, velocity: { x: 0, y: 0 } },
      simulatedAt,
    );
  } else {
    const initialReferencePosition = referenceName
      ? getBodyPositions(world, previousSimulationTime).get(referenceName)
      : undefined;
    const initialReferenceVelocity = referenceName
      ? getBodyVelocity(world, referenceName, previousSimulationTime)
      : undefined;
    let motion: Motion = {
      position: initialReferencePosition
        ? add(initialReferencePosition, relativePosition)
        : relativePosition,
      velocity: initialReferenceVelocity
        ? add(initialReferenceVelocity, relativeVelocity)
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
      const targetSpeedFeature =
        activeFeature?.type === 'target-speed' ? activeFeature : undefined;
      if (targetSpeedFeature) {
        const requestedAcceleration = getTargetSpeedBurnAcceleration(
          targetSpeedFeature,
          motion,
          world,
          stepStartedAt,
        );
        const activeThrusters = getActiveThrusters(
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
          stats = wearThrusters(
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
      const nextMotion = integrateStep(
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
        const surfaceVelocity = add(
          bodyVelocity,
          getSurfaceVelocity(
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
        impactState =
          impactSpeed > CRASH_SPEED_METERS_PER_SECOND ? 'crashed' : 'landed';
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
          burnSeconds < stepSeconds ||
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
        const coastMotion = integrateStep(
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
            coastStartedAt.getTime() +
              impact.fraction * coastSeconds * 1_000,
          );
          const bodyVelocity = getBodyVelocity(
            world,
            impact.body.name,
            impactTime,
          );
          const surfaceVelocity = add(
            bodyVelocity,
            getSurfaceVelocity(
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
          impactState =
            impactSpeed > CRASH_SPEED_METERS_PER_SECOND
              ? 'crashed'
              : 'landed';
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
              position: rotateAttachedPosition(
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
