import type { Collection, Document } from 'mongodb';
import {
  getDatabase,
  getSpaceshipVelocity,
  type SpaceshipDocument,
  type SpaceshipMotionState,
  type SpaceshipVelocity,
} from './spaceship';

type ReferenceBody = Document & {
  name: string;
  mass: string;
  radius: string;
  rotationPeriodSeconds?: number;
};

type Motion = {
  position: SpaceshipVelocity;
  velocity: SpaceshipVelocity;
};

const GRAVITATIONAL_CONSTANT = 6.6743e-11;
const SPACESHIP_RADIUS_METERS = 200;
const CRASH_SPEED_METERS_PER_SECOND = 15;
const MAX_PROPAGATION_STEPS = 20_000;
const TARGET_STEP_SECONDS = 30;

async function findReferenceBody(name: string) {
  const database = await getDatabase();
  const query = { name };
  const projection = {
    _id: 0,
    name: 1,
    mass: 1,
    radius: 1,
    rotationPeriodSeconds: 1,
  };
  const collections: Collection<ReferenceBody>[] = [
    database.collection<ReferenceBody>('planets'),
    database.collection<ReferenceBody>('stars'),
  ];

  return (
    (await collections[0].findOne(query, { projection })) ??
    (await collections[1].findOne(query, { projection }))
  );
}

function acceleration(
  position: SpaceshipVelocity,
  gravitationalParameter: number,
) {
  const radius = Math.hypot(position.x, position.y);
  if (radius === 0) return { x: 0, y: 0 };
  const scale = -gravitationalParameter / radius ** 3;
  return { x: position.x * scale, y: position.y * scale };
}

function add(
  value: SpaceshipVelocity,
  change: SpaceshipVelocity,
  scale: number,
) {
  return {
    x: value.x + change.x * scale,
    y: value.y + change.y * scale,
  };
}

function integrateStep(
  motion: Motion,
  seconds: number,
  gravitationalParameter: number,
): Motion {
  const position1 = motion.velocity;
  const velocity1 = acceleration(motion.position, gravitationalParameter);
  const position2 = add(motion.velocity, velocity1, seconds / 2);
  const velocity2 = acceleration(
    add(motion.position, position1, seconds / 2),
    gravitationalParameter,
  );
  const position3 = add(motion.velocity, velocity2, seconds / 2);
  const velocity3 = acceleration(
    add(motion.position, position2, seconds / 2),
    gravitationalParameter,
  );
  const position4 = add(motion.velocity, velocity3, seconds);
  const velocity4 = acceleration(
    add(motion.position, position3, seconds),
    gravitationalParameter,
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
  if (c <= 0) return start;
  if (a === 0) return undefined;

  const discriminant = b ** 2 - 4 * a * c;
  if (discriminant < 0) return undefined;
  const root = Math.sqrt(discriminant);
  const time = (-b - root) / (2 * a);
  if (time < 0 || time > 1) return undefined;

  return {
    x: start.x + delta.x * time,
    y: start.y + delta.y * time,
  };
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
      relativeTo: spaceship.position.relativeTo,
    },
    velocity: motionState === 'flying' ? motion.velocity : { x: 0, y: 0 },
    speed: Math.round(speed).toString(),
    direction,
    motionState,
    simulatedAt,
    updatedAt: simulatedAt,
  };
}

export async function propagateOfflineSpaceship(
  spaceship: SpaceshipDocument,
  simulatedAt = new Date(),
) {
  const previousSimulationTime = spaceship.simulatedAt ?? spaceship.updatedAt;
  const elapsedSeconds = Math.max(
    0,
    (simulatedAt.getTime() - previousSimulationTime.getTime()) / 1_000,
  );
  const referenceName = spaceship.position.relativeTo;
  if (elapsedSeconds === 0 || !referenceName) return spaceship;

  const body = await findReferenceBody(referenceName);
  if (!body) return spaceship;

  const collisionRadius = Number(body.radius) + SPACESHIP_RADIUS_METERS;
  const initialMotion: Motion = {
    position: {
      x: Number(spaceship.position.x),
      y: Number(spaceship.position.y),
    },
    velocity: getSpaceshipVelocity(spaceship),
  };
  const motionState =
    spaceship.motionState ?? (spaceship.speed === '0' ? 'landed' : 'flying');

  let update;
  if (motionState !== 'flying') {
    update = serializeMotion(
      spaceship,
      motionState,
      {
        position: rotateAttachedPosition(
          initialMotion.position,
          elapsedSeconds,
          body.rotationPeriodSeconds,
          collisionRadius,
        ),
        velocity: { x: 0, y: 0 },
      },
      simulatedAt,
    );
  } else {
    const stepCount = Math.min(
      MAX_PROPAGATION_STEPS,
      Math.max(1, Math.ceil(elapsedSeconds / TARGET_STEP_SECONDS)),
    );
    const stepSeconds = elapsedSeconds / stepCount;
    const gravitationalParameter = GRAVITATIONAL_CONSTANT * Number(body.mass);
    let motion = initialMotion;
    let impact: SpaceshipVelocity | undefined;
    let impactState: SpaceshipMotionState | undefined;

    for (let step = 0; step < stepCount; step += 1) {
      const nextMotion = integrateStep(
        motion,
        stepSeconds,
        gravitationalParameter,
      );
      impact = findImpact(
        motion.position,
        nextMotion.position,
        collisionRadius,
      );
      if (impact) {
        const surfaceVelocity = getSurfaceVelocity(
          impact,
          body.rotationPeriodSeconds,
        );
        const impactSpeed = Math.hypot(
          nextMotion.velocity.x - surfaceVelocity.x,
          nextMotion.velocity.y - surfaceVelocity.y,
        );
        impactState =
          impactSpeed > CRASH_SPEED_METERS_PER_SECOND ? 'crashed' : 'landed';
        motion = { position: impact, velocity: { x: 0, y: 0 } };
        break;
      }
      motion = nextMotion;
    }

    update = serializeMotion(
      spaceship,
      impactState ?? 'flying',
      motion,
      simulatedAt,
    );
  }

  const collection = (await getDatabase()).collection<SpaceshipDocument>(
    'spaceships',
  );
  const result = await collection.findOneAndUpdate(
    {
      securityCode: spaceship.securityCode,
      updatedAt: spaceship.updatedAt,
    },
    { $set: update },
    { returnDocument: 'after' },
  );
  return (
    result ??
    (await collection.findOne({ securityCode: spaceship.securityCode })) ??
    spaceship
  );
}
