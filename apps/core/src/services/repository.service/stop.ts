import { repositoryState } from './state';

export function stop() {
  if (repositoryState.databaseFlushTimer) {
    clearInterval(repositoryState.databaseFlushTimer);
    repositoryState.databaseFlushTimer = undefined;
  }

  repositoryState.startPromise = undefined;
  repositoryState.worldData = undefined;
  repositoryState.spaceshipsBySecurityCode = undefined;
}

