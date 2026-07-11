import { Router } from 'express';
import { asyncHandler } from '../../http';
import { getInfo } from './get-info';
import { register } from './register';

export const spaceshipsRouter = Router();

spaceshipsRouter.post('/register', asyncHandler(register));
spaceshipsRouter.get('/info', asyncHandler(getInfo));
