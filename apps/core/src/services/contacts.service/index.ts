import {
  CONTACTS,
  EASA_CHIEF_ID,
  INITIAL_CHIEF_MESSAGE,
  INITIAL_TINA_MESSAGE,
  TINA_ID,
} from './constants';
import { findLatestMessage } from './find-latest-message';
import { hasContact } from './has-contact';
import { initializeSpaceshipContacts } from './initialize-spaceship-contacts';
import { decodeMessageCursor, encodeMessageCursor } from './message-cursor';
import { parseJsonBody } from './parse-json-body';
import { parseSendMessageRequest } from './parse-send-message-request';
import { sendMessage } from './send-message';
import { toMessageDto } from './to-message-dto';

export {
  CONTACTS,
  EASA_CHIEF_ID,
  INITIAL_CHIEF_MESSAGE,
  INITIAL_TINA_MESSAGE,
  TINA_ID,
};
export type {
  ContactProfile,
  ContactReplyOptions,
  ContactShipContext,
} from './constants';

export type { ContactDocument } from '@models';
export type {
  ContactMessageDocument,
  ContactMessageSender,
  ContactMessageStatus,
} from '@models';

export class ContactsService {
  static initializeSpaceshipContacts = initializeSpaceshipContacts;
  static hasContact = hasContact;
  static parseJsonBody = parseJsonBody;
  static toMessageDto = toMessageDto;
  static encodeMessageCursor = encodeMessageCursor;
  static decodeMessageCursor = decodeMessageCursor;
  static findLatestMessage = findLatestMessage;
  static parseSendMessageRequest = parseSendMessageRequest;
  static sendMessage = sendMessage;
}
