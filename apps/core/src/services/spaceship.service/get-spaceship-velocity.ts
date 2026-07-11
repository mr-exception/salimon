import type { SpaceshipDocument } from '@models';

export function getSpaceshipVelocity(
  spaceship: Pick<SpaceshipDocument, 'direction' | 'speed' | 'velocity'>,
) {
  if (spaceship.velocity) return spaceship.velocity;

  const speed = Number(spaceship.speed);
  const headingRadians = (spaceship.direction * Math.PI) / 180;
  return {
    x: Math.sin(headingRadians) * speed,
    y: -Math.cos(headingRadians) * speed,
  };
}

