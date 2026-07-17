import { WebSocket } from 'ws';
import { SpaceshipService } from '../spaceship.service';
import { sendJson } from './send-json';

export async function handleSpaceshipMessage(
  socket: WebSocket,
  securityCode: string,
  rawMessage: string,
) {
  let message: unknown;
  try {
    message = JSON.parse(rawMessage);
  } catch {
    sendJson(socket, {
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
    sendJson(socket, {
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
      sendJson(socket, {
        type: 'error',
        error: 'Spaceship not found',
      });
      socket.close(1008, 'Spaceship not found');
      return;
    }

    sendJson(socket, {
      type: 'ship:info',
      spaceship: SpaceshipService.toSpaceshipDto(spaceship),
    });
  } catch (error) {
    sendJson(socket, {
      type: 'error',
      error:
        error instanceof Error ? error.message : 'Invalid spaceship update',
    });
  }
}
