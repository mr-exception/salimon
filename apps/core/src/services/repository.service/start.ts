import { loadFromDatabase } from './load-from-database';
import { repositoryState } from './state';

export async function start() {
  repositoryState.startPromise ??= loadFromDatabase();
  return repositoryState.startPromise;
}

