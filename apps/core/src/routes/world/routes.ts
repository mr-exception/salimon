import { Router } from 'express';
import { asyncHandler } from '../../http';
import { getSystems } from './get-systems';

export const worldRouter = Router();

worldRouter.post('/systems', asyncHandler(getSystems));
