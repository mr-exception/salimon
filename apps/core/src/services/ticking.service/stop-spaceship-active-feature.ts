import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import { updateSpaceship } from './update-spaceship';

export async function stopSpaceshipActiveFeature(
  spaceship: SpaceshipDocument,
) {
  const simulatedAt = new Date();
  const currentSpaceship = await updateSpaceship(spaceship, simulatedAt);
  return RepositoryService.updatePropagatedSpaceship(currentSpaceship, {
    activeFeature: undefined,
    simulatedAt,
    updatedAt: simulatedAt,
  });
}

