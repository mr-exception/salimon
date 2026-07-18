import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import { tickingState } from './state';

export async function stopSpaceshipActiveFeature(spaceship: SpaceshipDocument) {
  const simulatedAt = new Date();
  const snapshot = tickingState.sandbox?.stopSpaceshipForce(
    spaceship.securityCode,
    simulatedAt.getTime(),
  );

  return RepositoryService.updatePropagatedSpaceship(spaceship, {
    activeFeature: undefined,
    ...(snapshot
      ? {
          position: snapshot.position,
          velocity: snapshot.velocity,
          speed: snapshot.speed,
          direction: snapshot.direction,
        }
      : {}),
    simulatedAt,
    updatedAt: simulatedAt,
  });
}
