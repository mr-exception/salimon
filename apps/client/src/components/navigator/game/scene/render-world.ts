import { loadWorld, spaceshipState, subscribeToWorld } from '@store';
import { Spaceship } from '../spaceship';
import type { Scene } from '.';

export async function renderWorld(this: Scene) {
  try {
    const world = await loadWorld();
    if (!this.sys.isActive()) return;

    this.setWorldBodyData(world.planets, world.stars);
    this.spaceship = new Spaceship(this, spaceshipState);
    this.recenterOnSpaceship(false);
    this.unsubscribeFromWorld = subscribeToWorld((_world, changedBodyNames) => {
      this.syncWorldPositions(changedBodyNames);
      if (changedBodyNames) {
        this.updateChangedWorldVisibility(changedBodyNames);
      } else {
        this.updateWorldVisibility();
      }
    });
    this.updateWorldVisibility();
    this.onWorldLoadComplete?.();
  } catch (error) {
    console.error('Failed to load world data', error);
    this.onWorldLoadComplete?.(error);
  }
}
