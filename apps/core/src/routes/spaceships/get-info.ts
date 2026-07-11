import { SpaceshipService } from '@services';
import { getRequiredSecurityCode, sendError } from '../../http';
import type { Request, Response } from 'express';

export async function getInfo(request: Request, response: Response) {
  const securityCode = getRequiredSecurityCode(request, response);
  if (!securityCode) return;

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
}
