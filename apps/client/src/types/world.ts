export type Position = {
  x: bigint; // m
  y: bigint; // m
  relativeTo?: string;
};

type OrbitingBody = {
  position: Position;
  name: string;
  radius: bigint; // m
  mass: bigint; // kg
  orbitalCenter: string | null;
  clockwise: boolean;
  speed: bigint; // m/s
};

type RotatingBody = {
  rotationPeriodSeconds: number;
  rotationDegrees: number;
};

export type Planet = OrbitingBody &
  RotatingBody & {
    color: number; // Phaser-compatible RGB color
    variant: number; // each variant code defines the visual pattern
    shapeRenderZoomLevel: number;
    renderZoomLevel: number;
  };

export type Star = OrbitingBody &
  RotatingBody & {
    color: number; // Phaser-compatible RGB color
    variant: number; // each variant code defines the visual pattern
    shapeRenderZoomLevel: number;
    renderZoomLevel: number;
  };

export type Spaceship = OrbitingBody & {
  heading: number; // degrees; positive values rotate clockwise
};

export type World = {
  planets: Planet[];
  stars: Star[];
};

type SerializedBody<T extends Planet | Spaceship | Star> = Omit<
  T,
  'position' | 'radius' | 'mass' | 'speed'
> & {
  position: {
    x: string;
    y: string;
    relativeTo?: string;
  };
  radius: string;
  mass: string;
  speed: string;
};

export type SerializedWorld = {
  planets: SerializedBody<Planet>[];
  stars: SerializedBody<Star>[];
};
