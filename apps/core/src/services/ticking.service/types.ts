import type {
  SpaceshipActiveFeature,
  SpaceshipVelocity,
  WorldBodyDocument,
} from '@models';

export type Timer = ReturnType<typeof setInterval>;

export type Motion = {
  position: SpaceshipVelocity;
  velocity: SpaceshipVelocity;
};

export type TargetSpeedBurnPlan = Extract<
  SpaceshipActiveFeature,
  { type: 'target-speed' }
>;

export type ThrustersPlan = Extract<
  SpaceshipActiveFeature,
  { type: 'thrusters' | 'manual-force' }
>;

export type WorldSnapshot = {
  bodies: WorldBodyDocument[];
  bodiesByName: Map<string, WorldBodyDocument>;
};

export type Impact = {
  body: WorldBodyDocument;
  fraction: number;
  relativePosition: SpaceshipVelocity;
};
