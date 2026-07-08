import {
  MongoClient,
  type AnyBulkWriteOperation,
  type Document,
  type WithId,
} from 'mongodb';

const BODY_BATCH_SIZE = 100;
const FULL_ROTATION_RADIANS = Math.PI * 2;

type SerializedPosition = {
  x: string;
  y: string;
  relativeTo?: string;
};

type OrbitalBody = Document & {
  position: SerializedPosition;
  orbitalCenter: string | null;
  clockwise: boolean;
  speed: string;
  updatedAt: Date;
};

type UpdateOptions = {
  collectionName: 'planets' | 'stars';
  orbitalCenterCollection?: 'planets' | 'stars';
};

let clientPromise: Promise<MongoClient> | undefined;

async function getClient() {
  const uri = process.env.MONGODB_URI;
  if (!uri) throw new Error('MONGODB_URI is not configured');

  clientPromise ??= new MongoClient(uri).connect().catch((error: unknown) => {
    clientPromise = undefined;
    throw error;
  });

  return clientPromise;
}

function getInvocationTime(time: string | Date): Date {
  const invocationTime = new Date(time);
  if (Number.isNaN(invocationTime.getTime())) {
    throw new Error('Invocation time is invalid');
  }

  return invocationTime;
}

function advancePosition(body: WithId<OrbitalBody>, elapsedSeconds: number) {
  const x = BigInt(body.position.x);
  const y = BigInt(body.position.y);
  const orbitalRadius = Math.hypot(Number(x), Number(y));
  const speed = Number(BigInt(body.speed));

  if (
    !body.orbitalCenter ||
    orbitalRadius === 0 ||
    speed === 0 ||
    elapsedSeconds <= 0
  ) {
    return body.position;
  }

  const direction = body.clockwise ? 1 : -1;
  const angle =
    ((direction * speed * elapsedSeconds) / orbitalRadius) %
    FULL_ROTATION_RADIANS;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);

  return {
    x: BigInt(Math.round(Number(x) * cos - Number(y) * sin)).toString(),
    y: BigInt(Math.round(Number(x) * sin + Number(y) * cos)).toString(),
    ...(body.position.relativeTo
      ? { relativeTo: body.position.relativeTo }
      : {}),
  };
}

async function findOldestBodies(invocationTime: Date, options: UpdateOptions) {
  const client = await getClient();
  const bodies = client.db().collection<OrbitalBody>(options.collectionName);
  const initialStages: Document[] = [
    { $match: { updatedAt: { $type: 'date', $lt: invocationTime } } },
  ];

  if (options.orbitalCenterCollection) {
    initialStages.push(
      {
        $lookup: {
          from: options.orbitalCenterCollection,
          localField: 'orbitalCenter',
          foreignField: 'name',
          as: '_orbitalCenters',
        },
      },
      { $match: { '_orbitalCenters.0': { $exists: true } } },
      { $unset: '_orbitalCenters' },
    );
  }

  const oldestBodies = await bodies
    .aggregate<
      WithId<OrbitalBody>
    >([...initialStages, { $sort: { updatedAt: 1 } }, { $limit: BODY_BATCH_SIZE }])
    .toArray();

  return { bodies, oldestBodies };
}

export async function updateOrbitalBodies(
  time: string | Date,
  options: UpdateOptions,
) {
  const invocationTime = getInvocationTime(time);
  const { bodies, oldestBodies } = await findOldestBodies(
    invocationTime,
    options,
  );

  if (oldestBodies.length === 0) {
    return { selected: 0, updated: 0 };
  }

  const updates: AnyBulkWriteOperation<OrbitalBody>[] = oldestBodies.map(
    (body) => {
      const elapsedSeconds =
        (invocationTime.getTime() - body.updatedAt.getTime()) / 1_000;

      return {
        updateOne: {
          // Including updatedAt prevents concurrent invocations from moving the
          // same body twice from the same starting position.
          filter: { _id: body._id, updatedAt: body.updatedAt },
          update: {
            $set: {
              position: advancePosition(body, elapsedSeconds),
              updatedAt: invocationTime,
            },
          },
        },
      };
    },
  );
  const result = await bodies.bulkWrite(updates, { ordered: false });

  return {
    selected: oldestBodies.length,
    updated: result.modifiedCount,
  };
}
