import type { Server } from 'node:http';
import { WebSocket, WebSocketServer } from 'ws';
import { SpaceshipService } from '@services/spaceship.service';

const SPACESHIP_SOCKET_PATH = '/spaceship/socket';
const SPACESHIP_INFO_INTERVAL_MS = 5_000;

function sendJson(socket: WebSocket, message: unknown) {
  if (socket.readyState !== WebSocket.OPEN) return;
  socket.send(JSON.stringify(message));
}

function getSocketSecurityCode(url: URL) {
  return SpaceshipService.getSecurityCode({
    'x-spaceship-security-code':
      url.searchParams.get('shipSecret') ??
      url.searchParams.get('securityCode') ??
      url.searchParams.get('secret'),
  });
}

async function sendSpaceshipInfo(socket: WebSocket, securityCode: string) {
  try {
    const spaceship = await SpaceshipService.loadSpaceship(securityCode);
    if (!spaceship) {
      sendJson(socket, { type: 'error', error: 'Spaceship not found' });
      socket.close(1008, 'Spaceship not found');
      return;
    }

    sendJson(socket, {
      type: 'spaceship:info',
      spaceship: SpaceshipService.toSpaceshipDto(spaceship),
    });
  } catch (error) {
    console.error('Failed to send spaceship info', error);
    sendJson(socket, { type: 'error', error: 'Failed to load spaceship' });
  }
}

export function attachSpaceshipSocketServer(server: Server) {
  const socketServer = new WebSocketServer({ noServer: true });

  server.on('upgrade', (request, socket, head) => {
    const url = new URL(request.url ?? '/', 'http://localhost');
    if (url.pathname !== SPACESHIP_SOCKET_PATH) return;

    const securityCode = getSocketSecurityCode(url);
    if (!securityCode) {
      socket.write('HTTP/1.1 401 Unauthorized\r\n\r\n');
      socket.destroy();
      return;
    }

    socketServer.handleUpgrade(request, socket, head, (webSocket) => {
      socketServer.emit('connection', webSocket, securityCode);
    });
  });

  socketServer.on('connection', (socket: WebSocket, securityCode: string) => {
    void sendSpaceshipInfo(socket, securityCode);
    const interval = setInterval(() => {
      void sendSpaceshipInfo(socket, securityCode);
    }, SPACESHIP_INFO_INTERVAL_MS);

    socket.on('message', (data) => {
      void (async () => {
        let message: unknown;
        try {
          message = JSON.parse(data.toString());
        } catch {
          sendJson(socket, { type: 'error', error: 'Invalid JSON message' });
          return;
        }

        if (
          !message ||
          typeof message !== 'object' ||
          (message as { type?: unknown }).type !== 'spaceship:update'
        ) {
          sendJson(socket, {
            type: 'error',
            error: 'Unsupported message type',
          });
          return;
        }

        try {
          const body =
            (message as { spaceship?: unknown }).spaceship ?? message;
          const update = SpaceshipService.parseSpaceshipUpdate(body);
          const spaceship = await SpaceshipService.updateSpaceship(
            securityCode,
            update,
          );
          if (!spaceship) {
            sendJson(socket, { type: 'error', error: 'Spaceship not found' });
            socket.close(1008, 'Spaceship not found');
            return;
          }

          sendJson(socket, {
            type: 'spaceship:info',
            spaceship: SpaceshipService.toSpaceshipDto(spaceship),
          });
        } catch (error) {
          sendJson(socket, {
            type: 'error',
            error:
              error instanceof Error
                ? error.message
                : 'Invalid spaceship update',
          });
        }
      })();
    });

    socket.on('close', () => {
      clearInterval(interval);
    });
  });
}
