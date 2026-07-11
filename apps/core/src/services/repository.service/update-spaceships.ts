import type { SpaceshipDocument } from '@models';
import { requireSpaceshipsBySecurityCode } from './state';
import { start } from './start';

export async function updateSpaceships(
  updater: (
    spaceshipsBySecurityCode: Map<string, SpaceshipDocument>,
  ) => number,
): Promise<number> {
  await start();
  return updater(requireSpaceshipsBySecurityCode());
}

