import type {
  AnyBulkWriteOperation,
  Document,
  OptionalUnlessRequiredId,
  WithId,
} from 'mongodb';
import { DatabaseModel } from './database.model';

export type WorldBodyCollectionName = 'planets' | 'moons' | 'stars';
export type OrbitalCenterCollectionName = 'planets' | 'stars';

export type SerializedPosition = {
  x: string;
  y: string;
  relativeTo?: string;
};

export type WorldBodyDocument = Document & {
  name: string;
  position: SerializedPosition;
  orbitalCenter: string | null;
  clockwise: boolean;
  speed: string;
  mass: string;
  radius: string;
  rotationPeriodSeconds?: number;
  updatedAt: Date;
};

export class WorldBodyModel {
  static async getCollection(collectionName: WorldBodyCollectionName) {
    return (await DatabaseModel.getDatabase()).collection<WorldBodyDocument>(
      collectionName,
    );
  }

  static async findWorldSystemsBodies() {
    const projection = { _id: 0, updatedAt: 0 };
    const [planets, stars] = await Promise.all([
      (await WorldBodyModel.getCollection('planets'))
        .find({}, { projection })
        .toArray(),
      (await WorldBodyModel.getCollection('stars'))
        .find({}, { projection })
        .toArray(),
    ]);
    return { planets, stars };
  }

  static async findOfflineBodies() {
    const projection = {
      _id: 0,
      name: 1,
      position: 1,
      orbitalCenter: 1,
      clockwise: 1,
      speed: 1,
      mass: 1,
      radius: 1,
      rotationPeriodSeconds: 1,
      updatedAt: 1,
    };
    const collections = await Promise.all([
      WorldBodyModel.getCollection('planets'),
      WorldBodyModel.getCollection('moons'),
      WorldBodyModel.getCollection('stars'),
    ]);
    const bodyGroups = await Promise.all(
      collections.map((collection) =>
        collection.find({}, { projection }).toArray(),
      ),
    );
    return bodyGroups.flat();
  }

  static async findOldestBodies(
    invocationTime: Date,
    options: {
      collectionName: WorldBodyCollectionName;
      orbitalCenterCollection?: OrbitalCenterCollectionName;
    },
    batchSize: number,
  ) {
    const bodies = await WorldBodyModel.getCollection(options.collectionName);
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
        WithId<WorldBodyDocument>
      >([...initialStages, { $sort: { updatedAt: 1 } }, { $limit: batchSize }])
      .toArray();

    return { bodies, oldestBodies };
  }

  static async bulkWrite(
    collectionName: WorldBodyCollectionName,
    updates: AnyBulkWriteOperation<OptionalUnlessRequiredId<WorldBodyDocument>>[],
  ) {
    return (await WorldBodyModel.getCollection(collectionName)).bulkWrite(
      updates,
      { ordered: false },
    );
  }
}
