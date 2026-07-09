import { Router } from 'express';
import {
  ContactsService,
  RepositoryService,
  SpaceshipService,
} from '@services';
import { asyncHandler, sendError } from '../http';

export const spaceshipsRouter = Router();

spaceshipsRouter.post(
  '/register',
  asyncHandler(async (_request, response) => {
    try {
      const spaceship = SpaceshipService.createSpaceship();
      await RepositoryService.insertSpaceship(spaceship);
      await ContactsService.initializeSpaceshipContacts(spaceship.securityCode);
      response
        .status(201)
        .json({ spaceship: SpaceshipService.toSpaceshipDto(spaceship) });
    } catch (error) {
      console.error('Failed to register spaceship', error);
      sendError(response, 500, 'Failed to register spaceship');
    }
  }),
);

spaceshipsRouter.get(
  '/info',
  asyncHandler(async (request, response) => {
    const securityCode = request.securityCode;
    if (!securityCode) {
      sendError(
        response,
        400,
        'A valid x-spaceship-security-code header is required',
      );
      return;
    }

    try {
      const spaceship = await SpaceshipService.loadSpaceship(securityCode);
      if (!spaceship) {
        sendError(response, 404, 'Spaceship not found');
        return;
      }

      response.json({ spaceship: SpaceshipService.toSpaceshipDto(spaceship) });
    } catch (error) {
      console.error('Failed to load spaceship', error);
      sendError(response, 500, 'Failed to load spaceship');
    }
  }),
);

spaceshipsRouter.put(
  '/update',
  asyncHandler(async (request, response) => {
    const securityCode = request.securityCode;
    if (!securityCode) {
      sendError(
        response,
        400,
        'A valid x-spaceship-security-code header is required',
      );
      return;
    }

    let update;
    try {
      update = SpaceshipService.parseSpaceshipUpdate(request.body);
    } catch (error) {
      sendError(
        response,
        400,
        error instanceof Error ? error.message : 'Invalid request body',
      );
      return;
    }

    try {
      const spaceship = await SpaceshipService.updateSpaceship(
        securityCode,
        update,
      );
      if (!spaceship) {
        sendError(response, 404, 'Spaceship not found');
        return;
      }

      response.json({ spaceship: SpaceshipService.toSpaceshipDto(spaceship) });
    } catch (error) {
      console.error('Failed to update spaceship', error);
      sendError(response, 500, 'Failed to update spaceship');
    }
  }),
);
