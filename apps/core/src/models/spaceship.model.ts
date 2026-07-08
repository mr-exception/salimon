import { DatabaseModel } from './database.model';

type SpaceshipPosition = {
  x: string;
  y: string;
  relativeTo?: string;
};

export type SpaceshipVelocity = {
  x: number;
  y: number;
};

export type SpaceshipMotionState = 'flying' | 'landed' | 'crashed';

export type SpaceshipStats = {
  fuelKns: number;
  hullDurability: number;
  thrusterDurability: number[];
};

export type SpaceshipDocument = {
  securityCode: string;
  position: SpaceshipPosition;
  direction: number;
  speed: string;
  velocity?: SpaceshipVelocity;
  motionState?: SpaceshipMotionState;
  stats?: SpaceshipStats;
  simulatedAt?: Date;
  createdAt: Date;
  updatedAt: Date;
};

export class SpaceshipModel {
  static async getCollection() {
    return (await DatabaseModel.getDatabase()).collection<SpaceshipDocument>(
      'spaceships',
    );
  }

  static async insert(spaceship: SpaceshipDocument) {
    return (await SpaceshipModel.getCollection()).insertOne(spaceship);
  }

  static async findBySecurityCode(securityCode: string) {
    return (await SpaceshipModel.getCollection()).findOne({ securityCode });
  }

  static async updateBySecurityCode(
    securityCode: string,
    update: Partial<SpaceshipDocument>,
  ) {
    return (await SpaceshipModel.getCollection())
      .findOneAndUpdate(
        { securityCode },
        { $set: update },
        { returnDocument: 'after' },
      )
      .then((spaceship) => spaceship ?? undefined);
  }

  static async findOldestForSimulation(
    invocationTime: Date,
    batchSize: number,
  ) {
    return (await SpaceshipModel.getCollection())
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
      .toArray();
  }

  static async updatePropagatedSpaceship(
    spaceship: SpaceshipDocument,
    update: Partial<SpaceshipDocument>,
  ) {
    const collection = await SpaceshipModel.getCollection();
    const result = await collection.findOneAndUpdate(
      {
        securityCode: spaceship.securityCode,
        updatedAt: spaceship.updatedAt,
      },
      { $set: update },
      { returnDocument: 'after' },
    );
    return (
      result ??
      (await collection.findOne({ securityCode: spaceship.securityCode })) ??
      spaceship
    );
  }
}
