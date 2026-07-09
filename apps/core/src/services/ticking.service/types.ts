import type {
  SpaceshipVelocity,
  WorldBodyDocument,
} from '@models';

export type Timer = ReturnType<typeof setInterval>;

export type Motion = {
  position: SpaceshipVelocity;
  velocity: SpaceshipVelocity;
};

export type WorldSnapshot = {
  bodies: WorldBodyDocument[];
  bodiesByName: Map<string, WorldBodyDocument>;
};

export type Impact = {
  body: WorldBodyDocument;
  fraction: number;
  relativePosition: SpaceshipVelocity;
};
