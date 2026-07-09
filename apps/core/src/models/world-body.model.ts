import type { AnyBulkWriteOperation, PipelineStage, Types } from 'mongoose';
import { getModelForClass, modelOptions, prop } from '@typegoose/typegoose';
import { DatabaseModel } from './database.model';

export type WorldBodyCollectionName = 'planets' | 'moons' | 'stars';
export type OrbitalCenterCollectionName = 'planets' | 'stars';

export type SerializedPosition = {
  x: string;
  y: string;
  relativeTo?: string;
};

class SerializedPositionSchema implements SerializedPosition {
  @prop({ required: true })
  public x!: string;

  @prop({ required: true })
  public y!: string;

  @prop()
  public relativeTo?: string;
}

@modelOptions({ schemaOptions: { versionKey: false } })
class WorldBodySchema {
  @prop({ required: true })
  public name!: string;

  @prop({ required: true, type: () => SerializedPositionSchema })
  public position!: SerializedPosition;

  @prop({ default: null, type: () => String })
  public orbitalCenter!: string | null;

  @prop({ required: true })
  public clockwise!: boolean;

  @prop({ required: true })
  public speed!: string;

  @prop({ required: true })
  public mass!: string;

  @prop({ required: true })
  public radius!: string;

  @prop()
  public rotationPeriodSeconds?: number;

  @prop({ required: true })
  public updatedAt!: Date;
}

export type WorldBodyDocument = WorldBodySchema & { _id?: Types.ObjectId };

const PlanetTypegooseModel = getModelForClass(WorldBodySchema, {
  options: { customName: 'PlanetBody' },
  schemaOptions: { collection: 'planets', versionKey: false },
});
const MoonTypegooseModel = getModelForClass(WorldBodySchema, {
  options: { customName: 'MoonBody' },
  schemaOptions: { collection: 'moons', versionKey: false },
});
const StarTypegooseModel = getModelForClass(WorldBodySchema, {
  options: { customName: 'StarBody' },
  schemaOptions: { collection: 'stars', versionKey: false },
});

export class WorldBodyModel {
  static async getModel(collectionName: WorldBodyCollectionName) {
    await DatabaseModel.connect();
    if (collectionName === 'planets') return PlanetTypegooseModel;
    if (collectionName === 'moons') return MoonTypegooseModel;
    return StarTypegooseModel;
  }

  static async findWorldSystemsBodies() {
    const [planets, stars] = await Promise.all([
      (await WorldBodyModel.getModel('planets'))
        .find({}, { _id: 0, updatedAt: 0 })
        .lean<WorldBodyDocument[]>()
        .exec(),
      (await WorldBodyModel.getModel('stars'))
        .find({}, { _id: 0, updatedAt: 0 })
        .lean<WorldBodyDocument[]>()
        .exec(),
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
    const models = await Promise.all([
      WorldBodyModel.getModel('planets'),
      WorldBodyModel.getModel('moons'),
      WorldBodyModel.getModel('stars'),
    ]);
    const bodyGroups = await Promise.all(
      models.map((model) =>
        model.find({}, projection).lean<WorldBodyDocument[]>().exec(),
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
    const bodies = await WorldBodyModel.getModel(options.collectionName);
    const initialStages: PipelineStage[] = [
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
      .aggregate<WorldBodyDocument>([
        ...initialStages,
        { $sort: { updatedAt: 1 } },
        { $limit: batchSize },
      ])
      .exec();

    return { oldestBodies };
  }

  static async bulkWrite(
    collectionName: WorldBodyCollectionName,
    updates: AnyBulkWriteOperation<WorldBodyDocument>[],
  ) {
    return (await WorldBodyModel.getModel(collectionName)).bulkWrite(updates, {
      ordered: false,
    });
  }
}
