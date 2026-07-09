import { getModelForClass, modelOptions, prop } from '@typegoose/typegoose';
import type {
  SerializedPosition,
  SpaceshipMotionState,
  SpaceshipStats,
  Velocity,
} from '@repo/types';
import { DatabaseModel } from './database.model';

export type SpaceshipVelocity = Velocity;
export type { SpaceshipMotionState, SpaceshipStats };

class SpaceshipPosition implements SerializedPosition {
  @prop({ required: true })
  public x!: string;

  @prop({ required: true })
  public y!: string;

  @prop()
  public relativeTo?: string;
}
class SpaceshipVelocitySchema implements SpaceshipVelocity {
  @prop({ required: true })
  public x!: number;

  @prop({ required: true })
  public y!: number;
}

class SpaceshipStatsSchema implements SpaceshipStats {
  @prop({ required: true })
  public fuelKns!: number;

  @prop({ required: true })
  public hullDurability!: number;

  @prop({ required: true, type: () => [Number] })
  public thrusterDurability!: number[];
}

@modelOptions({
  schemaOptions: { collection: 'spaceships', versionKey: false },
})
class SpaceshipSchema {
  @prop({ required: true })
  public securityCode!: string;

  @prop({ required: true, type: () => SpaceshipPosition })
  public position!: SpaceshipPosition;

  @prop({ required: true })
  public direction!: number;

  @prop({ required: true })
  public speed!: string;

  @prop({ type: () => SpaceshipVelocitySchema })
  public velocity?: SpaceshipVelocity;

  @prop({ enum: ['flying', 'landed', 'crashed'], type: () => String })
  public motionState?: SpaceshipMotionState;

  @prop({ type: () => SpaceshipStatsSchema })
  public stats?: SpaceshipStats;

  @prop()
  public simulatedAt?: Date;

  @prop({ required: true })
  public createdAt!: Date;

  @prop({ required: true })
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
}
