import { Router } from 'express';
import { asyncHandler } from '../../http';
import { getInfo } from './get-info';
import { register } from './register';
import { save } from './save';
import { updateInfo } from './update-info';

export const spaceshipsRouter = Router();

spaceshipsRouter.post('/register', asyncHandler(register));
spaceshipsRouter.get('/info', asyncHandler(getInfo));
spaceshipsRouter.put('/info', asyncHandler(updateInfo));
spaceshipsRouter.post('/save', asyncHandler(save));
