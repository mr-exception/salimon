import { mongoose } from '@typegoose/typegoose';

let connectionPromise: Promise<typeof mongoose> | undefined;

export class DatabaseModel {
  static async connect() {
    const uri = process.env.MONGODB_URI;
    if (!uri) throw new Error('MONGODB_URI is not configured');

    connectionPromise ??= mongoose
      .connect(uri, { serverSelectionTimeoutMS: 5_000 })
      .catch((error: unknown) => {
        connectionPromise = undefined;
        throw error;
      });

    return connectionPromise;
  }
}
