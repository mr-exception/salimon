import { Router } from 'express';
import type { WithId } from 'mongodb';
import { asyncHandler } from '../http';
import {
  loadOfflineWorld,
  propagateOfflineSpaceship,
} from '../services/offline-spaceship';
import { updateOrbitalBodies } from '../services/orbital-updater';
import {
  getSpaceshipsCollection,
  type SpaceshipDocument,
} from '../services/spaceship';

const SPACESHIP_BATCH_SIZE = 100;

function getInvocationTime(value: unknown) {
  const date = typeof value === 'string' ? new Date(value) : new Date();
  if (Number.isNaN(date.getTime())) {
    throw new Error('time must be an ISO date string');
  }
  return date;
}

export const updatesRouter = Router();

updatesRouter.post(
  '/planets',
  asyncHandler(async (request, response) => {
    const result = await updateOrbitalBodies(
      getInvocationTime(request.body?.time),
      {
        collectionName: 'planets',
        orbitalCenterCollection: 'stars',
      },
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/moons',
  asyncHandler(async (request, response) => {
    const result = await updateOrbitalBodies(
      getInvocationTime(request.body?.time),
      {
        collectionName: 'planets',
        orbitalCenterCollection: 'planets',
      },
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/stars',
  asyncHandler(async (request, response) => {
    const result = await updateOrbitalBodies(
      getInvocationTime(request.body?.time),
      {
        collectionName: 'stars',
      },
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/spaceships',
  asyncHandler(async (request, response) => {
    const invocationTime = getInvocationTime(request.body?.time);
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

    if (oldestSpaceships.length === 0) {
      response.json({ selected: 0, processed: 0 });
      return;
    }

    const world = await loadOfflineWorld();
    await Promise.all(
      oldestSpaceships.map((spaceship: WithId<SpaceshipDocument>) =>
        propagateOfflineSpaceship(spaceship, invocationTime, world),
      ),
    );

    response.json({
      selected: oldestSpaceships.length,
      processed: oldestSpaceships.length,
    });
  }),
);
