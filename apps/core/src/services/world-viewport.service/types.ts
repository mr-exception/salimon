import type { WorldBodyDocument } from '@models';
import type { Velocity } from '@repo/types';

export type Coordinate = {
  x: bigint;
  y: bigint;
};

export type WorldBodyResponse = Omit<WorldBodyDocument, 'updatedAt'> & {
  velocity: Velocity;
  positionCapturedAt: number;
};

export type PlanetSystem = {
  planet: WorldBodyResponse;
  moons: WorldBodyResponse[];
};

export type StarSystem = {
  star: WorldBodyResponse;
  planets: PlanetSystem[];
};

export type VisiblePlanetSystem = {
  planet: WorldBodyDocument;
  moons: WorldBodyDocument[];
};

export type WorldViewportRequest = {
  x?: string;
  y?: string;
  radius?: string;
  coordinate?: string;
  left?: string;
  right?: string;
  top?: string;
  bottom?: string;
};

export type WorldViewportOptions = {
  requiredBodyNames?: Iterable<string>;
};
