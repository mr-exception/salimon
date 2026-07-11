import { RepositoryService } from '../repository.service';

export async function loadSpaceship(securityCode: string) {
  const storedSpaceship =
    await RepositoryService.findSpaceshipBySecurityCode(securityCode);
  if (!storedSpaceship) return undefined;

  const { TickingService } = await import('../ticking.service/index.js');
  return TickingService.updateSpaceship(storedSpaceship);
}

