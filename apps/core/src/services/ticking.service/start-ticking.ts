import { WorldSandbox } from '@repo/sandbox';
import { RepositoryService } from '../repository.service';
import { tickingState } from './state';
import type { SpaceshipDocument } from '@models';
import type { SandboxObject } from '@repo/sandbox';

export async function startTicking() {
  console.log('Starting world ticking');
  await RepositoryService.start();
  console.log('Repository data loaded for world ticking');

  if (tickingState.sandbox) return;

  const sandbox = new WorldSandbox();
  const worldData = await RepositoryService.getWorldData();
  let spaceships: SpaceshipDocument[] = [];
  await RepositoryService.updateSpaceships((spaceshipsBySecurityCode) => {
    spaceships = [...spaceshipsBySecurityCode.values()];
    return 0;
  });

  console.log(
    `Loading world sandbox objects, stars: ${worldData.stars.length}, planets: ${worldData.planets.length}, moons: ${worldData.moons.length}, spaceships: ${spaceships.length}`,
  );
  sandbox.batchObjects(() => {
    sandbox.loadBodies(worldData.stars, 'star');
    sandbox.loadBodies(worldData.planets, 'planet');
    sandbox.loadBodies(worldData.moons, 'moon');
    sandbox.loadSpaceships(spaceships);
  });
  console.log(`World sandbox loaded, objects: ${sandbox.listObjects().length}`);

  tickingState.unsubscribeFromSandboxTicks = sandbox.onObjectTick((object) => {
    void syncSandboxObject(sandbox, object).catch((error: unknown) => {
      console.error('Failed to sync sandbox object', error);
    });
  });
  tickingState.sandbox = sandbox;
  sandbox.start();
}

async function syncSandboxObject(sandbox: WorldSandbox, object: SandboxObject) {
  const bodySnapshot = sandbox.getBodySnapshot(object);
  if (bodySnapshot) {
    const kind = sandbox.getBodyKind(object);
    const collectionName =
      kind === 'star' ? 'stars' : kind === 'moon' ? 'moons' : 'planets';

    await RepositoryService.updateWorldBodies((worldData) => {
      const body = worldData[collectionName].find(
        (item) => item.name === bodySnapshot.name,
      );
      if (!body) return 0;

      body.position = bodySnapshot.position;
      body.updatedAt = bodySnapshot.updatedAt;
      return 1;
    });
    return;
  }

  const spaceshipSnapshot = sandbox.getSpaceshipSnapshot(object);
  if (!spaceshipSnapshot) return;

  await RepositoryService.updateSpaceships((spaceshipsBySecurityCode) => {
    const spaceship = spaceshipsBySecurityCode.get(
      spaceshipSnapshot.securityCode,
    );
    if (!spaceship) return 0;

    const motionState =
      spaceship.motionState ?? (spaceship.speed === '0' ? 'landed' : 'flying');
    spaceshipsBySecurityCode.set(spaceship.securityCode, {
      ...spaceship,
      position: spaceshipSnapshot.position,
      velocity:
        motionState === 'flying' ? spaceshipSnapshot.velocity : { x: 0, y: 0 },
      speed: motionState === 'flying' ? spaceshipSnapshot.speed : '0',
      direction:
        motionState === 'flying'
          ? spaceshipSnapshot.direction
          : spaceship.direction,
      simulatedAt: spaceshipSnapshot.simulatedAt,
      updatedAt: spaceshipSnapshot.updatedAt,
    });

    return 1;
  });
}
