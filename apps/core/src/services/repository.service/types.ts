import type {
  SpaceshipDocument,
  WorldBodyCollectionName,
  WorldBodyDocument,
} from '@models';

export type WorldData = Record<WorldBodyCollectionName, WorldBodyDocument[]>;
export type Timer = ReturnType<typeof setInterval>;
export type RepositoryState = {
  worldData: WorldData | undefined;
  worldDataPromise: Promise<WorldData> | undefined;
  spaceshipsBySecurityCode: Map<string, SpaceshipDocument> | undefined;
  startPromise: Promise<void> | undefined;
  databaseFlushTimer: Timer | undefined;
  databaseFlushPromise: Promise<void> | undefined;
};
