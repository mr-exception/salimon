import { getModelForClass, modelOptions, prop } from '@typegoose/typegoose';
import type {
  SerializedPosition,
  SpaceshipActiveFeature,
  SpaceshipInventory,
  SpaceshipMotionState,
  SpaceshipStats,
  Velocity,
} from '@repo/types';
import { DatabaseModel } from './database.model';

export type SpaceshipVelocity = Velocity;
export type {
  SpaceshipActiveFeature,
  SpaceshipInventory,
  SpaceshipMotionState,
  SpaceshipStats,
};

class SpaceshipPosition implements SerializedPosition {
  @prop({ required: true, type: () => String })
  public x!: string;

  @prop({ required: true, type: () => String })
  public y!: string;

  @prop({ type: () => String })
  public relativeTo?: string;
}
class SpaceshipVelocitySchema implements SpaceshipVelocity {
  @prop({ required: true, type: () => Number })
  public x!: number;

  @prop({ required: true, type: () => Number })
  public y!: number;
}

class SpaceshipStatsSchema implements SpaceshipStats {
  @prop({ required: true, type: () => Number })
  public fuelKns!: number;

  @prop({ required: true, type: () => Number })
  public hullDurability!: number;

  @prop({ required: true, type: () => [Number] })
  public thrusterDurability!: number[];
}

class SpaceshipInventorySchema implements SpaceshipInventory {
  @prop({ required: true, type: () => Number })
  public iron!: number;

  @prop({ required: true, type: () => Number })
  public silicates!: number;

  @prop({ required: true, type: () => Number })
  public ice!: number;

  @prop({ required: true, type: () => Number })
  public silver!: number;

  @prop({ required: true, type: () => Number })
  public carbon!: number;

  @prop({ required: true, type: () => Number })
  public gold!: number;

  @prop({ required: true, type: () => Number })
  public hydrogen!: number;

  @prop({ required: true, type: () => Number })
  public nitrogen!: number;
}

class SpaceshipTargetSpeedFeatureSchema implements SpaceshipActiveFeature {
  @prop({ required: true, type: () => String })
  public type!: 'target-speed';

  @prop({ required: true, type: () => Number })
  public targetSpeedMetersPerSecond!: number;

  @prop({ required: true, type: () => Number })
  public maximumThrustPercent!: number;

  @prop({ type: () => Number })
  public targetDirection?: number;

  @prop({ required: true, type: () => SpaceshipVelocitySchema })
  public targetVelocity!: SpaceshipVelocity;

  @prop({ required: true, type: () => Number })
  public maximumAcceleration!: number;

  @prop({ required: true, type: () => Number })
  public durationSeconds!: number;

  @prop({ required: true, type: () => Number })
  public elapsedSeconds!: number;
}

@modelOptions({
  schemaOptions: { collection: 'spaceships', versionKey: false },
})
class SpaceshipSchema {
  @prop({ required: true, type: () => String })
  public securityCode!: string;

  @prop({ required: true, type: () => SpaceshipPosition })
  public position!: SpaceshipPosition;

  @prop({ required: true, type: () => Number })
  public direction!: number;

  @prop({ required: true, type: () => String })
  public speed!: string;

  @prop({ type: () => SpaceshipVelocitySchema })
  public velocity?: SpaceshipVelocity;

  @prop({ enum: ['flying', 'landed', 'crashed'], type: () => String })
  public motionState?: SpaceshipMotionState;

  @prop({ type: () => SpaceshipStatsSchema })
  public stats?: SpaceshipStats;

  @prop({ type: () => SpaceshipInventorySchema })
  public inventory?: SpaceshipInventory;

  @prop({ type: () => SpaceshipTargetSpeedFeatureSchema })
  public activeFeature?: SpaceshipActiveFeature;

  @prop({ type: () => Date })
  public simulatedAt?: Date;

  @prop({ required: true, type: () => Date })
  public createdAt!: Date;

  @prop({ required: true, type: () => Date })
  public updatedAt!: Date;
}

export type SpaceshipDocument = SpaceshipSchema;

const SpaceshipTypegooseModel = getModelForClass(SpaceshipSchema);

export class SpaceshipModel {
  static async getModel() {
    await DatabaseModel.connect();
    return SpaceshipTypegooseModel;
  }

  static async insert(spaceship: SpaceshipDocument) {
    return (await SpaceshipModel.getModel()).create(spaceship);
  }

  static async findAll() {
    return (await SpaceshipModel.getModel())
      .find({})
      .lean<SpaceshipDocument[]>()
      .exec();
  }

  static async findBySecurityCode(securityCode: string) {
    return (await SpaceshipModel.getModel())
      .findOne({ securityCode })
      .lean<SpaceshipDocument>()
      .exec();
  }

  static async updateBySecurityCode(
    securityCode: string,
    update: Partial<SpaceshipDocument>,
  ) {
    return (await SpaceshipModel.getModel())
      .findOneAndUpdate({ securityCode }, { $set: update }, { new: true })
      .lean<SpaceshipDocument>()
      .exec()
      .then((spaceship) => spaceship ?? undefined);
  }

  static async findOldestForSimulation(
    invocationTime: Date,
    batchSize: number,
  ) {
    return (await SpaceshipModel.getModel())
      .find({
        $or: [
          { simulatedAt: { $type: 'date', $lt: invocationTime } },
          {
            simulatedAt: { $exists: false },
            updatedAt: { $type: 'date', $lt: invocationTime },
          },
        ],
      })
      .sort({ simulatedAt: 1, updatedAt: 1 })
      .limit(batchSize)
      .lean<SpaceshipDocument[]>()
      .exec();
  }

  static async updatePropagatedSpaceship(
    spaceship: SpaceshipDocument,
    update: Partial<SpaceshipDocument>,
  ) {
    const model = await SpaceshipModel.getModel();
    const result = await model
      .findOneAndUpdate(
        {
          securityCode: spaceship.securityCode,
          updatedAt: spaceship.updatedAt,
        },
        { $set: update },
        { new: true },
      )
      .lean<SpaceshipDocument>()
      .exec();
    return (
      result ??
      (await model
        .findOne({ securityCode: spaceship.securityCode })
        .lean<SpaceshipDocument>()
        .exec()) ??
      spaceship
    );
  }

  static async replaceSpaceships(spaceships: SpaceshipDocument[]) {
    if (spaceships.length === 0) {
      return { modifiedCount: 0, upsertedCount: 0 };
    }

    return (await SpaceshipModel.getModel()).bulkWrite(
      spaceships.map((spaceship) => ({
        replaceOne: {
          filter: { securityCode: spaceship.securityCode },
          replacement: spaceship,
          upsert: true,
        },
      })),
      { ordered: false },
    );
  }
}
