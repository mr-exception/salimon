import { spaceshipState, subscribeToWorld } from '@store';
import { Spaceship } from '../spaceship';
import type { Scene } from '.';

export async function renderWorld(this: Scene) {
  try {
    this.spaceship = new Spaceship(this, spaceshipState);
    this.recenterOnSpaceship(false);
    await this.refreshWorldFromViewport();
    if (!this.sys.isActive()) return;
    if (this.initialFocus === 'spaceship') {
      this.recenterOnSpaceship(false);
    } else {
      this.focusOnAttachedBodyAsteroidOrbit();
    }
    await this.reconcileClientAsteroids();
    if (!this.sys.isActive()) return;

    this.unsubscribeFromWorld = subscribeToWorld((_world, changedBodyNames) => {
      this.setWorldBodyData(_world.planets, _world.stars);
      this.syncWorldPositions(changedBodyNames);
      if (changedBodyNames) {
        this.updateChangedWorldVisibility(changedBodyNames);
      } else {
        void this.reconcileClientAsteroids();
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
