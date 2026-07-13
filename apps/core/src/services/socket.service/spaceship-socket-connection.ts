import { WebSocket } from 'ws';
import type { ContactMessageDocument } from '@models';
import { ContactsService } from '../contacts.service';
import { SpaceshipService } from '../spaceship.service';
import type { WorldViewportRequest } from '../world-viewport.service';
import {
  SPACESHIP_INFO_INTERVAL_MS,
  SPACESHIP_PERSIST_INTERVAL_MS,
} from './constants';
import { sendJson } from './send-json';
import { SpaceshipSession } from './spaceship-session';

type SpaceshipSocketIncomingMessage =
  | { type: 'spaceship:update'; spaceship?: unknown }
  | { type: 'spaceship:movement'; spaceship?: unknown }
  | { type: 'spaceship:inventory'; inventory?: unknown }
  | {
      type: 'spaceship:start-target-speed';
      targetSpeedMetersPerSecond?: unknown;
      maximumThrustPercent?: unknown;
      targetDirection?: unknown;
    }
  | {
      type: 'spaceship:start-manual-force';
      thrusters?: unknown;
    }
  | {
      type: 'spaceship:start-thrusters';
      thrusters?: unknown;
    }
  | { type: 'spaceship:stop-active-feature' }
  | {
      type: 'contact:message:send';
      requestId?: unknown;
      contactId?: unknown;
      text?: unknown;
      clientMessageId?: unknown;
    }
  | {
      type: 'world:viewport';
      requestId?: unknown;
      x?: unknown;
      y?: unknown;
      radius?: unknown;
      left?: unknown;
      right?: unknown;
      top?: unknown;
      bottom?: unknown;
    };

export class SpaceshipSocketConnection {
  private session?: SpaceshipSession;
  private readonly infoInterval: NodeJS.Timeout;
  private readonly persistInterval: NodeJS.Timeout;
  private isPersisting = false;
  private hasPendingPersist = false;

  constructor(
    private readonly socket: WebSocket,
    private readonly securityCode: string,
  ) {
    this.infoInterval = setInterval(() => {
      void this.sendWorldInfo();
    }, SPACESHIP_INFO_INTERVAL_MS);
    this.persistInterval = setInterval(() => {
      void this.persistSpaceship();
    }, SPACESHIP_PERSIST_INTERVAL_MS);

    socket.on('message', (data) => {
      void this.handleMessage(data.toString());
    });
    socket.on('close', () => {
      this.close();
    });

    void this.loadSpaceship();
  }

  private async loadSpaceship() {
    try {
      this.session = await SpaceshipSession.create(this.securityCode);
      if (!this.session) {
        sendJson(this.socket, {
          type: 'error',
          error: 'Spaceship not found',
        });
        this.socket.close(1008, 'Spaceship not found');
        return;
      }

      void this.sendWorldInfo();
    } catch (error) {
      console.error('Failed to load spaceship socket connection', error);
      sendJson(this.socket, {
        type: 'error',
        error: 'Failed to load spaceship',
      });
      this.socket.close(1011, 'Failed to load spaceship');
    }
  }

  private async sendWorldInfo(
    options: {
      requestId?: string;
      viewport?: WorldViewportRequest;
    } = {},
  ) {
    if (!this.session) return;

    try {
      await this.session.refreshSpaceship();
      const world = options.viewport
        ? await this.session.getViewportWorldSystems(options.viewport)
        : await this.session.getCurrentViewportWorldSystems();
      sendJson(this.socket, {
        type: 'world:info',
        requestId: options.requestId,
        spaceship: this.session.getSpaceshipDto(),
        ...world,
      });
    } catch (error) {
      console.error('Failed to send spaceship socket info', error);
      sendJson(this.socket, {
        type: 'error',
        error: 'Failed to update spaceship',
      });
    }
  }

  private async handleMessage(rawMessage: string) {
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
      message.type !== 'spaceship:movement' &&
      message.type !== 'spaceship:inventory' &&
      message.type !== 'spaceship:start-target-speed' &&
      message.type !== 'spaceship:start-manual-force' &&
      message.type !== 'spaceship:start-thrusters' &&
      message.type !== 'spaceship:stop-active-feature' &&
      message.type !== 'contact:message:send' &&
      message.type !== 'world:viewport'
    ) {
      sendJson(this.socket, {
        type: 'error',
        error: 'Unsupported message type',
      });
      return;
    }

    try {
      if (!this.session) {
        sendJson(this.socket, {
          type: 'error',
          error: 'Spaceship is not loaded yet',
        });
        return;
      }

      if (message.type === 'spaceship:start-target-speed') {
        const targetSpeedMetersPerSecond = message.targetSpeedMetersPerSecond;
        const maximumThrustPercent = message.maximumThrustPercent;
        const targetDirection = message.targetDirection;
        if (
          typeof targetSpeedMetersPerSecond !== 'number' ||
          typeof maximumThrustPercent !== 'number' ||
          (targetDirection !== undefined && typeof targetDirection !== 'number')
        ) {
          throw new Error('Invalid target speed feature parameters');
        }

        const spaceship = await this.session.startTargetSpeedFeature({
          targetSpeedMetersPerSecond,
          maximumThrustPercent,
          targetDirection,
        });
        if (!spaceship) {
          throw new Error('Target speed feature could not be started');
        }

        this.hasPendingPersist = false;
        void this.sendWorldInfo();
        return;
      }

      if (
        message.type === 'spaceship:start-thrusters' ||
        message.type === 'spaceship:start-manual-force'
      ) {
        const thrusters = message.thrusters;
        if (
          !Array.isArray(thrusters) ||
          thrusters.some(
            (thruster) =>
              typeof thruster?.powerPercent !== 'number' ||
              typeof thruster?.durationSeconds !== 'number',
          )
        ) {
          throw new Error('Invalid thrusters feature parameters');
        }

        const spaceship = await this.session.startThrustersFeature({
          thrusters,
        });
        if (!spaceship) {
          throw new Error('Thrusters feature could not be started');
        }

        this.hasPendingPersist = false;
        void this.sendWorldInfo();
        return;
      }

      if (message.type === 'spaceship:stop-active-feature') {
        await this.session.stopActiveFeature();
        this.hasPendingPersist = false;
        void this.sendWorldInfo();
        return;
      }

      if (message.type === 'world:viewport') {
        const { x, y, radius, requestId, left, right, top, bottom } = message;
        if (
          typeof x !== 'string' ||
          typeof y !== 'string' ||
          typeof radius !== 'string' ||
          (left !== undefined && typeof left !== 'string') ||
          (right !== undefined && typeof right !== 'string') ||
          (top !== undefined && typeof top !== 'string') ||
          (bottom !== undefined && typeof bottom !== 'string') ||
          (requestId !== undefined && typeof requestId !== 'string')
        ) {
          throw new Error('Invalid world viewport parameters');
        }

        void this.sendWorldInfo({
          requestId,
          viewport: {
            x,
            y,
            radius,
            ...(left === undefined ? {} : { left }),
            ...(right === undefined ? {} : { right }),
            ...(top === undefined ? {} : { top }),
            ...(bottom === undefined ? {} : { bottom }),
          },
        });
        return;
      }

      if (message.type === 'contact:message:send') {
        await this.sendContactMessage(message);
        return;
      }

      if (message.type === 'spaceship:inventory') {
        const spaceship = await this.session.updateInventoryFromClient(
          message.inventory,
        );
        if (!spaceship) {
          throw new Error('Spaceship not found');
        }

        this.hasPendingPersist = false;
        sendJson(this.socket, {
          type: 'spaceship:info',
          spaceship: this.session.getSpaceshipDto(),
        });
        return;
      }

      const body = message.spaceship ?? message;
      this.session.updateSpaceshipFromClient(body);
      this.hasPendingPersist = true;
    } catch (error) {
      sendJson(this.socket, {
        type: 'error',
        error:
          error instanceof Error ? error.message : 'Invalid spaceship update',
      });
    }
  }

  private async sendContactMessage(message: {
    requestId?: unknown;
    contactId?: unknown;
    text?: unknown;
    clientMessageId?: unknown;
  }) {
    const requestId =
      typeof message.requestId === 'string' ? message.requestId : undefined;

    try {
      const messageRequest = ContactsService.parseSendMessageRequest({
        contactId: message.contactId,
        text: message.text,
        clientMessageId: message.clientMessageId,
      });
      const contactMessage = await ContactsService.sendMessage(
        this.securityCode,
        messageRequest,
        {
          onReply: (reply) => this.sendContactMessageInfo(reply),
        },
      );
      this.sendContactMessageInfo(contactMessage, requestId);
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : 'Failed to send message';
      sendJson(this.socket, {
        type: 'contact:message:error',
        requestId,
        error: errorMessage,
      });
    }
  }

  private sendContactMessageInfo(
    message: ContactMessageDocument,
    requestId?: string,
  ) {
    sendJson(this.socket, {
      type: 'contact:message',
      requestId,
      message: ContactsService.toMessageDto(message),
    });
  }

  private async persistSpaceship() {
    if (!this.session || !this.hasPendingPersist || this.isPersisting) {
      return;
    }

    this.isPersisting = true;
    try {
      const update = SpaceshipService.parseSpaceshipUpdate(
        this.session.getSpaceshipDto(),
      );
      const spaceship = await SpaceshipService.updateSpaceship(
        this.securityCode,
        update,
      );
      if (spaceship) {
        this.session = await SpaceshipSession.create(this.securityCode);
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
