import { ContactMessageModel } from '@models';
import { ContactsService, RepositoryService } from '@services';
import { getRequiredSecurityCode, sendError } from '../../http';
import type { Request, Response } from 'express';

export async function getUnreadMessages(request: Request, response: Response) {
  const securityCode = getRequiredSecurityCode(request, response);
  if (!securityCode) return;

  try {
    const spaceship =
      await RepositoryService.findSpaceshipBySecurityCode(securityCode);
    if (!spaceship) {
      sendError(response, 404, 'Spaceship not found');
      return;
    }
    await ContactsService.initializeSpaceshipContacts(securityCode);

    const messages =
      await ContactMessageModel.findUnreadContactMessages(securityCode);

    response.json({ messages: messages.map(ContactsService.toMessageDto) });
  } catch (error) {
    console.error('Failed to load unread contact messages', error);
    sendError(response, 500, 'Failed to load unread contact messages');
  }
}
