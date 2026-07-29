export type Position = {
  x: bigint;
  y: bigint;
  relativeTo?: string;
  relativeToId?: string;
};

export type SerializedPosition = {
  x: string;
  y: string;
  relativeTo?: string;
  relativeToId?: string;
};

export type Velocity = {
  x: number;
  y: number;
};

type OrbitingBody = {
  id?: string;
  isReal?: boolean;
  position: Position;
  cTime?: number;
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
    type?: 'planet' | 'moon' | 'blackhole';
    color: number;
    variant: number;
    minZoomRenderShape: number;
    minZoomRenderName: number;
    shapeRenderZoomLevel?: number;
    renderZoomLevel?: number;
  };

export type Star = OrbitingBody &
  RotatingBody & {
    color: number;
    variant: number;
    minZoomRenderShape: number;
    minZoomRenderName: number;
    shapeRenderZoomLevel?: number;
    renderZoomLevel?: number;
  };

export type Spaceship = Omit<OrbitingBody, 'cTime'> & {
  positionCapturedAt?: string;
  heading: number;
};

export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';

export type SpaceshipStats = {
  fuelKns: number;
  hullDurability: number;
  thrusterDurability: number[];
};

export type InventoryMaterial =
  | 'iron'
  | 'silicates'
  | 'ice'
  | 'silver'
  | 'carbon'
  | 'gold'
  | 'hydrogen'
  | 'nitrogen';
export type SpaceshipInventory = Record<InventoryMaterial, number>;

export type SpaceshipTargetSpeedFeature = {
  type: 'target-speed';
  targetSpeedMetersPerSecond: number;
  maximumThrustPercent: number;
  targetDirection?: number;
  targetVelocity: Velocity;
  maximumAcceleration: number;
  durationSeconds: number;
  elapsedSeconds: number;
};

export type SpaceshipLockOnFeature = {
  type: 'lock-on';
  targetName: string;
  targetKind: 'Planet' | 'Star' | 'Asteroid' | 'Spaceship';
  targetSpeedMetersPerSecond: number;
  maximumThrustPercent: number;
  targetVelocity: Velocity;
  targetBodyVelocity: Velocity;
  targetPosition: Velocity;
  maximumAcceleration: number;
  durationSeconds: number;
  elapsedSeconds: number;
};

export type SpaceshipManualForceThruster = {
  powerPercent: number;
  active: boolean;
};

export type SpaceshipThrusterSchedule = SpaceshipManualForceThruster;

export type SpaceshipThrustersFeature = {
  type: 'thrusters';
  thrusters: SpaceshipManualForceThruster[];
  elapsedSeconds: number;
};

export type SpaceshipManualForceFeature = Omit<
  SpaceshipThrustersFeature,
  'type'
> & {
  type: 'manual-force';
};

export type SpaceshipActiveFeature =
  | SpaceshipTargetSpeedFeature
  | SpaceshipLockOnFeature
  | SpaceshipThrustersFeature
  | SpaceshipManualForceFeature;

export type SpaceshipDto = {
  securityCode: string;
  position: SerializedPosition;
  positionCapturedAt?: string;
  direction: number;
  speed: string;
  velocity?: Velocity;
  motionState?: SpaceshipMotionState;
  stats?: Partial<SpaceshipStats> & Pick<SpaceshipStats, 'fuelKns'>;
  inventory?: Partial<SpaceshipInventory>;
  activeFeature?: SpaceshipActiveFeature;
  simulatedAt?: string;
};

export type World = {
  planets: Planet[];
  stars: Star[];
};

export type SerializedBody<T extends Planet | Spaceship | Star> = Omit<
  T,
  'type' | 'position' | 'radius' | 'mass' | 'speed' | 'cTime'
> & {
  isReal: boolean;
  position: SerializedPosition;
  radius: string;
  mass: string;
  speed: string;
  velocity?: Velocity;
  cTime?: number;
};

export type SerializedWorld = {
  planets: SerializedBody<Planet>[];
  stars: SerializedBody<Star>[];
};

export type SerializedWorldBody =
  | (SerializedBody<Star> & { type: 'star' })
  | (SerializedBody<Planet> & {
      type: 'planet' | 'moon' | 'blackhole';
    });

export type SerializedWorldSystems = {
  systems: SerializedWorldBody[][];
};
