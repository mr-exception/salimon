import { Router } from 'express';
import { TickingService } from '@services';
import { asyncHandler } from '../http';

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
    const result = await TickingService.updateWorldBodies(
      getInvocationTime(request.body?.time),
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/moons',
  asyncHandler(async (request, response) => {
    const result = await TickingService.updateWorldBodies(
      getInvocationTime(request.body?.time),
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/stars',
  asyncHandler(async (request, response) => {
    const result = await TickingService.updateWorldBodies(
      getInvocationTime(request.body?.time),
    );
    response.json(result);
  }),
);

updatesRouter.post(
  '/spaceships',
  asyncHandler(async (request, response) => {
    const result = await TickingService.updateSpaceships(
      getInvocationTime(request.body?.time),
    );
    response.json(result);
  }),
);
