import { WorldViewportService } from '@services';
import { sendError } from '../../http';
import type { Request, Response } from 'express';

export async function getSystems(request: Request, response: Response) {
  try {
    const world = await WorldViewportService.getWorldSystems(request.body);
    response.json(world);
  } catch (error) {
    console.error('Failed to load world systems', error);
    sendError(response, 400, 'Failed to load world systems');
  }
}
