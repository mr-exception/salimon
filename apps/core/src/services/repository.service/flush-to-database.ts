import { SpaceshipModel } from '@models';
import { cloneSpaceship } from './clone-spaceship';
import { repositoryState, requireSpaceshipsBySecurityCode } from './state';
import { start } from './start';

export async function flushToDatabase() {
  await start();

  if (repositoryState.databaseFlushPromise) {
    return repositoryState.databaseFlushPromise;
  }

  repositoryState.databaseFlushPromise = (async () => {
    const spaceships = [...requireSpaceshipsBySecurityCode().values()];

    await SpaceshipModel.replaceSpaceships(spaceships.map(cloneSpaceship));
  })().finally(() => {
    repositoryState.databaseFlushPromise = undefined;
  });

  return repositoryState.databaseFlushPromise;
}
