import { Router } from 'express';
import { asyncHandler } from '../../http';
import { getInfo } from './get-info';
import { register } from './register';
import { updateInfo } from './update-info';

export const spaceshipsRouter = Router();

spaceshipsRouter.post('/register', asyncHandler(register));
spaceshipsRouter.get('/info', asyncHandler(getInfo));
spaceshipsRouter.put('/info', asyncHandler(updateInfo));
