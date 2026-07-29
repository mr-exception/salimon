import type {
  SerializedWorldSystems,
  SpaceshipActiveFeature,
  SpaceshipDto,
} from '@repo/types';
import type { Vector } from '@repo/world';
import type {
  SpaceshipMotionState,
  SpaceshipProximityTelemetry,
} from './world';

export type SimulationPositionSnapshot = {
  name: string;
  x: bigint;
  y: bigint;
};

export type SimulationFrameSnapshot = {
  elapsedSeconds: number;
  bodyPositions: SimulationPositionSnapshot[];
  spaceship: {
    x: bigint;
    y: bigint;
    relativeTo?: string;
    heading: number;
    speed: bigint;
    velocity?: Vector;
    attachedBodyName?: string;
  };
  motionState: SpaceshipMotionState;
  activeFeature?: SpaceshipActiveFeature;
  absoluteSpeed: number;
  proximityTelemetry?: SpaceshipProximityTelemetry;
  activeThrustVector?: Vector;
  activeThrusters: { powerPercent: number; active: boolean }[];
};

export type SimulationWorkerRequest =
  | {
      type: 'initialize-spaceship';
      requestId: number;
      request:
        | { type: 'new' }
        | { type: 'continue'; securityCode: string }
        | { type: 'claim'; securityCode: string };
    }
  | {
      type: 'hydrate-world';
      systems: SerializedWorldSystems;
    }
  | {
      type: 'hydrate-spaceship';
      spaceship: SpaceshipDto;
    }
  | {
      type: 'set-active-bodies';
      names?: string[];
    }
  | {
      type: 'set-heading';
      heading: number;
    }
  | {
      type: 'start-thrusters';
      thrusters: { powerPercent: number; active: boolean }[];
    }
  | {
      type: 'start-target-speed';
      targetSpeedMetersPerSecond: number;
      maximumThrustPercent: number;
      targetDirection?: number;
    }
  | {
      type: 'start-lock-on';
      targetName: string;
      targetKind: 'Planet' | 'Star' | 'Asteroid' | 'Spaceship';
      targetSpeedMetersPerSecond: number;
      maximumThrustPercent: number;
      targetPosition: Vector;
      targetVelocity: Vector;
    }
  | {
      type: 'stop-active-feature';
    }
  | {
      type: 'advance';
      requestId: number;
      elapsedSeconds: number;
    };

export type SimulationWorkerResponse =
  | {
      type: 'spaceship';
      requestId: number;
      spaceship: SpaceshipDto;
      snapshot: SimulationFrameSnapshot;
    }
  | {
      type: 'frame';
      requestId?: number;
      snapshot: SimulationFrameSnapshot;
    }
  | {
      type: 'error';
      message: string;
    };
