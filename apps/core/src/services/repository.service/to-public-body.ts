import type { WorldBodyDocument } from '@models';
import { cloneBody } from './clone-body';

export function toPublicBody(body: WorldBodyDocument): WorldBodyDocument {
  const { _id, ...publicBody } = cloneBody(body);
  void _id;
  return publicBody as WorldBodyDocument;
}

