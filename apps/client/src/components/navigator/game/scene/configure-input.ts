import Phaser from 'phaser';
import type { Scene } from '.';

export const MIN_ZOOM = 0.000000000000005;
export const MAX_ZOOM = 0.1;
const MIN_PAN_EFFECTIVE_ZOOM = 0.000000000000005;
const CLICK_DISTANCE_THRESHOLD_PX = 5;

export function configureInput(this: Scene) {
  const camera = this.cameras.main;
  const canvas = this.game.canvas;
  let clickStart: Phaser.Math.Vector2 | undefined;
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
    if (this.isTargetDirectionSelectionActive() || this.isRulerActive()) {
      clickStart = new Phaser.Math.Vector2(pointer.x, pointer.y);
      return;
    }

    this.dragging = true;
    this.lastPointer.set(pointer.x, pointer.y);
    clickStart = new Phaser.Math.Vector2(pointer.x, pointer.y);
    canvas.style.cursor = 'grabbing';
  });

  this.input.on('pointermove', (pointer: Phaser.Input.Pointer) => {
    if (this.isRulerActive()) {
      this.previewRulerAt(pointer.x, pointer.y);
      return;
    }
    if (this.isTargetDirectionSelectionActive()) {
      this.previewTargetDirectionAt(pointer.x, pointer.y);
      return;
    }
    if (!this.dragging || !pointer.leftButtonDown()) return;

    this.releaseCameraLock();

    const effectiveZoom = Math.max(camera.zoom, MIN_PAN_EFFECTIVE_ZOOM);
    const deltaX = (pointer.x - this.lastPointer.x) / effectiveZoom;
    const deltaY = (pointer.y - this.lastPointer.y) / effectiveZoom;
    if (deltaX === 0 && deltaY === 0) return;

    camera.scrollX -= deltaX;
    camera.scrollY -= deltaY;
    this.rebaseRenderOriginAtCameraCenter();
    this.queueViewportWorldRefresh();
    this.lastPointer.set(pointer.x, pointer.y);
  });

  this.input.on('pointerup', (pointer: Phaser.Input.Pointer) => {
    if (this.isRulerActive()) {
      if (
        pointer.leftButtonReleased() &&
        clickStart &&
        Phaser.Math.Distance.Between(
          clickStart.x,
          clickStart.y,
          pointer.x,
          pointer.y,
        ) <= CLICK_DISTANCE_THRESHOLD_PX
      ) {
        this.selectRulerPointAt(pointer.x, pointer.y);
      }
      clickStart = undefined;
      return;
    }

    if (this.isTargetDirectionSelectionActive()) {
      if (pointer.leftButtonReleased()) {
        this.selectTargetDirectionAt(pointer.x, pointer.y);
      }
      return;
    }

    this.dragging = false;
    canvas.style.cursor = 'grab';
    if (
      pointer.leftButtonReleased() &&
      clickStart &&
      Phaser.Math.Distance.Between(
        clickStart.x,
        clickStart.y,
        pointer.x,
        pointer.y,
      ) <= CLICK_DISTANCE_THRESHOLD_PX
    ) {
      this.openBodyDetailsAt(pointer.x, pointer.y);
    }
    clickStart = undefined;
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
      this.queueViewportWorldRefresh();
    },
  );

  this.input.on('gameout', () => {
    this.dragging = false;
    clickStart = undefined;
    this.hideTargetDirectionPreview();
    this.hideRulerPreview();
    canvas.style.cursor = this.isTargetDirectionSelectionActive()
      ? 'crosshair'
      : this.isRulerActive()
        ? 'crosshair'
        : 'grab';
  });
}
