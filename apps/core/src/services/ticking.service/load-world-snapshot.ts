import { RepositoryService } from '../repository.service';
import type { WorldSnapshot } from './types';

export async function loadWorldSnapshot(): Promise<WorldSnapshot> {
  const { planets, moons, stars } = await RepositoryService.getWorldData();
  const bodies = [...planets, ...moons, ...stars];
  return {
    bodies,
    bodiesByName: new Map(bodies.map((body) => [body.name, body])),
  };
}

