import type { SerializedPosition, WorldBodyDocument } from '@models';
import { WorldService } from '@repo/world';

export function advanceBodyPosition(
  body: WorldBodyDocument,
  elapsedSeconds: number,
): SerializedPosition {
  return WorldService.advanceBodyPosition(body, elapsedSeconds);
}
