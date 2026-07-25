import { RepositoryService } from '../repository.service';
import { propagateSpaceshipToNow } from './propagate-spaceship';

export async function loadSpaceship(securityCode: string) {
  const spaceship =
    await RepositoryService.findSpaceshipBySecurityCode(securityCode);
  return spaceship ? propagateSpaceshipToNow(spaceship) : undefined;
}
