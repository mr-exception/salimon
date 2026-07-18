import { WorldSandbox } from '@repo/sandbox';
import { SpaceshipService } from '../spaceship.service';
import { SPACESHIP_RADIUS_METERS } from './constants';
import { start } from './start';
import { tickingState } from './state';

const EARTH_NAME = 'Earth';

export async function createSpaceship() {
  await start();

  const spaceship = SpaceshipService.createSpaceship();
  const earth = tickingState.sandbox?.getObject(
    WorldSandbox.getBodyObjectId(EARTH_NAME),
  );
  if (earth) {
    spaceship.position = {
      x: Math.round(earth.radius + SPACESHIP_RADIUS_METERS).toString(),
      y: '0',
      relativeTo: EARTH_NAME,
    };
  }

  tickingState.sandbox?.loadSpaceship(spaceship);

  return spaceship;
}
