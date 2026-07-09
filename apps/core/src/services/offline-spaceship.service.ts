import {
  type SpaceshipDocument,
  type SpaceshipMotionState,
  type SpaceshipVelocity,
  type WorldBodyDocument,
} from '@models';
import { OrbitalUpdaterService } from './orbital-updater.service';
import { RepositoryService } from './repository.service';
import { SpaceshipService } from './spaceship.service';

type Motion = {
  position: SpaceshipVelocity;
  velocity: SpaceshipVelocity;
};

export type OfflineWorld = {
  bodies: WorldBodyDocument[];
  bodiesByName: Map<string, WorldBodyDocument>;
};

type Impact = {
  body: WorldBodyDocument;
  fraction: number;
  relativePosition: SpaceshipVelocity;
};

const GRAVITATIONAL_CONSTANT = 6.6743e-11;
const SPACESHIP_RADIUS_METERS = 200;
const CRASH_SPEED_METERS_PER_SECOND = 15;
const MAX_PROPAGATION_STEPS = 20_000;
const TARGET_STEP_SECONDS = 30;

export class OfflineSpaceshipService {
  static async loadOfflineWorld(): Promise<OfflineWorld> {
    const { planets, moons, stars } =
      await OrbitalUpdaterService.getWorldData();
    const bodies = [...planets, ...moons, ...stars];
    return {
      bodies,
      bodiesByName: new Map(bodies.map((body) => [body.name, body])),
    };
  }

  static async propagateOfflineSpaceship(
    spaceship: SpaceshipDocument,
    simulatedAt = new Date(),
    suppliedWorld?: OfflineWorld,
  ) {
    return propagateOfflineSpaceship(spaceship, simulatedAt, suppliedWorld);
  }
}

async function propagateOfflineSpaceship(
  spaceship: SpaceshipDocument,
  simulatedAt = new Date(),
  suppliedWorld?: OfflineWorld,
) {
  const previousSimulationTime = spaceship.simulatedAt ?? spaceship.updatedAt;
  const elapsedSeconds = Math.max(
    0,
    (simulatedAt.getTime() - previousSimulationTime.getTime()) / 1_000,
  );
  if (elapsedSeconds === 0) return spaceship;

  const world =
    suppliedWorld ?? (await OfflineSpaceshipService.loadOfflineWorld());
  const referenceName = spaceship.position.relativeTo;
  const referenceBody = referenceName
    ? world.bodiesByName.get(referenceName)
    : undefined;
  if (referenceName && !referenceBody) return spaceship;

  const relativePosition = {
    x: Number(spaceship.position.x),
    y: Number(spaceship.position.y),
  };
  const relativeVelocity = SpaceshipService.getSpaceshipVelocity(spaceship);
  const motionState =
    spaceship.motionState ?? (spaceship.speed === '0' ? 'landed' : 'flying');
  let update;

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

    for (let step = 0; step < stepCount; step += 1) {
      const stepStartedAt = new Date(
        previousSimulationTime.getTime() + step * stepSeconds * 1_000,
      );
      const nextMotion = integrateStep(
        motion,
        stepStartedAt,
        stepSeconds,
        world,
      );
      impact = findFirstImpact(
        motion,
        nextMotion,
        world,
        stepStartedAt,
        stepSeconds,
      );
      if (impact) {
        impactTime = new Date(
          stepStartedAt.getTime() + impact.fraction * stepSeconds * 1_000,
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
        break;
      }
      motion = nextMotion;
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
  }

  const stats = SpaceshipService.normalizeSpaceshipStats(spaceship.stats);
  stats.hullDurability =
    update.motionState === 'crashed'
      ? 0
      : Math.max(0, stats.hullDurability - (elapsedSeconds / (30 * 60)) * 0.01);
  return RepositoryService.updatePropagatedSpaceship(spaceship, {
    ...update,
    stats,
  });
}

function add(value: SpaceshipVelocity, change: SpaceshipVelocity, scale = 1) {
  return {
    x: value.x + change.x * scale,
    y: value.y + change.y * scale,
  };
}

function getBodyPositions(world: OfflineWorld, time: Date) {
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

function getBodyVelocity(world: OfflineWorld, bodyName: string, time: Date) {
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
  world: OfflineWorld,
  time: Date,
) {
  const bodyPositions = getBodyPositions(world, time);
  let x = 0;
  let y = 0;

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
  world: OfflineWorld,
): Motion {
  const midpoint = new Date(startedAt.getTime() + (seconds * 1_000) / 2);
  const finishedAt = new Date(startedAt.getTime() + seconds * 1_000);
  const position1 = motion.velocity;
  const velocity1 = acceleration(motion.position, world, startedAt);
  const position2 = add(motion.velocity, velocity1, seconds / 2);
  const velocity2 = acceleration(
    add(motion.position, position1, seconds / 2),
    world,
    midpoint,
  );
  const position3 = add(motion.velocity, velocity2, seconds / 2);
  const velocity3 = acceleration(
    add(motion.position, position2, seconds / 2),
    world,
    midpoint,
  );
  const position4 = add(motion.velocity, velocity3, seconds);
  const velocity4 = acceleration(
    add(motion.position, position3, seconds),
    world,
    finishedAt,
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
  world: OfflineWorld,
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
