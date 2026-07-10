export type Position = {
  x: bigint;
  y: bigint;
  relativeTo?: string;
};

export type SerializedPosition = {
  x: string;
  y: string;
  relativeTo?: string;
};

export type Velocity = {
  x: number;
  y: number;
};

type OrbitingBody = {
  position: Position;
  positionCapturedAt?: number;
  name: string;
  radius: bigint;
  mass: bigint;
  orbitalCenter: string | null;
  clockwise: boolean;
  speed: bigint;
};

type RotatingBody = {
  rotationPeriodSeconds: number;
  rotationDegrees: number;
};

export type Planet = OrbitingBody &
  RotatingBody & {
    color: number;
    variant: number;
    shapeRenderZoomLevel: number;
    renderZoomLevel: number;
  };

export type Star = OrbitingBody &
  RotatingBody & {
    color: number;
    variant: number;
    shapeRenderZoomLevel: number;
    renderZoomLevel: number;
  };

export type Spaceship = Omit<OrbitingBody, 'positionCapturedAt'> & {
  positionCapturedAt?: string;
  heading: number;
};

export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';

export type SpaceshipStats = {
  fuelKns: number;
  hullDurability: number;
  thrusterDurability: number[];
};

export type SpaceshipActiveFeature = {
  type: 'target-speed';
  targetSpeedMetersPerSecond: number;
  maximumThrustPercent: number;
  targetDirection?: number;
  targetVelocity: Velocity;
  maximumAcceleration: number;
  durationSeconds: number;
  elapsedSeconds: number;
};

export type SpaceshipDto = {
  securityCode: string;
  position: SerializedPosition;
  positionCapturedAt?: string;
  direction: number;
  speed: string;
  velocity?: Velocity;
  motionState?: SpaceshipMotionState;
  stats?: Partial<SpaceshipStats> & Pick<SpaceshipStats, 'fuelKns'>;
  activeFeature?: SpaceshipActiveFeature;
  simulatedAt?: string;
};

export type World = {
  planets: Planet[];
  stars: Star[];
};

export type SerializedBody<T extends Planet | Spaceship | Star> = Omit<
  T,
  'position' | 'radius' | 'mass' | 'speed' | 'positionCapturedAt'
> & {
  position: SerializedPosition;
  radius: string;
  mass: string;
  speed: string;
  velocity?: Velocity;
  positionCapturedAt?: number;
};

export type SerializedWorld = {
  planets: SerializedBody<Planet>[];
  stars: SerializedBody<Star>[];
};

export type SerializedPlanetSystem = {
  planet: SerializedBody<Planet>;
  moons: SerializedBody<Planet>[];
};

export type SerializedStarSystem = {
  star: SerializedBody<Star>;
  planets: SerializedPlanetSystem[];
};

export type SerializedWorldSystems = {
  systems: SerializedStarSystem[];
};
