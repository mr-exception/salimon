import { EASA_CHIEF_ID, EasaChiefContact } from './easa-chief';
import type { BaseContact } from './base-contact';

export * from './base-contact';
export * from './easa-chief';
export type {
  ContactDocumentCollection,
  ContactMessageRequest,
  ContactProfile,
  ContactReplyOptions,
} from './types';

export type ContactType = typeof EASA_CHIEF_ID;

export const CONTACT_DEFINITIONS = {
  [EASA_CHIEF_ID]: new EasaChiefContact(),
} as const satisfies Record<ContactType, BaseContact>;

export const EASA_CHIEF_CONTACT = CONTACT_DEFINITIONS[EASA_CHIEF_ID];
export const CONTACTS = CONTACT_DEFINITIONS;
export const INITIAL_CHIEF_MESSAGE = EASA_CHIEF_CONTACT.initialMessage;

export function getContactDefinition(contactId: string) {
  return CONTACT_DEFINITIONS[contactId as ContactType];
}
