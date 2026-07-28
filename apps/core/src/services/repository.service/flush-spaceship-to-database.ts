import type { SpaceshipDocument } from '@models';
import { SpaceshipModel } from '@models';
import { cloneSpaceship } from './clone-spaceship';

export async function flushSpaceshipToDatabase(spaceship: SpaceshipDocument) {
  await SpaceshipModel.replaceSpaceship(cloneSpaceship(spaceship));
}
