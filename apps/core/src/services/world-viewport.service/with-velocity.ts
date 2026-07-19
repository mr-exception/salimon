import type { WorldBodyDocument } from '@models';
import type { Velocity } from '@repo/types';
import type { WorldBodyResponse } from './types';

export function withVelocity(
  body: WorldBodyDocument,
  velocities: Map<string, Velocity>,
): WorldBodyResponse {
  const { updatedAt, ...responseBody } = body;

  return {
    ...responseBody,
    velocity: velocities.get(body.name) ?? { x: 0, y: 0 },
    cTime: updatedAt.getTime(),
  };
}
