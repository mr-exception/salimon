import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import type { parseSpaceshipUpdate } from './parse-spaceship-update';

export function updateSpaceship(
  securityCode: string,
  update: ReturnType<typeof parseSpaceshipUpdate>,
): Promise<SpaceshipDocument | undefined> {
  const now = new Date();
  return RepositoryService.updateSpaceshipBySecurityCode(securityCode, {
    ...update,
    simulatedAt: now,
    updatedAt: now,
  });
}

