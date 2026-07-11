import { RepositoryService } from '../repository.service';
import { start } from './start';

export async function getWorldSystemsBodies() {
  await start();
  return RepositoryService.getWorldSystemsBodies();
}

