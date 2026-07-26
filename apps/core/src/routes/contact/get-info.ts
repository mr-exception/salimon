import { ContactModel, ContactMessageModel } from '@models';
import { CONTACTS, ContactsService, RepositoryService } from '@services';
import { getRequiredSecurityCode, sendError } from '../../http';
import type { ContactInfo } from '@repo/types';
import type { Request, Response } from 'express';

export async function getInfo(request: Request, response: Response) {
  const securityCode = getRequiredSecurityCode(request, response);
  if (!securityCode) return;

  try {
    const spaceship =
      await RepositoryService.findSpaceshipBySecurityCode(securityCode);
    if (!spaceship) {
      sendError(response, 404, 'Spaceship not found');
      return;
    }

    let contacts = await ContactModel.findBySpaceshipSecurityCode(securityCode);
    if (contacts.length === 0) {
      await ContactsService.initializeSpaceshipContacts(securityCode);
      contacts = await ContactModel.findBySpaceshipSecurityCode(securityCode);
    }

    const contactInfo: ContactInfo[] = await Promise.all(
      contacts.flatMap(async (contact) => {
        const profile = CONTACTS[contact.contactId as keyof typeof CONTACTS];
        if (!profile) return [];
        const [latestMessage, unreadCount] = await Promise.all([
          ContactsService.findLatestMessage(securityCode, contact.contactId),
          ContactMessageModel.countUnreadContactMessages(
            securityCode,
            contact.contactId,
          ),
        ]);
        return [
          {
            id: profile.id,
            name: profile.name,
            organization: profile.organization,
            unreadCount,
            lastMessageAt: latestMessage?.createdAt.toISOString(),
          },
        ];
      }),
    ).then((groups) => groups.flat());

    response.json({
      contacts: contactInfo,
    });
  } catch (error) {
    console.error('Failed to load contacts', error);
    sendError(response, 500, 'Failed to load contacts');
  }
}
