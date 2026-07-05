import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import {
  CONTACTS,
  findLatestMessage,
  getContactMessagesCollection,
  getContactsCollection,
  initializeSpaceshipContacts,
} from '../contacts';
import {
  getSecurityCode,
  getSpaceshipsCollection,
  jsonResponse,
} from '../spaceship';

export async function handler(
  event: APIGatewayProxyEventV2,
): Promise<APIGatewayProxyResultV2> {
  const securityCode = getSecurityCode(event);
  if (!securityCode) {
    return jsonResponse(400, {
      error: 'A valid x-spaceship-security-code header is required',
    });
  }

  try {
    const spaceship = await (
      await getSpaceshipsCollection()
    ).findOne({ securityCode });
    if (!spaceship) {
      return jsonResponse(404, { error: 'Spaceship not found' });
    }

    let contacts = await (await getContactsCollection())
      .find({ spaceshipSecurityCode: securityCode })
      .toArray();
    if (contacts.length === 0) {
      await initializeSpaceshipContacts(securityCode);
      contacts = await (await getContactsCollection())
        .find({ spaceshipSecurityCode: securityCode })
        .toArray();
    }
    const messages = await getContactMessagesCollection();

    return jsonResponse(200, {
      contacts: await Promise.all(
        contacts.flatMap(async (contact) => {
          const profile = CONTACTS[contact.contactId as keyof typeof CONTACTS];
          if (!profile) return [];
          const [latestMessage, unreadCount] = await Promise.all([
            findLatestMessage(messages, securityCode, contact.contactId),
            messages.countDocuments({
              spaceshipSecurityCode: securityCode,
              contactId: contact.contactId,
              sender: 'contact',
              ...(contact.lastReadAt
                ? { createdAt: { $gt: contact.lastReadAt } }
                : {}),
            }),
          ]);
          return [
            {
              ...profile,
              unreadCount,
              lastMessageAt: latestMessage?.createdAt.toISOString(),
            },
          ];
        }),
      ).then((groups) => groups.flat()),
    });
  } catch (error) {
    console.error('Failed to load contacts', error);
    return jsonResponse(500, { error: 'Failed to load contacts' });
  }
}
