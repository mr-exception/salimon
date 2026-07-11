import { SpaceshipModel, WorldBodyModel } from '@models';
import { cloneSpaceship } from './clone-spaceship';
import { cloneWorldData } from './clone-world-data';
import { repositoryState } from './state';
import { flushToDatabase } from './flush-to-database';

const DATABASE_FLUSH_INTERVAL_MS = 5 * 60 * 1_000;

export async function loadFromDatabase() {
  const [worldData, spaceships] = await Promise.all([
    WorldBodyModel.findAllWorldBodies(),
    SpaceshipModel.findAll(),
  ]);

  repositoryState.worldData = cloneWorldData(worldData);
  repositoryState.spaceshipsBySecurityCode = new Map(
    spaceships.map((spaceship) => [
      spaceship.securityCode,
      cloneSpaceship(spaceship),
    ]),
  );

  repositoryState.databaseFlushTimer ??= setInterval(() => {
    void flushToDatabase().catch((error: unknown) => {
      console.error('Failed to store repository data', error);
    });
  }, DATABASE_FLUSH_INTERVAL_MS);
}

