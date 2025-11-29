import { MongoClient, Db, Collection, Document } from 'mongodb';

const MONGODB_URI = process.env.MONGODB_URI!;
const DB_NAME = process.env.MONGODB_DB_NAME!;
const COLLECTION_NAME = process.env.MONGODB_COLLECTION_NAME!;

let db: Db;
let client: MongoClient;

export const initializeMongoDB = async (): Promise<void> => {
  client = new MongoClient(MONGODB_URI);
  await client.connect();
  db = client.db(DB_NAME);
};

export const getDatabase = (): Db => {
  if (!db) {
    throw new Error('Database not initialized. Call initializeMongoDB first.');
  }
  return db;
};

export const getCollection = <T extends Document = Document>(collectionName?: string): Collection<T> => {
  const name = collectionName || COLLECTION_NAME;
  return getDatabase().collection<T>(name);
};

export const closeConnection = async (): Promise<void> => {
  if (client) {
    await client.close();
  }
};
