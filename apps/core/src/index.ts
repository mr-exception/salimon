import 'dotenv/config';
import { createServer } from 'node:http';
import { createApp } from './app';
import { attachSpaceshipSocketServer } from './socket';

const DEFAULT_PORT = 3000;
const port = Number(process.env.PORT ?? DEFAULT_PORT);

if (!Number.isInteger(port) || port <= 0) {
  throw new Error('PORT must be a positive integer');
}

const server = createServer(createApp());
attachSpaceshipSocketServer(server);

server.listen(port, () => {
  console.log(`Core API listening on http://localhost:${port}`);
});
