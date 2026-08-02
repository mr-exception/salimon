import { INITIAL_CONTACTS } from '../../contacts';

export async function initializeSpaceshipContacts(
  spaceshipSecurityCode: string,
) {
  await Promise.all(
    INITIAL_CONTACTS.map((contact) =>
      contact.triggerFirstMessage(spaceshipSecurityCode),
    ),
  );
}
