import { Router } from 'express';
import { asyncHandler, sendError } from '../http';
import { initializeSpaceshipContacts } from '../services/contacts';
import { propagateOfflineSpaceship } from '../services/offline-spaceship';
import {
  createSpaceship,
  getSpaceshipsCollection,
  parseSpaceshipUpdate,
  toSpaceshipDto,
} from '../services/spaceship';

export const spaceshipsRouter = Router();

spaceshipsRouter.post(
  '/register',
  asyncHandler(async (_request, response) => {
    try {
      const spaceship = createSpaceship();
      await (await getSpaceshipsCollection()).insertOne(spaceship);
      await initializeSpaceshipContacts(spaceship.securityCode);
      response.status(201).json({ spaceship: toSpaceshipDto(spaceship) });
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
      const storedSpaceship = await (
        await getSpaceshipsCollection()
      ).findOne({ securityCode });
      if (!storedSpaceship) {
        sendError(response, 404, 'Spaceship not found');
        return;
      }
      const spaceship = await propagateOfflineSpaceship(storedSpaceship);

      response.json({ spaceship: toSpaceshipDto(spaceship) });
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
      update = parseSpaceshipUpdate(request.body);
    } catch (error) {
      sendError(
        response,
        400,
        error instanceof Error ? error.message : 'Invalid request body',
      );
      return;
    }

    try {
      const now = new Date();
      const spaceship = await (
        await getSpaceshipsCollection()
      ).findOneAndUpdate(
        { securityCode },
        { $set: { ...update, simulatedAt: now, updatedAt: now } },
        { returnDocument: 'after' },
      );
      if (!spaceship) {
        sendError(response, 404, 'Spaceship not found');
        return;
      }

      response.json({ spaceship: toSpaceshipDto(spaceship) });
    } catch (error) {
      console.error('Failed to update spaceship', error);
      sendError(response, 500, 'Failed to update spaceship');
    }
  }),
);
