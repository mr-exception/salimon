import type { APIGatewayProxyResultV2 } from 'aws-lambda';
import {
  createSpaceship,
  getSpaceshipsCollection,
  jsonResponse,
  toSpaceshipDto,
} from '../spaceship';

export async function handler(): Promise<APIGatewayProxyResultV2> {
  try {
    const spaceship = createSpaceship();
    await (await getSpaceshipsCollection()).insertOne(spaceship);
    return jsonResponse(201, { spaceship: toSpaceshipDto(spaceship) });
  } catch (error) {
    console.error('Failed to register spaceship', error);
    return jsonResponse(500, { error: 'Failed to register spaceship' });
  }
}
