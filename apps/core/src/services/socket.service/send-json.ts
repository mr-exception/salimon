import { WebSocket } from 'ws';

export function sendJson(socket: WebSocket, message: unknown) {
  if (socket.readyState !== WebSocket.OPEN) return;
  socket.send(JSON.stringify(message));
}
