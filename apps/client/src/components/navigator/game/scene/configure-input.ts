import Phaser from 'phaser';
import type { Scene } from '.';

export const MIN_ZOOM = 0.000000000000001;
export const MAX_ZOOM = 0.1;

export function configureInput(this: Scene) {
  const camera = this.cameras.main;
  const canvas = this.game.canvas;
  const openContextMenu = (event: MouseEvent) => {
    event.preventDefault();

    const bounds = canvas.getBoundingClientRect();
    const x = ((event.clientX - bounds.left) / bounds.width) * this.scale.width;
    const y =
      ((event.clientY - bounds.top) / bounds.height) * this.scale.height;
    this.openBodyContextMenuAt(x, y);
  };

  canvas.style.cursor = 'grab';
  canvas.addEventListener('contextmenu', openContextMenu);

  this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
    canvas.removeEventListener('contextmenu', openContextMenu);
  });

  this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
    if (!pointer.leftButtonDown()) return;
    if (this.isTargetDirectionSelectionActive()) return;

    this.dragging = true;
    this.lastPointer.set(pointer.x, pointer.y);
    canvas.style.cursor = 'grabbing';
  });

  this.input.on('pointermove', (pointer: Phaser.Input.Pointer) => {
    if (this.isTargetDirectionSelectionActive()) {
      this.previewTargetDirectionAt(pointer.x, pointer.y);
      return;
    }
    if (!this.dragging || !pointer.leftButtonDown()) return;

    this.releaseCameraLock();

    const previousWorldPoint = camera.getWorldPoint(
      this.lastPointer.x,
      this.lastPointer.y,
    );
    const currentWorldPoint = camera.getWorldPoint(pointer.x, pointer.y);
    const deltaX = currentWorldPoint.x - previousWorldPoint.x;
    const deltaY = currentWorldPoint.y - previousWorldPoint.y;
    if (deltaX === 0 && deltaY === 0) return;

    camera.scrollX -= deltaX;
    camera.scrollY -= deltaY;
    this.rebaseRenderOriginAtCameraCenter();
    this.lastPointer.set(pointer.x, pointer.y);
  });

  this.input.on('pointerup', (pointer: Phaser.Input.Pointer) => {
    if (this.isTargetDirectionSelectionActive()) {
      if (pointer.leftButtonReleased()) {
        this.selectTargetDirectionAt(pointer.x, pointer.y);
      }
      return;
    }

    this.dragging = false;
    canvas.style.cursor = 'grab';
  });

  this.input.on(
    'wheel',
    (
      _pointer: Phaser.Input.Pointer,
      _objects: unknown[],
      _deltaX: number,
      deltaY: number,
    ) => {
      const nextZoom = Phaser.Math.Clamp(
        camera.zoom * Math.exp(-deltaY * 0.0015),
        MIN_ZOOM,
        MAX_ZOOM,
      );

      this.setZoom(nextZoom);
    },
  );

  this.input.on('gameout', () => {
    this.dragging = false;
    this.hideTargetDirectionPreview();
    canvas.style.cursor = this.isTargetDirectionSelectionActive()
      ? 'crosshair'
      : 'grab';
  });
}
