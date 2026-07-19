import path from 'node:path';
import dotenv from 'dotenv';
import { createApp } from './app';

dotenv.config({ path: path.resolve(__dirname, '../.env') });

const DEFAULT_PORT = 3000;
const port = Number(process.env.PORT ?? DEFAULT_PORT);

if (!Number.isInteger(port) || port <= 0) {
  throw new Error('PORT must be a positive integer');
}

async function start() {
  const app = createApp();
  app.listen(port, () => {
    console.log(`Core API listening on http://localhost:${port}`);
  });
}

start().catch((error: unknown) => {
  console.error('Failed to start Core API', error);
  process.exit(1);
});
