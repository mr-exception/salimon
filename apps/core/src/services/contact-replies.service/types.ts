import type { ContactShipContext } from '../contacts.service';

export type ReplyJob = {
  spaceshipSecurityCode: string;
  contactId: string;
  playerMessageId: string;
  shipContext?: ContactShipContext;
};
