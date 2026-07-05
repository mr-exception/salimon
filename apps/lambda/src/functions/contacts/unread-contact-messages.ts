import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import {
  getContactMessagesCollection,
  initializeSpaceshipContacts,
  toMessageDto,
} from '../../contacts';
import {
  getSecurityCode,
  getSpaceshipsCollection,
  jsonResponse,
} from '../../spaceship';

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
    await initializeSpaceshipContacts(securityCode);

    const messages = await (
      await getContactMessagesCollection()
    )
      .find({
        spaceshipSecurityCode: securityCode,
        sender: 'contact',
        isRead: { $ne: true },
      })
      .sort({ createdAt: 1, _id: 1 })
      .toArray();

    return jsonResponse(200, { messages: messages.map(toMessageDto) });
  } catch (error) {
    console.error('Failed to load unread contact messages', error);
    return jsonResponse(500, {
      error: 'Failed to load unread contact messages',
    });
  }
}
