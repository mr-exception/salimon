import { EASA_CHIEF_CONTACT } from '../../contacts';

export async function initializeSpaceshipContacts(
  spaceshipSecurityCode: string,
) {
  await EASA_CHIEF_CONTACT.triggerFirstMessage(spaceshipSecurityCode);
}
