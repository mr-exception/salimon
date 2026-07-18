import { ContactsService } from '@services';
import { getRequiredSecurityCode, sendError } from '../../http';
import type { Request, Response } from 'express';

export async function sendMessage(request: Request, response: Response) {
  const securityCode = getRequiredSecurityCode(request, response);
  if (!securityCode) return;

  try {
    const messageRequest = ContactsService.parseSendMessageRequest(
      request.body,
    );
    const message = await ContactsService.sendMessage(
      securityCode,
      messageRequest,
    );
    response.status(201).json({ message: ContactsService.toMessageDto(message) });
  } catch (error) {
    console.error('Failed to send contact message', error);
    sendError(response, 400, 'Failed to send contact message');
  }
}
