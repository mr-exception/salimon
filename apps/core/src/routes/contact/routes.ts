import { Router } from 'express';
import { asyncHandler } from '../../http';
import { getInfo } from './get-info';
import { getMessages } from './get-messages';
import { getUnreadMessages } from './get-unread-messages';
import { sendMessage } from './send-message';

export const contactsRouter = Router();

contactsRouter.get('/info', asyncHandler(getInfo));
contactsRouter.get('/messages', asyncHandler(getMessages));
contactsRouter.get('/messages/unread', asyncHandler(getUnreadMessages));
contactsRouter.post('/messages', asyncHandler(sendMessage));
