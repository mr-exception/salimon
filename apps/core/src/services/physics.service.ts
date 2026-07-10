import type {
  SpaceshipDocument,
  SpaceshipMotionState,
  SpaceshipVelocity,
} from '@models';
import { SPACESHIP_MASS_KG, WorldService } from '@repo/world';
import {
  SPACESHIP_THRUSTER_COUNT,
  SpaceshipService,
} from './spaceship.service';
import { THRUSTER_DURABILITY_DRAIN_RATE } from './ticking.service/constants';
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
    return WorldService.add(value, change, scale);
  }

  static getBodyPositions(world: WorldSnapshot, time: Date) {
    return WorldService.getBodyPositions(world, time);
  }

  static calculateGravityAcceleration(
    position: SpaceshipVelocity,
    world: WorldSnapshot,
    time: Date,
  ): SpaceshipVelocity {
    const bodyPositions = PhysicsService.getBodyPositions(world, time);
    return WorldService.calculateGravityAcceleration(
      position,
      world.bodies,
      (body) => bodyPositions.get(body.name),
    );
  }

  static calculateAcceleration(
    position: SpaceshipVelocity,
    world: WorldSnapshot,
    time: Date,
    thrustAcceleration?: SpaceshipVelocity,
  ): SpaceshipVelocity {
    return WorldService.calculateAcceleration(
      position,
      (currentPosition) =>
        PhysicsService.calculateGravityAcceleration(
          currentPosition,
          world,
          time,
        ),
      thrustAcceleration,
    );
  }

  static integrateStep(
    motion: Motion,
    startedAt: Date,
    seconds: number,
    world: WorldSnapshot,
    thrustAcceleration?: SpaceshipVelocity,
  ): Motion {
    return WorldService.integrateStep(motion, seconds, (position, offset) =>
      PhysicsService.calculateAcceleration(
        position,
        world,
        new Date(startedAt.getTime() + offset * 1_000),
        thrustAcceleration,
      ),
    );
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
    return WorldService.calculateRequiredBurnAcceleration(
      targetVelocity,
      remainingSeconds,
      currentVelocity,
      position,
      (currentPosition) =>
        PhysicsService.calculateGravityAcceleration(
          currentPosition,
          world,
          time,
        ),
    );
  }

  static calculateMaximumEngineAcceleration(maximumThrustPercent = 100) {
    return WorldService.calculateMaximumEngineAcceleration(
      maximumThrustPercent,
    );
  }

  static rotateAttachedPosition(
    position: SpaceshipVelocity,
    elapsedSeconds: number,
    rotationPeriodSeconds: number | undefined,
    collisionRadius: number,
  ): SpaceshipVelocity {
    return WorldService.rotateAttachedPosition(
      position,
      elapsedSeconds,
      rotationPeriodSeconds,
      collisionRadius,
    );
  }

  static getSurfaceVelocity(
    position: SpaceshipVelocity,
    rotationPeriodSeconds: number | undefined,
  ): SpaceshipVelocity {
    return WorldService.getSurfaceVelocity(position, rotationPeriodSeconds);
  }

  static getImpactMotionState(impactSpeed: number): SpaceshipMotionState {
    return WorldService.getImpactMotionState(impactSpeed);
  }
}
