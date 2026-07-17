import { MetricsService } from '@services';
import type { Request, Response } from 'express';

export async function getMetrics(_request: Request, response: Response) {
  response.json({ memory: MetricsService.getMemoryUsage() });
}
