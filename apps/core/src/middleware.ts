import type { NextFunction, Request, Response } from 'express';
import { SpaceshipService } from '@services/spaceship.service';

export function spaceshipSecurityCode(
  request: Request,
  _response: Response,
  next: NextFunction,
) {
  request.securityCode = SpaceshipService.getSecurityCode(request.headers);
  next();
}
