import { mongoose } from '@typegoose/typegoose';

let connectionPromise: Promise<typeof mongoose> | undefined;
let isQueryTimingLoggerAttached = false;

type MongoCommandStartedEvent = {
  commandName: string;
  requestId: number;
  databaseName: string;
  command: Record<string, unknown>;
};

type MongoCommandFinishedEvent = {
  commandName: string;
  requestId: number;
  duration: number;
  failure?: unknown;
};

type MongoCommandInfo = {
  collectionName?: string;
  databaseName: string;
};

const loggedCommandNames = new Set([
  'aggregate',
  'bulkWrite',
  'count',
  'delete',
  'distinct',
  'find',
  'findAndModify',
  'getMore',
  'insert',
  'update',
]);

const activeCommands = new Map<number, MongoCommandInfo>();

function getCommandCollectionName(
  event: MongoCommandStartedEvent,
): string | undefined {
  const cursorCollectionName = event.command.collection;
  if (typeof cursorCollectionName === 'string') return cursorCollectionName;

  const collectionName = event.command[event.commandName];
  return typeof collectionName === 'string' ? collectionName : undefined;
}

function attachQueryTimingLogger() {
  if (isQueryTimingLoggerAttached) return;

  const client = mongoose.connection.getClient();

  client.on('commandStarted', (event: MongoCommandStartedEvent) => {
    if (!loggedCommandNames.has(event.commandName)) return;

    activeCommands.set(event.requestId, {
      collectionName: getCommandCollectionName(event),
      databaseName: event.databaseName,
    });
  });

  client.on('commandSucceeded', (event: MongoCommandFinishedEvent) => {
    logFinishedCommand(event, 'completed');
  });

  client.on('commandFailed', (event: MongoCommandFinishedEvent) => {
    logFinishedCommand(event, 'failed');
  });

  isQueryTimingLoggerAttached = true;
}

function logFinishedCommand(
  event: MongoCommandFinishedEvent,
  status: 'completed' | 'failed',
) {
  const commandInfo = activeCommands.get(event.requestId);
  if (!commandInfo) return;

  activeCommands.delete(event.requestId);

  const namespace = commandInfo.collectionName
    ? `${commandInfo.databaseName}.${commandInfo.collectionName}`
    : commandInfo.databaseName;
  const message = `MongoDB ${event.commandName} ${namespace} ${status} in ${event.duration}ms`;

  if (status === 'failed') {
    console.warn(message);
    return;
  }

  // console.info(message);
}

export class DatabaseModel {
  static async connect() {
    const uri = process.env.MONGODB_URI;
    if (!uri) throw new Error('MONGODB_URI is not configured');

    connectionPromise ??= mongoose
      .connect(uri, { monitorCommands: true, serverSelectionTimeoutMS: 5_000 })
      .then((connection) => {
        attachQueryTimingLogger();
        return connection;
      })
      .catch((error: unknown) => {
        connectionPromise = undefined;
        throw error;
      });

    return connectionPromise;
  }
}
