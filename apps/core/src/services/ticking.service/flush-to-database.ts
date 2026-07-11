import { RepositoryService } from '../repository.service';

export async function flushToDatabase() {
  return RepositoryService.flushToDatabase();
}

