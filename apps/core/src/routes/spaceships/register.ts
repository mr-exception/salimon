import {
  ContactsService,
  RepositoryService,
  SpaceshipService,
} from '@services';
import { sendError } from '../../http';
import type { Request, Response } from 'express';

export async function register(_request: Request, response: Response) {
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
}
