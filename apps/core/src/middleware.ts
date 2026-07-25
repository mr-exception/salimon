import type { NextFunction, Request, Response } from 'express';
import { SpaceshipService } from '@services';

const NANOSECONDS_PER_MILLISECOND = 1_000_000;

export function apiRequestTiming(
  request: Request,
  response: Response,
  next: NextFunction,
) {
  const startedAt = process.hrtime.bigint();

  response.once('finish', () => {
    const durationMs =
      Number(process.hrtime.bigint() - startedAt) / NANOSECONDS_PER_MILLISECOND;

    console.info(
      `${request.method} ${request.originalUrl} ${response.statusCode} ${durationMs.toFixed(2)}ms`,
    );
  });

  next();
}

export function spaceshipSecurityCode(
  request: Request,
  _response: Response,
  next: NextFunction,
) {
  request.securityCode = SpaceshipService.getSecurityCode(request.headers);
  next();
}
