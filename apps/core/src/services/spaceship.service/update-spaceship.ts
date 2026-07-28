import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import type { parseSpaceshipUpdate } from './parse-spaceship-update';

type SpaceshipUpdate = ReturnType<typeof parseSpaceshipUpdate>;

export function updateSpaceship(
  securityCode: string,
  update: SpaceshipUpdate,
): Promise<SpaceshipDocument | undefined> {
  const now = new Date();
  return RepositoryService.updateSpaceshipBySecurityCode(securityCode, {
    ...update,
    simulatedAt: now,
    updatedAt: now,
  });
}

export async function saveSpaceship(
  securityCode: string,
  update: SpaceshipUpdate,
): Promise<SpaceshipDocument | undefined> {
  if (update.motionState !== 'flying') {
    throw new Error('Spaceship must be free flying to save');
  }
  if (update.activeFeature) {
    throw new Error('All thrusters must be off to save');
  }

  return updateSpaceship(securityCode, update);
}
