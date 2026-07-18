import { SandBox, WorldSandbox, type SandboxCrashEvent } from '@repo/sandbox';
import { CRASH_SPEED_METERS_PER_SECOND, SPACESHIP_MASS_KG } from '@repo/world';
import { RepositoryService } from '../repository.service';
import { SpaceshipService } from '../spaceship.service';
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

  const spaceshipCrashThresholdN = getSpaceshipCrashThresholdN(sandbox);
  console.log(
    `Spaceship crash threshold: ${Math.round(spaceshipCrashThresholdN)}N`,
  );

  tickingState.unsubscribeFromSandboxTicks = sandbox.onObjectTick((object) => {
    void syncSandboxObject(sandbox, object).catch((error: unknown) => {
      console.error('Failed to sync sandbox object', error);
    });
  });
  tickingState.unsubscribeFromSandboxCrashes = sandbox.onCrash((crash) => {
    if (crash.forceN <= spaceshipCrashThresholdN) return;

    void registerSpaceshipCrash(sandbox, crash).catch((error: unknown) => {
      console.error('Failed to register spaceship crash', error);
    });
  });
  tickingState.sandbox = sandbox;
  sandbox.start();
}

function getSpaceshipCrashThresholdN(sandbox: WorldSandbox) {
  const earth = sandbox.getObject(WorldSandbox.getBodyObjectId('Earth'));

  if (!earth) {
    throw new Error('Earth object is required to calculate crash threshold.');
  }

  return SandBox.calculateCrashForceN(
    SPACESHIP_MASS_KG,
    earth.mass,
    CRASH_SPEED_METERS_PER_SECOND,
  );
}

async function registerSpaceshipCrash(
  sandbox: WorldSandbox,
  crash: SandboxCrashEvent,
) {
  const objectA = sandbox.getObject(crash.objectAId);
  const objectB = sandbox.getObject(crash.objectBId);
  const spaceshipObject =
    objectA?.kind === 'spaceship'
      ? objectA
      : objectB?.kind === 'spaceship'
        ? objectB
        : undefined;
  if (!spaceshipObject) return;

  const securityCode = sandbox.getSpaceshipSecurityCode(spaceshipObject);
  if (!securityCode) return;

  const crashedIntoObject =
    spaceshipObject.id === objectA?.id ? objectB : objectA;
  const crashedIntoBody =
    crashedIntoObject && crashedIntoObject.kind !== 'spaceship'
      ? crashedIntoObject
      : undefined;
  const position = crashedIntoBody
    ? {
        x: Math.round(
          spaceshipObject.position.x - crashedIntoBody.position.x,
        ).toString(),
        y: Math.round(
          spaceshipObject.position.y - crashedIntoBody.position.y,
        ).toString(),
        relativeTo: crashedIntoBody.name,
      }
    : {
        x: Math.round(spaceshipObject.position.x).toString(),
        y: Math.round(spaceshipObject.position.y).toString(),
      };
  const updatedAt = new Date();

  sandbox.crashSpaceship(securityCode, updatedAt.getTime());

  await RepositoryService.updateSpaceships((spaceshipsBySecurityCode) => {
    const spaceship = spaceshipsBySecurityCode.get(securityCode);
    if (!spaceship || spaceship.motionState === 'crashed') return 0;

    const motionState =
      spaceship.motionState ?? (spaceship.speed === '0' ? 'landed' : 'flying');
    if (motionState !== 'flying') return 0;

    const stats = SpaceshipService.normalizeSpaceshipStats(spaceship.stats);
    spaceshipsBySecurityCode.set(securityCode, {
      ...spaceship,
      position,
      velocity: { x: 0, y: 0 },
      speed: '0',
      motionState: 'crashed',
      stats: { ...stats, hullDurability: 0 },
      activeFeature: undefined,
      simulatedAt: updatedAt,
      updatedAt,
    });

    return 1;
  });
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
      activeFeature: sandbox.hasActiveForce(object)
        ? spaceship.activeFeature
        : undefined,
      simulatedAt: spaceshipSnapshot.simulatedAt,
      updatedAt: spaceshipSnapshot.updatedAt,
    });

    return 1;
  });
}
