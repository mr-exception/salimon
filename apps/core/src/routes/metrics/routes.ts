import { Router } from 'express';
import { asyncHandler } from '../../http';
import { getMetrics } from './get-metrics';

export const metricsRouter = Router();

metricsRouter.get('/', asyncHandler(getMetrics));
