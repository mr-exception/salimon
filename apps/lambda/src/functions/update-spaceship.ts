import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import {
  getSecurityCode,
  getSpaceshipsCollection,
  jsonResponse,
  parseSpaceshipUpdate,
  toSpaceshipDto,
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

  let update;
  try {
    update = parseSpaceshipUpdate(event);
  } catch (error) {
    return jsonResponse(400, {
      error: error instanceof Error ? error.message : 'Invalid request body',
    });
  }

  try {
    const spaceship = await (
      await getSpaceshipsCollection()
    ).findOneAndUpdate(
      { securityCode },
      { $set: { ...update, updatedAt: new Date() } },
      { returnDocument: 'after' },
    );
    if (!spaceship) return jsonResponse(404, { error: 'Spaceship not found' });

    return jsonResponse(200, { spaceship: toSpaceshipDto(spaceship) });
  } catch (error) {
    console.error('Failed to update spaceship', error);
    return jsonResponse(500, { error: 'Failed to update spaceship' });
  }
}
