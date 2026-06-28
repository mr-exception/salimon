import type { APIGatewayProxyResultV2 } from 'aws-lambda';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const world = readFileSync(join(__dirname, 'world.json'), 'utf8');

export async function handler(): Promise<APIGatewayProxyResultV2> {
  return {
    statusCode: 200,
    headers: {
      'content-type': 'application/json',
    },
    body: world,
  };
}
