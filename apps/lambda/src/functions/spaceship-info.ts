import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import {
  getSecurityCode,
  getSpaceshipsCollection,
  jsonResponse,
  toSpaceshipDto,
} from '../spaceship';
import { propagateOfflineSpaceship } from '../offline-spaceship';

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
    const storedSpaceship = await (
      await getSpaceshipsCollection()
    ).findOne({ securityCode });
    if (!storedSpaceship) {
      return jsonResponse(404, { error: 'Spaceship not found' });
    }
    const spaceship = await propagateOfflineSpaceship(storedSpaceship);

    return jsonResponse(200, { spaceship: toSpaceshipDto(spaceship) });
  } catch (error) {
    console.error('Failed to load spaceship', error);
    return jsonResponse(500, { error: 'Failed to load spaceship' });
  }
}
