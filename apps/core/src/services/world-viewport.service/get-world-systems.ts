import type { SerializedWorldSystems } from '@repo/types';
import { WorldAssetService } from '../world-asset.service';
import type { WorldViewportOptions, WorldViewportRequest } from './types';

export async function getWorldSystems(
  request: WorldViewportRequest,
  options: WorldViewportOptions = {},
): Promise<SerializedWorldSystems> {
  void request;
  void options;
  return WorldAssetService.getWorldSystems();
}
