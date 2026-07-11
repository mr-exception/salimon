import { ContactModel } from '@models';

export async function hasContact(
  spaceshipSecurityCode: string,
  contactId: string,
) {
  return Boolean(
    await ContactModel.findBySpaceshipAndContact(
      spaceshipSecurityCode,
      contactId,
    ),
  );
}

