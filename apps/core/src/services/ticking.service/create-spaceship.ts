import { SpaceshipService } from '../spaceship.service';
import { tickingState } from './state';

export async function createSpaceship() {
  const spaceship = SpaceshipService.createSpaceship();

  tickingState.sandbox?.loadSpaceship(spaceship);

  return spaceship;
}
