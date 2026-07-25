import { Router } from 'express';
import { asyncHandler } from '../../http';
import { getSearch } from './get-search';

export const searchRouter = Router();

searchRouter.get('/', asyncHandler(getSearch));
