import type { Server } from 'node:http';
import { WebSocket, WebSocketServer } from 'ws';
import { SpaceshipService } from './spaceship.service';

const SPACESHIP_SOCKET_PATH = '/spaceship/socket';
const SPACESHIP_INFO_INTERVAL_MS = 5_000;

export class SocketService {
  static attachSpaceshipSocketServer(server: Server) {
    const socketServer = new WebSocketServer({ noServer: true });

    server.on('upgrade', (request, socket, head) => {
      const url = new URL(request.url ?? '/', 'http://localhost');
      if (url.pathname !== SPACESHIP_SOCKET_PATH) return;

      const securityCode = SocketService.getSocketSecurityCode(url);
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
      void SocketService.sendSpaceshipInfo(socket, securityCode);
      const interval = setInterval(() => {
        void SocketService.sendSpaceshipInfo(socket, securityCode);
      }, SPACESHIP_INFO_INTERVAL_MS);

      socket.on('message', (data) => {
        void SocketService.handleSpaceshipMessage(
          socket,
          securityCode,
          data.toString(),
        );
      });

      socket.on('close', () => {
        clearInterval(interval);
      });
    });
  }

  private static sendJson(socket: WebSocket, message: unknown) {
    if (socket.readyState !== WebSocket.OPEN) return;
    socket.send(JSON.stringify(message));
  }

  private static getSocketSecurityCode(url: URL) {
    return SpaceshipService.getSecurityCode({
      'x-spaceship-security-code':
        url.searchParams.get('shipSecret') ??
        url.searchParams.get('securityCode') ??
        url.searchParams.get('secret'),
    });
  }

  private static async sendSpaceshipInfo(
    socket: WebSocket,
    securityCode: string,
  ) {
    try {
      const spaceship = await SpaceshipService.loadSpaceship(securityCode);
      if (!spaceship) {
        SocketService.sendJson(socket, {
          type: 'error',
          error: 'Spaceship not found',
        });
        socket.close(1008, 'Spaceship not found');
        return;
      }

      SocketService.sendJson(socket, {
        type: 'spaceship:info',
        spaceship: SpaceshipService.toSpaceshipDto(spaceship),
      });
    } catch (error) {
      console.error('Failed to send spaceship info', error);
      SocketService.sendJson(socket, {
        type: 'error',
        error: 'Failed to load spaceship',
      });
    }
  }

  private static async handleSpaceshipMessage(
    socket: WebSocket,
    securityCode: string,
    rawMessage: string,
  ) {
    let message: unknown;
    try {
      message = JSON.parse(rawMessage);
    } catch {
      SocketService.sendJson(socket, {
        type: 'error',
        error: 'Invalid JSON message',
      });
      return;
    }

    if (
      !message ||
      typeof message !== 'object' ||
      (message as { type?: unknown }).type !== 'spaceship:update'
    ) {
      SocketService.sendJson(socket, {
        type: 'error',
        error: 'Unsupported message type',
      });
      return;
    }

    try {
      const body = (message as { spaceship?: unknown }).spaceship ?? message;
      const update = SpaceshipService.parseSpaceshipUpdate(body);
      const spaceship = await SpaceshipService.updateSpaceship(
        securityCode,
        update,
      );
      if (!spaceship) {
        SocketService.sendJson(socket, {
          type: 'error',
          error: 'Spaceship not found',
        });
        socket.close(1008, 'Spaceship not found');
        return;
      }

      SocketService.sendJson(socket, {
        type: 'spaceship:info',
        spaceship: SpaceshipService.toSpaceshipDto(spaceship),
      });
    } catch (error) {
      SocketService.sendJson(socket, {
        type: 'error',
        error:
          error instanceof Error ? error.message : 'Invalid spaceship update',
      });
    }
  }
}
