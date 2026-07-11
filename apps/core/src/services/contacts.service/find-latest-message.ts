import { ContactMessageModel } from '@models';

export function findLatestMessage(
  spaceshipSecurityCode: string,
  contactId: string,
) {
  return ContactMessageModel.findLatest(spaceshipSecurityCode, contactId);
}

