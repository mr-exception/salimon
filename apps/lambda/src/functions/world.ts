import type {
  APIGatewayProxyEventV2,
  APIGatewayProxyResultV2,
} from 'aws-lambda';
import { MongoClient, type Document } from 'mongodb';

type SerializedPosition = {
  x: string;
  y: string;
  relativeTo?: string;
};

type WorldBody = Document & {
  name: string;
  position: SerializedPosition;
};

type Coordinate = {
  x: bigint;
  y: bigint;
};

let clientPromise: Promise<MongoClient> | undefined;

function jsonResponse(
  statusCode: number,
  body: Record<string, unknown>,
): APIGatewayProxyResultV2 {
  return {
    statusCode,
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify(body),
  };
}

function parseInteger(value: string | undefined, name: string): bigint {
  if (!value || !/^-?\d+$/.test(value)) {
    throw new Error(`${name} must be an integer in meters`);
  }

  return BigInt(value);
}

function getSearchArea(event: APIGatewayProxyEventV2) {
  const parameters = event.queryStringParameters ?? {};
  const coordinate = parameters.coordinate?.split(',');
  const x = parseInteger(parameters.x ?? coordinate?.[0]?.trim(), 'x');
  const y = parseInteger(parameters.y ?? coordinate?.[1]?.trim(), 'y');
  const radius = parseInteger(parameters.radius, 'radius');

  if (radius < 0n) {
    throw new Error('radius must be greater than or equal to zero');
  }

  return { x, y, radius };
}

async function getClient() {
  const uri = process.env.MONGODB_URI;
  if (!uri) throw new Error('MONGODB_URI is not configured');

  clientPromise ??= new MongoClient(uri).connect().catch((error: unknown) => {
    clientPromise = undefined;
    throw error;
  });

  return clientPromise;
}

function resolvePositions(bodies: WorldBody[]) {
  const bodiesByName = new Map(bodies.map((body) => [body.name, body]));
  const positionsByName = new Map<string, Coordinate>();

  function resolve(body: WorldBody, ancestors = new Set<string>()): Coordinate {
    const cached = positionsByName.get(body.name);
    if (cached) return cached;
    if (ancestors.has(body.name)) {
      throw new Error(`Circular position reference involving ${body.name}`);
    }

    const position = {
      x: BigInt(body.position.x),
      y: BigInt(body.position.y),
    };
    const referenceName = body.position.relativeTo;
    if (referenceName) {
      const reference = bodiesByName.get(referenceName);
      if (!reference) {
        throw new Error(
          `Position reference ${referenceName} for ${body.name} was not found`,
        );
      }

      const nextAncestors = new Set(ancestors).add(body.name);
      const referencePosition = resolve(reference, nextAncestors);
      position.x += referencePosition.x;
      position.y += referencePosition.y;
    }

    positionsByName.set(body.name, position);
    return position;
  }

  bodies.forEach((body) => resolve(body));
  return positionsByName;
}

function isInsideCircle(
  position: Coordinate,
  center: Coordinate,
  radiusSquared: bigint,
) {
  const deltaX = position.x - center.x;
  const deltaY = position.y - center.y;
  return deltaX * deltaX + deltaY * deltaY <= radiusSquared;
}

export async function handler(
  event: APIGatewayProxyEventV2,
): Promise<APIGatewayProxyResultV2> {
  let searchArea;
  try {
    searchArea = getSearchArea(event);
  } catch (error) {
    return jsonResponse(400, {
      error: error instanceof Error ? error.message : 'Invalid query parameters',
    });
  }

  try {
    const client = await getClient();
    const database = client.db();
    const projection = { _id: 0, updatedAt: 0 };
    const [planets, stars] = await Promise.all([
      database
        .collection<WorldBody>('planets')
        .find({}, { projection })
        .toArray(),
      database.collection<WorldBody>('stars').find({}, { projection }).toArray(),
    ]);
    const positions = resolvePositions([...planets, ...stars]);
    const center = { x: searchArea.x, y: searchArea.y };
    const radiusSquared = searchArea.radius * searchArea.radius;
    const isIncluded = (body: WorldBody) =>
      isInsideCircle(positions.get(body.name)!, center, radiusSquared);

    return jsonResponse(200, {
      planets: planets.filter(isIncluded),
      stars: stars.filter(isIncluded),
    });
  } catch (error) {
    console.error('Failed to load world data', error);
    return jsonResponse(500, { error: 'Failed to load world data' });
  }
}
