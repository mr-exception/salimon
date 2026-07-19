import { mongoose } from '@typegoose/typegoose';
import type { SerializedWorldBody } from '@repo/types';
import { DatabaseModel } from './database.model';

export type WorldSystemDocument = {
  name: string;
  bodies: SerializedWorldBody[];
  generatedAt?: Date;
  updatedAt?: Date;
};

const WorldSystemSchema = new mongoose.Schema(
  {
    name: { required: true, type: String, unique: true },
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
}
