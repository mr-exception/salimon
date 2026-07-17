import { WebSocket } from 'ws';
import { SpaceshipService } from '../spaceship.service';
import { sendJson } from './send-json';

export async function sendSpaceshipInfo(
  socket: WebSocket,
  securityCode: string,
) {
  try {
    const spaceship = await SpaceshipService.loadSpaceship(securityCode);
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
    console.error('Failed to send spaceship info', error);
    sendJson(socket, {
      type: 'error',
      error: 'Failed to load spaceship',
    });
  }
}
