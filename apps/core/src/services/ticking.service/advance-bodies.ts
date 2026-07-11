import { RepositoryService } from '../repository.service';
import { advanceBodyPosition } from './advance-body-position';

export async function advanceBodies(invocationTime: Date) {
  let updated = 0;

  await RepositoryService.updateWorldBodies((worldData) => {
    for (const body of [
      ...[...worldData.stars].sort((left, right) =>
        left.name.localeCompare(right.name),
      ),
      ...[...worldData.planets].sort((left, right) =>
        left.name.localeCompare(right.name),
      ),
      ...[...worldData.moons].sort((left, right) =>
        left.name.localeCompare(right.name),
      ),
    ]) {
      const elapsedSeconds = Math.max(
        0,
        (invocationTime.getTime() - body.updatedAt.getTime()) / 1_000,
      );
      body.position = advanceBodyPosition(body, elapsedSeconds);
      body.updatedAt = invocationTime;
      updated += 1;
    }

    return updated;
  });

  return updated;
}

