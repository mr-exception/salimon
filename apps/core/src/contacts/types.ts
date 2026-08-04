import type { ContactMessageDocument } from '@models';
import type { SpaceshipDto } from '@repo/types';

export type ContactShipProximityTelemetry = {
  bodyName: string;
  bodyKind: 'Planet' | 'Star';
  surfaceDistanceMeters: number;
  relativeSpeedMetersPerSecond: number;
};

export type ContactShipContext = Partial<
  Pick<
    SpaceshipDto,
    | 'position'
    | 'positionCapturedAt'
    | 'direction'
    | 'speed'
    | 'velocity'
    | 'motionState'
    | 'stats'
    | 'inventory'
    | 'activeFeature'
    | 'simulatedAt'
  >
> & {
  proximityTelemetry?: ContactShipProximityTelemetry;
};

export type ContactDocumentCollection = {
  knowledgeContext: string;
  missions: string;
};

export type ContactMessageRequest = {
  contactId: string;
  text: string;
  clientMessageId: string;
  shipContext?: ContactShipContext;
};

export type ContactReplyOptions = {
  onReply?: (message: ContactMessageDocument) => void;
  shipContext?: ContactShipContext;
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
