import type { NextFunction, Request, Response } from 'express';

export function asyncHandler(
  handler: (request: Request, response: Response) => Promise<void>,
) {
  return (request: Request, response: Response, next: NextFunction) => {
    handler(request, response).catch(next);
  };
}

export function sendError(response: Response, status: number, error: string) {
  response.status(status).json({ error });
}

export function getRequiredSecurityCode(request: Request, response: Response) {
  const securityCode = request.securityCode;
  if (!securityCode) {
    sendError(
      response,
      400,
      'A valid x-spaceship-security-code header is required',
    );
    return undefined;
  }
  return securityCode;
}

declare global {
  // Express request augmentation uses namespace merging by design.
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace Express {
    interface Request {
      securityCode?: string;
    }
  }
}
