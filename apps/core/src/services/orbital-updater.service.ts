import {
  type AnyBulkWriteOperation,
  type WithId,
} from 'mongodb';
import {
  WorldBodyModel,
  type OrbitalCenterCollectionName,
  type WorldBodyCollectionName,
  type WorldBodyDocument,
} from '@models';

const BODY_BATCH_SIZE = 100;
const FULL_ROTATION_RADIANS = Math.PI * 2;

type UpdateOptions = {
  collectionName: WorldBodyCollectionName;
  orbitalCenterCollection?: OrbitalCenterCollectionName;
};

function getInvocationTime(time: string | Date): Date {
  const invocationTime = new Date(time);
  if (Number.isNaN(invocationTime.getTime())) {
    throw new Error('Invocation time is invalid');
  }

  return invocationTime;
}

function advancePosition(
  body: WithId<WorldBodyDocument>,
  elapsedSeconds: number,
) {
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

export class OrbitalUpdaterService {
  static async updateOrbitalBodies(
    time: string | Date,
    options: UpdateOptions,
  ) {
    const invocationTime = getInvocationTime(time);
    const { oldestBodies } = await WorldBodyModel.findOldestBodies(
      invocationTime,
      options,
      BODY_BATCH_SIZE,
    );

    if (oldestBodies.length === 0) {
      return { selected: 0, updated: 0 };
    }

    const updates: AnyBulkWriteOperation<WorldBodyDocument>[] =
      oldestBodies.map((body) => {
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
      });
    const result = await WorldBodyModel.bulkWrite(
      options.collectionName,
      updates,
    );

    return {
      selected: oldestBodies.length,
      updated: result.modifiedCount,
    };
  }
}
