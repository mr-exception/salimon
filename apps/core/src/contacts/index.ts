import { EASA_CHIEF_ID, EasaChiefContact } from './easa-chief';
import { TINA_ID, TinaContact } from './tina';
import type { BaseContact } from './base-contact';

export * from './base-contact';
export * from './easa-chief';
export * from './tina';
export type {
  ContactDocumentCollection,
  ContactMessageRequest,
  ContactProfile,
  ContactReplyOptions,
  ContactShipContext,
} from './types';

export type ContactType = typeof EASA_CHIEF_ID | typeof TINA_ID;

export const CONTACT_DEFINITIONS = {
  [EASA_CHIEF_ID]: new EasaChiefContact(),
  [TINA_ID]: new TinaContact(),
} as const satisfies Record<ContactType, BaseContact>;

export const EASA_CHIEF_CONTACT = CONTACT_DEFINITIONS[EASA_CHIEF_ID];
export const TINA_CONTACT = CONTACT_DEFINITIONS[TINA_ID];
export const INITIAL_CONTACTS = Object.values(CONTACT_DEFINITIONS);
export const CONTACTS = CONTACT_DEFINITIONS;
export const INITIAL_CHIEF_MESSAGE = EASA_CHIEF_CONTACT.initialMessage;
export const INITIAL_TINA_MESSAGE = TINA_CONTACT.initialMessage;

export function getContactDefinition(contactId: string) {
  return CONTACT_DEFINITIONS[contactId as ContactType];
}
