import { Server } from 'socket.io';
import { ContactMessageModel } from '@models';
import {
  ContactsService,
  RepositoryService,
  SpaceshipService,
} from '@services';
import type { ContactMessageDto } from '@repo/types';
import type { ContactShipContext } from './contacts';
import type { IncomingHttpHeaders } from 'node:http';
import type { Server as HttpServer } from 'node:http';

type SendContactMessageRequest = {
  contactId: string;
  text: string;
  clientMessageId: string;
  shipContext?: ContactShipContext;
};

type SendContactMessageAck =
  | { ok: true; message: ContactMessageDto }
  | { ok: false; error: string };

type MarkThreadReadAck =
  | { ok: true; contactId: string }
  | { ok: false; error: string };

const CLIENT_ORIGIN = process.env.CLIENT_ORIGIN ?? 'http://localhost:5173';
const SPACESHIP_ROOM_PREFIX = 'spaceship:';

export function configureSocketServer(server: HttpServer) {
  const io = new Server(server, {
    cors: {
      origin: CLIENT_ORIGIN,
    },
  });

  io.on('connection', (socket) => {
    const securityCode = getSocketSecurityCode(
      socket.handshake.auth,
      socket.handshake.headers,
    );

    if (!securityCode) {
      socket.emit('communications:error', {
        error: 'A valid spaceship security code is required',
      });
      socket.disconnect(true);
      return;
    }

    const spaceshipRoom = getSpaceshipRoom(securityCode);
    socket.join(spaceshipRoom);

    void sendUnreadMessages(securityCode, socket).catch((error: unknown) => {
      console.error('Failed to send unread messages over socket', error);
      socket.emit('communications:error', {
        error: 'Failed to load unread messages',
      });
    });

    socket.on(
      'contact:send-message',
      (
        request: SendContactMessageRequest,
        acknowledge?: (response: SendContactMessageAck) => void,
      ) => {
        void handleSendContactMessage(io, securityCode, request, acknowledge);
      },
    );

    socket.on(
      'contact:mark-thread-read',
      (
        request: { contactId: string },
        acknowledge?: (response: MarkThreadReadAck) => void,
      ) => {
        void handleMarkThreadRead(securityCode, request, acknowledge);
      },
    );
  });
}

async function sendUnreadMessages(
  securityCode: string,
  socket: Parameters<Parameters<Server['on']>[1]>[0],
) {
  const spaceship =
    await RepositoryService.findSpaceshipBySecurityCode(securityCode);
  if (!spaceship) {
    socket.emit('communications:error', { error: 'Spaceship not found' });
    socket.disconnect(true);
    return;
  }

  await ContactsService.initializeSpaceshipContacts(securityCode);
  const messages =
    await ContactMessageModel.findUnreadContactMessages(securityCode);
  socket.emit('contact:unread-messages', {
    messages: messages.map(ContactsService.toMessageDto),
  });
}

async function handleSendContactMessage(
  io: Server,
  securityCode: string,
  request: SendContactMessageRequest,
  acknowledge?: (response: SendContactMessageAck) => void,
) {
  try {
    const messageRequest = ContactsService.parseSendMessageRequest(request);
    const message = await ContactsService.sendMessage(
      securityCode,
      messageRequest,
      {
        onReply: (reply) => {
          io.to(getSpaceshipRoom(securityCode)).emit(
            'contact:message',
            ContactsService.toMessageDto(reply),
          );
        },
      },
    );
    const messageDto = ContactsService.toMessageDto(message);
    io.to(getSpaceshipRoom(securityCode)).emit('contact:message', messageDto);
    acknowledge?.({ ok: true, message: messageDto });
  } catch (error) {
    console.error('Failed to send contact message over socket', error);
    acknowledge?.({ ok: false, error: 'Failed to send contact message' });
  }
}

async function handleMarkThreadRead(
  securityCode: string,
  request: { contactId: string },
  acknowledge?: (response: MarkThreadReadAck) => void,
) {
  try {
    const contactId = request.contactId.trim();
    if (
      !contactId ||
      !(await ContactsService.hasContact(securityCode, contactId))
    ) {
      acknowledge?.({ ok: false, error: 'Contact not found' });
      return;
    }

    await ContactMessageModel.markContactMessagesRead(securityCode, contactId);
    acknowledge?.({ ok: true, contactId });
  } catch (error) {
    console.error('Failed to mark contact thread read over socket', error);
    acknowledge?.({ ok: false, error: 'Failed to mark contact thread read' });
  }
}

function getSocketSecurityCode(
  auth: Record<string, unknown>,
  headers: IncomingHttpHeaders,
) {
  const authSecurityCode = auth.securityCode;
  if (typeof authSecurityCode === 'string') return authSecurityCode.trim();
  return SpaceshipService.getSecurityCode(headers)?.trim();
}

function getSpaceshipRoom(securityCode: string) {
  return `${SPACESHIP_ROOM_PREFIX}${securityCode}`;
}
