export const SPACESHIP_PHYSICS_LABEL = 'spaceship';
export const PLANET_PHYSICS_LABEL_PREFIX = 'planet:';

export function getPlanetPhysicsLabel(name: string) {
  return `${PLANET_PHYSICS_LABEL_PREFIX}${name}`;
}

export function getPlanetNameFromPhysicsLabel(label: string) {
  return label.startsWith(PLANET_PHYSICS_LABEL_PREFIX)
    ? label.slice(PLANET_PHYSICS_LABEL_PREFIX.length)
    : undefined;
}
