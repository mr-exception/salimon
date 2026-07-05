import type { ScheduledEvent } from 'aws-lambda';
import type { WithId } from 'mongodb';
import { propagateOfflineSpaceship } from '../offline-spaceship';
import { getSpaceshipsCollection, type SpaceshipDocument } from '../spaceship';

const SPACESHIP_BATCH_SIZE = 100;

export async function handler(event: ScheduledEvent) {
  const invocationTime = new Date(event.time);
  if (Number.isNaN(invocationTime.getTime())) {
    throw new Error('Scheduled event time is invalid');
  }

  const spaceships = await getSpaceshipsCollection();
  const oldestSpaceships = await spaceships
    .find({
      $or: [
        { simulatedAt: { $type: 'date', $lt: invocationTime } },
        {
          simulatedAt: { $exists: false },
          updatedAt: { $type: 'date', $lt: invocationTime },
        },
      ],
    })
    .sort({ simulatedAt: 1, updatedAt: 1 })
    .limit(SPACESHIP_BATCH_SIZE)
    .toArray();

  await Promise.all(
    oldestSpaceships.map((spaceship: WithId<SpaceshipDocument>) =>
      propagateOfflineSpaceship(spaceship, invocationTime),
    ),
  );

  return {
    selected: oldestSpaceships.length,
    processed: oldestSpaceships.length,
  };
}
