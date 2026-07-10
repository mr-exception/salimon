import type {
  SpaceshipDocument,
  SpaceshipMotionState,
  SpaceshipVelocity,
  WorldBodyDocument,
} from '@models';
import {
  SPACESHIP_THRUSTER_COUNT,
  SpaceshipService,
} from './spaceship.service';
import {
  CRASH_SPEED_METERS_PER_SECOND,
  GRAVITATIONAL_CONSTANT,
  MAX_ENGINE_THRUST_KN,
  SPACESHIP_MASS_KG,
  THRUSTER_DURABILITY_DRAIN_RATE,
} from './ticking.service/constants';
import type { Motion, WorldSnapshot } from './ticking.service/types';

export type ActiveThrusters = {
  availableSeconds: number;
  effectiveAcceleration: SpaceshipVelocity;
  thrustByIndex: number[];
  totalKilonewtons: number;
};

export class PhysicsService {
  static add(
    value: SpaceshipVelocity,
    change: SpaceshipVelocity,
    scale = 1,
  ): SpaceshipVelocity {
    return {
      x: value.x + change.x * scale,
      y: value.y + change.y * scale,
    };
  }

  static getBodyPositions(world: WorldSnapshot, time: Date) {
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
      const elapsedSeconds =
        (time.getTime() - body.updatedAt.getTime()) / 1_000;
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
        localPosition = PhysicsService.add(
          localPosition,
          resolve(reference, nextPath),
        );
      }

      positions.set(body.name, localPosition);
      return localPosition;
    }

    world.bodies.forEach((body) => resolve(body, new Set()));
    return positions;
  }

  static calculateGravityAcceleration(
    position: SpaceshipVelocity,
    world: WorldSnapshot,
    time: Date,
  ): SpaceshipVelocity {
    const bodyPositions = PhysicsService.getBodyPositions(world, time);
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

  static calculateAcceleration(
    position: SpaceshipVelocity,
    world: WorldSnapshot,
    time: Date,
    thrustAcceleration?: SpaceshipVelocity,
  ): SpaceshipVelocity {
    const gravityAcceleration = PhysicsService.calculateGravityAcceleration(
      position,
      world,
      time,
    );
    return {
      x: gravityAcceleration.x + (thrustAcceleration?.x ?? 0),
      y: gravityAcceleration.y + (thrustAcceleration?.y ?? 0),
    };
  }

  static integrateStep(
    motion: Motion,
    startedAt: Date,
    seconds: number,
    world: WorldSnapshot,
    thrustAcceleration?: SpaceshipVelocity,
  ): Motion {
    const midpoint = new Date(startedAt.getTime() + (seconds * 1_000) / 2);
    const finishedAt = new Date(startedAt.getTime() + seconds * 1_000);
    const position1 = motion.velocity;
    const velocity1 = PhysicsService.calculateAcceleration(
      motion.position,
      world,
      startedAt,
      thrustAcceleration,
    );
    const position2 = PhysicsService.add(
      motion.velocity,
      velocity1,
      seconds / 2,
    );
    const velocity2 = PhysicsService.calculateAcceleration(
      PhysicsService.add(motion.position, position1, seconds / 2),
      world,
      midpoint,
      thrustAcceleration,
    );
    const position3 = PhysicsService.add(
      motion.velocity,
      velocity2,
      seconds / 2,
    );
    const velocity3 = PhysicsService.calculateAcceleration(
      PhysicsService.add(motion.position, position2, seconds / 2),
      world,
      midpoint,
      thrustAcceleration,
    );
    const position4 = PhysicsService.add(motion.velocity, velocity3, seconds);
    const velocity4 = PhysicsService.calculateAcceleration(
      PhysicsService.add(motion.position, position3, seconds),
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

  static getActiveThrusters(
    accelerationValue: SpaceshipVelocity | undefined,
    stats: SpaceshipDocument['stats'],
  ): ActiveThrusters | undefined {
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

  static wearThrusters(
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

  static calculateRequiredBurnAcceleration(
    targetVelocity: SpaceshipVelocity,
    remainingSeconds: number,
    currentVelocity: SpaceshipVelocity,
    position: SpaceshipVelocity,
    world: WorldSnapshot,
    time: Date,
  ): SpaceshipVelocity {
    const desiredAcceleration = {
      x: (targetVelocity.x - currentVelocity.x) / remainingSeconds,
      y: (targetVelocity.y - currentVelocity.y) / remainingSeconds,
    };
    const gravityAcceleration = PhysicsService.calculateGravityAcceleration(
      position,
      world,
      time,
    );

    return {
      x: desiredAcceleration.x - gravityAcceleration.x,
      y: desiredAcceleration.y - gravityAcceleration.y,
    };
  }

  static calculateMaximumEngineAcceleration(maximumThrustPercent = 100) {
    return (
      ((MAX_ENGINE_THRUST_KN * 1_000) / SPACESHIP_MASS_KG) *
      (maximumThrustPercent / 100)
    );
  }

  static rotateAttachedPosition(
    position: SpaceshipVelocity,
    elapsedSeconds: number,
    rotationPeriodSeconds: number | undefined,
    collisionRadius: number,
  ): SpaceshipVelocity {
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

  static getSurfaceVelocity(
    position: SpaceshipVelocity,
    rotationPeriodSeconds: number | undefined,
  ): SpaceshipVelocity {
    if (!rotationPeriodSeconds || rotationPeriodSeconds <= 0) {
      return { x: 0, y: 0 };
    }
    const angularVelocity = (2 * Math.PI) / rotationPeriodSeconds;
    return {
      x: -position.y * angularVelocity,
      y: position.x * angularVelocity,
    };
  }

  static getImpactMotionState(impactSpeed: number): SpaceshipMotionState {
    return impactSpeed > CRASH_SPEED_METERS_PER_SECOND ? 'crashed' : 'landed';
  }
}
