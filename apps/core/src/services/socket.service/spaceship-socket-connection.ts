import { WebSocket } from 'ws';
import {
  type SpaceshipDocument,
  SpaceshipService,
} from '../spaceship.service';
import {
  SPACESHIP_INFO_INTERVAL_MS,
  SPACESHIP_PERSIST_INTERVAL_MS,
} from './constants';
import { sendJson } from './send-json';

type SpaceshipSocketIncomingMessage =
  | { type: 'spaceship:update'; spaceship?: unknown }
  | { type: 'spaceship:movement'; spaceship?: unknown };

export class SpaceshipSocketConnection {
  private spaceship?: SpaceshipDocument;
  private readonly infoInterval: NodeJS.Timeout;
  private readonly persistInterval: NodeJS.Timeout;
  private isPersisting = false;
  private hasPendingPersist = false;

  constructor(
    private readonly socket: WebSocket,
    private readonly securityCode: string,
  ) {
    this.infoInterval = setInterval(() => {
      this.sendSpaceshipInfo();
    }, SPACESHIP_INFO_INTERVAL_MS);
    this.persistInterval = setInterval(() => {
      void this.persistSpaceship();
    }, SPACESHIP_PERSIST_INTERVAL_MS);

    socket.on('message', (data) => {
      this.handleMessage(data.toString());
    });
    socket.on('close', () => {
      this.close();
    });

    void this.loadSpaceship();
  }

  private async loadSpaceship() {
    try {
      const spaceship = await SpaceshipService.loadSpaceship(
        this.securityCode,
      );
      if (!spaceship) {
        sendJson(this.socket, {
          type: 'error',
          error: 'Spaceship not found',
        });
        this.socket.close(1008, 'Spaceship not found');
        return;
      }

      this.spaceship = spaceship;
      this.sendSpaceshipInfo();
    } catch (error) {
      console.error('Failed to load spaceship socket connection', error);
      sendJson(this.socket, {
        type: 'error',
        error: 'Failed to load spaceship',
      });
      this.socket.close(1011, 'Failed to load spaceship');
    }
  }

  private sendSpaceshipInfo() {
    if (!this.spaceship) return;

    sendJson(this.socket, {
      type: 'spaceship:info',
      spaceship: SpaceshipService.toSpaceshipDto(this.spaceship),
    });
  }

  private handleMessage(rawMessage: string) {
    let message: SpaceshipSocketIncomingMessage;
    try {
      message = JSON.parse(rawMessage) as SpaceshipSocketIncomingMessage;
    } catch {
      sendJson(this.socket, {
        type: 'error',
        error: 'Invalid JSON message',
      });
      return;
    }

    if (
      message.type !== 'spaceship:update' &&
      message.type !== 'spaceship:movement'
    ) {
      sendJson(this.socket, {
        type: 'error',
        error: 'Unsupported message type',
      });
      return;
    }

    try {
      if (!this.spaceship) {
        sendJson(this.socket, {
          type: 'error',
          error: 'Spaceship is not loaded yet',
        });
        return;
      }

      const body = message.spaceship ?? message;
      const update = SpaceshipService.parseSpaceshipUpdate(body);
      const now = new Date();
      this.spaceship = {
        ...this.spaceship,
        ...update,
        securityCode: this.securityCode,
        simulatedAt: now,
        updatedAt: now,
      };
      this.hasPendingPersist = true;
    } catch (error) {
      sendJson(this.socket, {
        type: 'error',
        error:
          error instanceof Error ? error.message : 'Invalid spaceship update',
      });
    }
  }

  private async persistSpaceship() {
    if (!this.spaceship || !this.hasPendingPersist || this.isPersisting) {
      return;
    }

    this.isPersisting = true;
    try {
      const update = SpaceshipService.parseSpaceshipUpdate(
        SpaceshipService.toSpaceshipDto(this.spaceship),
      );
      const spaceship = await SpaceshipService.updateSpaceship(
        this.securityCode,
        update,
      );
      if (spaceship) {
        this.spaceship = spaceship;
        this.hasPendingPersist = false;
      }
    } catch (error) {
      console.error('Failed to persist spaceship socket connection', error);
    } finally {
      this.isPersisting = false;
    }
  }

  private close() {
    clearInterval(this.infoInterval);
    clearInterval(this.persistInterval);
    void this.persistSpaceship();
  }
}
