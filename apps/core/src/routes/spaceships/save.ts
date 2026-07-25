import type { Request, Response } from 'express';
import { SpaceshipService } from '@services';
import { getRequiredSecurityCode, sendError } from '../../http';

export async function save(request: Request, response: Response) {
  const securityCode = getRequiredSecurityCode(request, response);
  if (!securityCode) return;

  try {
    const update = SpaceshipService.parseSpaceshipUpdate(request.body);
    const spaceship = await SpaceshipService.saveSpaceship(
      securityCode,
      update,
    );
    if (!spaceship) {
      sendError(response, 404, 'Spaceship not found');
      return;
    }

    response.json({ spaceship: SpaceshipService.toSpaceshipDto(spaceship) });
  } catch (error) {
    const message =
      error instanceof Error && error.message
        ? error.message
        : 'Failed to save spaceship';
    sendError(response, 400, message);
  }
}
