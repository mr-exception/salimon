import { SpaceshipModel, WorldBodyModel } from '@models';
import { cloneBody } from './clone-body';
import { cloneSpaceship } from './clone-spaceship';
import {
  repositoryState,
  requireSpaceshipsBySecurityCode,
  requireWorldData,
} from './state';
import { start } from './start';

export async function flushToDatabase() {
  await start();

  if (repositoryState.databaseFlushPromise) {
    return repositoryState.databaseFlushPromise;
  }

  repositoryState.databaseFlushPromise = (async () => {
    const worldData = requireWorldData();
    const spaceships = [...requireSpaceshipsBySecurityCode().values()];

    await Promise.all([
      ...(
        [
          ['planets', worldData.planets],
          ['moons', worldData.moons],
          ['stars', worldData.stars],
        ] as const
      ).map(([collectionName, bodies]) =>
        WorldBodyModel.replaceBodies(collectionName, bodies.map(cloneBody)),
      ),
      SpaceshipModel.replaceSpaceships(spaceships.map(cloneSpaceship)),
    ]);
  })().finally(() => {
    repositoryState.databaseFlushPromise = undefined;
  });

  return repositoryState.databaseFlushPromise;
}

