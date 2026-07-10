import type { Server } from 'node:http';
import { WebSocketServer } from 'ws';
import { SPACESHIP_SOCKET_PATH } from './constants';
import { getSocketSecurityCode } from './get-socket-security-code';
import { SpaceshipSocketConnection } from './spaceship-socket-connection';

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

  socketServer.on('connection', (socket, securityCode: string) => {
    new SpaceshipSocketConnection(socket, securityCode);
  });
}
