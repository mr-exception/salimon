import type { WorldBodyDocument } from '@models';

export function cloneBody(body: WorldBodyDocument): WorldBodyDocument {
  return {
    ...body,
    position: { ...body.position },
    updatedAt: new Date(body.updatedAt),
  };
}

