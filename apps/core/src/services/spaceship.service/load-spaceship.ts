import { RepositoryService } from '../repository.service';

export async function loadSpaceship(securityCode: string) {
  return RepositoryService.findSpaceshipBySecurityCode(securityCode);
}
