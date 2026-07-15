import 'dotenv/config';
import { SocketService, TickingService } from '@services';
import { createApp } from './app';

const DEFAULT_PORT = 3000;
const port = Number(process.env.PORT ?? DEFAULT_PORT);

if (!Number.isInteger(port) || port <= 0) {
  throw new Error('PORT must be a positive integer');
}

void TickingService.start().catch((error: unknown) => {
  console.error('Failed to start world ticking', error);
});

const app = createApp();
const server = app.listen(port, () => {
  console.log(`Core API listening on http://localhost:${port}`);
});

SocketService.attachSpaceshipSocketServer(server);
