import type { NextFunction, Request, Response } from 'express';
import { getSecurityCode } from './services/spaceship';

export function spaceshipSecurityCode(
  request: Request,
  _response: Response,
  next: NextFunction,
) {
  request.securityCode = getSecurityCode(request.headers);
  next();
}
