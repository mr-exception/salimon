import { MongoClient } from 'mongodb';

let clientPromise: Promise<MongoClient> | undefined;

export class DatabaseModel {
  static async getDatabase() {
    const uri = process.env.MONGODB_URI;
    if (!uri) throw new Error('MONGODB_URI is not configured');

    clientPromise ??= new MongoClient(uri).connect().catch((error: unknown) => {
      clientPromise = undefined;
      throw error;
    });

    return (await clientPromise).db();
  }
}
