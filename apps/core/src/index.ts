import 'dotenv/config';
import { createServer } from 'node:http';
import { SocketService, TickingService } from '@services';
import { createApp } from './app';

const DEFAULT_PORT = 3000;
const port = Number(process.env.PORT ?? DEFAULT_PORT);

if (!Number.isInteger(port) || port <= 0) {
  throw new Error('PORT must be a positive integer');
}

const server = createServer(createApp());
SocketService.attachSpaceshipSocketServer(server);

TickingService.start();

server.listen(port, () => {
  console.log(`Core API listening on http://localhost:${port}`);
});
