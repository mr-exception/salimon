import { attachSpaceshipSocketServer } from './attach-spaceship-socket-server';
import { getSocketSecurityCode } from './get-socket-security-code';
import { handleSpaceshipMessage } from './handle-spaceship-message';
import { sendJson } from './send-json';
import { sendSpaceshipInfo } from './send-spaceship-info';
import { SpaceshipSocketConnection } from './spaceship-socket-connection';

export class SocketService {
  static attachSpaceshipSocketServer = attachSpaceshipSocketServer;
  static sendJson = sendJson;
  static getSocketSecurityCode = getSocketSecurityCode;
  static sendSpaceshipInfo = sendSpaceshipInfo;
  static handleSpaceshipMessage = handleSpaceshipMessage;
  static SpaceshipSocketConnection = SpaceshipSocketConnection;
}
