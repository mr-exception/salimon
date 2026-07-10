import { Router } from 'express';
import { WorldViewportService } from '@services';
import { asyncHandler, sendError } from '../http';

function getQueryValue(value: unknown) {
  return typeof value === 'string' ? value : undefined;
}

export const worldRouter = Router();

worldRouter.get(
  '/systems',
  asyncHandler(async (request, response) => {
    try {
      const systems = await WorldViewportService.getWorldSystems({
        x: getQueryValue(request.query.x),
        y: getQueryValue(request.query.y),
        radius: getQueryValue(request.query.radius),
        coordinate: getQueryValue(request.query.coordinate),
      });
      response.json(systems);
    } catch (error) {
      const message =
        error instanceof Error ? error.message : 'Failed to load world systems';
      const status = message.includes('must be') ? 400 : 500;
      if (status === 500) console.error('Failed to load world systems', error);
      sendError(response, status, message);
    }
  }),
);
