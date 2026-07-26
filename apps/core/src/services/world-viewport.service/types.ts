import type { WorldBodyDocument } from '@models';
import type { Velocity } from '@repo/types';

export type Coordinate = {
  x: bigint;
  y: bigint;
};

export type WorldBodyResponse = Omit<WorldBodyDocument, 'updatedAt'> & {
  velocity: Velocity;
  cTime: number;
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
  x1?: string;
  y1?: string;
  x2?: string;
  y2?: string;
  radius?: string;
  coordinate?: string;
  left?: string;
  right?: string;
  top?: string;
  bottom?: string;
  zoom?: string | number;
  sectorX?: string | number;
  sectorY?: string | number;
  requiredBodyNames?: string | string[];
};

export type WorldViewportOptions = {
  requiredBodyNames?: Iterable<string>;
};
