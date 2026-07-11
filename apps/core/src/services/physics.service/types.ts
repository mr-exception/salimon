import type { SpaceshipVelocity } from '@models';

export type ActiveThrusters = {
  availableSeconds: number;
  effectiveAcceleration: SpaceshipVelocity;
  thrustByIndex: number[];
  totalKilonewtons: number;
};

