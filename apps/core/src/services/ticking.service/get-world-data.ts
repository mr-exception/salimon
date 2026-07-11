import { RepositoryService } from '../repository.service';
import { start } from './start';

export async function getWorldData() {
  await start();
  return RepositoryService.getWorldData();
}

