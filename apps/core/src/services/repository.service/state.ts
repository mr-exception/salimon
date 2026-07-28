import type { RepositoryState } from './types';

export const repositoryState: RepositoryState = {
  worldData: undefined,
  worldDataPromise: undefined,
  spaceshipsBySecurityCode: undefined,
  startPromise: undefined,
  databaseFlushTimer: undefined,
  databaseFlushPromise: undefined,
};

export function requireWorldData() {
  if (!repositoryState.worldData) {
    throw new Error('Repository world data has not been loaded');
  }

  return repositoryState.worldData;
}

export function requireSpaceshipsBySecurityCode() {
  if (!repositoryState.spaceshipsBySecurityCode) {
    throw new Error('Repository spaceship data has not been loaded');
  }

  return repositoryState.spaceshipsBySecurityCode;
}
