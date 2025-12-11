import { MongoClient, Db, Collection, Document, MongoError } from 'mongodb';

const MONGODB_URI = process.env.MONGODB_URI!;
const DB_NAME = process.env.MONGODB_DB_NAME || process.env.MONGODB_DATABASE!;
const COLLECTION_NAME = process.env.MONGODB_COLLECTION_NAME!;

let db: Db;
let client: MongoClient;
let isConnected = false;
let connectionRetries = 0;
const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 2000;

export class DatabaseError extends Error {
  constructor(
    message: string,
    public readonly originalError?: Error,
    public readonly isConnectionError: boolean = false
  ) {
    super(message);
    this.name = 'DatabaseError';
  }
}

const sleep = (ms: number): Promise<void> => {
  return new Promise(resolve => setTimeout(resolve, ms));
};

export const initializeMongoDB = async (): Promise<void> => {
  if (!MONGODB_URI) {
    throw new DatabaseError('MONGODB_URI environment variable is not set');
  }

  if (!DB_NAME) {
    throw new DatabaseError('MONGODB_DB_NAME or MONGODB_DATABASE environment variable is not set');
  }

  while (connectionRetries < MAX_RETRIES) {
    try {
      client = new MongoClient(MONGODB_URI, {
        connectTimeoutMS: 10000,
        serverSelectionTimeoutMS: 10000,
        socketTimeoutMS: 45000,
      });

      await client.connect();
      db = client.db(DB_NAME);
      isConnected = true;

      client.on('error', (error) => {
        console.error('MongoDB connection error:', error);
        isConnected = false;
      });

      client.on('close', () => {
        console.warn('MongoDB connection closed');
        isConnected = false;
      });

      console.log('Successfully connected to MongoDB');
      connectionRetries = 0;
      return;
    } catch (error) {
      connectionRetries++;
      console.error(`MongoDB connection attempt ${connectionRetries} failed:`, error);

      if (connectionRetries >= MAX_RETRIES) {
        throw new DatabaseError(
          `Failed to connect to MongoDB after ${MAX_RETRIES} attempts`,
          error instanceof Error ? error : undefined,
          true
        );
      }

      await sleep(RETRY_DELAY_MS * connectionRetries);
    }
  }
};

export const getDatabase = (): Db => {
  if (!db) {
    throw new DatabaseError('Database not initialized. Call initializeMongoDB first.', undefined, true);
  }

  if (!isConnected) {
    throw new DatabaseError('Database connection is not active', undefined, true);
  }

  return db;
};

export const getCollection = <T extends Document = Document>(collectionName?: string): Collection<T> => {
  if (!collectionName && !COLLECTION_NAME) {
    throw new DatabaseError('Collection name not provided and MONGODB_COLLECTION_NAME is not set');
  }

  const name = collectionName || COLLECTION_NAME;
  return getDatabase().collection<T>(name);
};

export const isConnectionHealthy = (): boolean => {
  return isConnected && !!db;
};

export const closeConnection = async (): Promise<void> => {
  if (client) {
    try {
      await client.close();
      isConnected = false;
      console.log('MongoDB connection closed successfully');
    } catch (error) {
      console.error('Error closing MongoDB connection:', error);
      throw new DatabaseError(
        'Failed to close database connection',
        error instanceof Error ? error : undefined
      );
    }
  }
};

export const handleDatabaseError = (error: unknown): DatabaseError => {
  if (error instanceof DatabaseError) {
    return error;
  }

  if (error instanceof MongoError) {
    const isConnectionIssue = [
      'ECONNREFUSED',
      'ETIMEDOUT',
      'ENOTFOUND',
      'NetworkError',
      'MongoNetworkError'
    ].some(code => error.message.includes(code) || error.name.includes(code));

    return new DatabaseError(
      `Database operation failed: ${error.message}`,
      error,
      isConnectionIssue
    );
  }

  if (error instanceof Error) {
    return new DatabaseError(
      `Unexpected database error: ${error.message}`,
      error
    );
  }

  return new DatabaseError('An unknown database error occurred');
};
