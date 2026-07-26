import { mongoose } from '@typegoose/typegoose';
import type { SerializedWorldBody } from '@repo/types';
import { DatabaseModel } from './database.model';

export type WorldSystemDocument = {
  name: string;
  bodies: SerializedWorldBody[];
  primaryPosition?: {
    x: typeof mongoose.Types.Decimal128.prototype;
    y: typeof mongoose.Types.Decimal128.prototype;
  };
  generatedAt?: Date;
  updatedAt?: Date;
};

export type WorldSystemViewport = {
  left: bigint;
  right: bigint;
  top: bigint;
  bottom: bigint;
};

const WorldSystemSchema = new mongoose.Schema(
  {
    name: { required: true, type: String, unique: true },
    primaryPosition: {
      x: { type: mongoose.Schema.Types.Decimal128 },
      y: { type: mongoose.Schema.Types.Decimal128 },
    },
    bodies: { required: true, type: [mongoose.Schema.Types.Mixed] },
    generatedAt: { type: Date },
    updatedAt: { type: Date },
  },
  { collection: 'systems', versionKey: false },
);

const WorldSystemMongooseModel =
  mongoose.models.WorldSystem ??
  mongoose.model<WorldSystemDocument>(
    'WorldSystem',
    WorldSystemSchema as mongoose.Schema<WorldSystemDocument>,
  );

export class WorldSystemModel {
  static async getModel() {
    await DatabaseModel.connect();
    return WorldSystemMongooseModel;
  }

  static async findAllSystems() {
    return (await WorldSystemModel.getModel())
      .find({}, { _id: 0 })
      .lean<WorldSystemDocument[]>()
      .exec();
  }

  static async findSystemsInViewport(
    viewport: WorldSystemViewport,
    requiredBodyNames: Iterable<string> = [],
  ) {
    const requiredNames = [...requiredBodyNames];
    const left = mongoose.Types.Decimal128.fromString(viewport.left.toString());
    const right = mongoose.Types.Decimal128.fromString(
      viewport.right.toString(),
    );
    const top = mongoose.Types.Decimal128.fromString(viewport.top.toString());
    const bottom = mongoose.Types.Decimal128.fromString(
      viewport.bottom.toString(),
    );
    const indexedViewportMatch = {
      'primaryPosition.x': { $gte: left, $lte: right },
      'primaryPosition.y': { $gte: top, $lte: bottom },
    };
    const legacyViewportMatch = {
      $and: [
        { primaryPosition: { $exists: false } },
        {
          $expr: {
            $let: {
              vars: {
                primaryBody: {
                  $ifNull: [
                    {
                      $first: {
                        $filter: {
                          input: '$bodies',
                          as: 'body',
                          cond: { $eq: ['$$body.type', 'star'] },
                        },
                      },
                    },
                    { $first: '$bodies' },
                  ],
                },
              },
              in: {
                $and: [
                  {
                    $gte: [{ $toDecimal: '$$primaryBody.position.x' }, left],
                  },
                  {
                    $lte: [{ $toDecimal: '$$primaryBody.position.x' }, right],
                  },
                  {
                    $gte: [{ $toDecimal: '$$primaryBody.position.y' }, top],
                  },
                  {
                    $lte: [{ $toDecimal: '$$primaryBody.position.y' }, bottom],
                  },
                ],
              },
            },
          },
        },
      ],
    };
    const requiredMatch =
      requiredNames.length === 0
        ? []
        : [{ 'bodies.name': { $in: requiredNames } }];
    const blackHoleMatch = { 'bodies.type': 'blackhole' };

    return (await WorldSystemModel.getModel())
      .aggregate<WorldSystemDocument>([
        {
          $match: {
            $or: [
              indexedViewportMatch,
              legacyViewportMatch,
              blackHoleMatch,
              ...requiredMatch,
            ],
          },
        },
        {
          $project: {
            _id: 0,
            name: 1,
            bodies: 1,
            generatedAt: 1,
            updatedAt: 1,
          },
        },
      ])
      .exec();
  }
}
