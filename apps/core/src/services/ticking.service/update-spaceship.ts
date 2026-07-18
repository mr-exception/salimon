import type { SpaceshipDocument } from '@models';
import { WorldSandbox } from '@repo/sandbox';
import { RepositoryService } from '../repository.service';
import { start } from './start';
import { tickingState } from './state';

export async function updateSpaceship(
  spaceship: SpaceshipDocument,
  simulatedAt = new Date(),
) {
  await start();

  const sandbox = tickingState.sandbox;
  if (!sandbox) return spaceship;

  const objectId = WorldSandbox.getSpaceshipObjectId(spaceship.securityCode);
  const object =
    sandbox.getObject(objectId) ?? sandbox.loadSpaceship(spaceship);
  if (!object) return spaceship;

  sandbox.tick(simulatedAt.getTime());
  const snapshot = sandbox.getSpaceshipSnapshot(object);
  if (!snapshot) return spaceship;

  return RepositoryService.updatePropagatedSpaceship(spaceship, {
    position: snapshot.position,
    velocity: snapshot.velocity,
    speed: snapshot.speed,
    direction: snapshot.direction,
    activeFeature: sandbox.hasActiveForce(object)
      ? spaceship.activeFeature
      : undefined,
    simulatedAt: snapshot.simulatedAt,
    updatedAt: snapshot.updatedAt,
  });
}
