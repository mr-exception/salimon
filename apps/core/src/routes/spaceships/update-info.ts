import { SpaceshipService } from '@services';
import { getRequiredSecurityCode, sendError } from '../../http';
import type { Request, Response } from 'express';

export async function updateInfo(request: Request, response: Response) {
  const securityCode = getRequiredSecurityCode(request, response);
  if (!securityCode) return;

  try {
    const update = SpaceshipService.parseSpaceshipUpdate(request.body);
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
    sendError(response, 400, 'Failed to update spaceship');
  }
}
