import { SearchService } from '@services';
import { sendError } from '../../http';
import type { Request, Response } from 'express';

export async function getSearch(request: Request, response: Response) {
  try {
    const results = await SearchService.searchByName(request.query.q);
    response.json({ results });
  } catch (error) {
    console.error('Failed to search world bodies', error);
    sendError(response, 400, 'Failed to search world bodies');
  }
}
