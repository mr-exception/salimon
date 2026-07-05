import type { ScheduledEvent } from 'aws-lambda';
import { updateOrbitalBodies } from '../../orbital-updater';

export function handler(event: ScheduledEvent) {
  return updateOrbitalBodies(event, {
    collectionName: 'stars',
  });
}
