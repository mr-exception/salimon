import type { ContactMessageDocument } from '@models';

export type ContactDocumentCollection = {
  knowledgeContext: string;
  missions: string;
};

export type ContactMessageRequest = {
  contactId: string;
  text: string;
  clientMessageId: string;
};

export type ContactReplyOptions = {
  onReply?: (message: ContactMessageDocument) => void;
};

export type ContactProfile = {
  id: string;
  name: string;
  description: string;
  position: string;
  organization: string;
  role: string;
  documents: ContactDocumentCollection;
  background: readonly string[];
  worldGoal: readonly string[];
  personality: readonly string[];
  speakingStyle: readonly string[];
  knownCanon: readonly string[];
  unknowns: readonly string[];
  boundaries: readonly string[];
  initialMessage: string;
};
