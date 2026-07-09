import { Router } from 'express';
import { type SpaceshipDocument } from '@models';
import {
  OfflineSpaceshipService,
  OrbitalUpdaterService,
  RepositoryService,
} from '@services';
import { asyncHandler } from '../http';

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
    const result = await OrbitalUpdaterService.updateOrbitalBodies(
      getInvocationTime(request.body?.time),
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/moons',
  asyncHandler(async (request, response) => {
    const result = await OrbitalUpdaterService.updateOrbitalBodies(
      getInvocationTime(request.body?.time),
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/stars',
  asyncHandler(async (request, response) => {
    const result = await OrbitalUpdaterService.updateOrbitalBodies(
      getInvocationTime(request.body?.time),
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/spaceships',
  asyncHandler(async (request, response) => {
    const invocationTime = getInvocationTime(request.body?.time);
    const oldestSpaceships =
      await RepositoryService.findOldestSpaceshipsForSimulation(
        invocationTime,
        SPACESHIP_BATCH_SIZE,
      );

    if (oldestSpaceships.length === 0) {
      response.json({ selected: 0, processed: 0 });
      return;
    }

    const world = await OfflineSpaceshipService.loadOfflineWorld();
    await Promise.all(
      oldestSpaceships.map((spaceship: SpaceshipDocument) =>
        OfflineSpaceshipService.propagateOfflineSpaceship(
          spaceship,
          invocationTime,
          world,
        ),
      ),
    );

    response.json({
      selected: oldestSpaceships.length,
      processed: oldestSpaceships.length,
    });
  }),
);
