import { getWorldPosition } from '@store';
import type { Position } from '@types';

export const DEFAULT_RENDER_ORIGIN_NAME = 'Spaceship';

let renderOriginName: string | undefined = DEFAULT_RENDER_ORIGIN_NAME;
let renderOriginPosition: Position | undefined;
let renderOriginOffset = { x: 0n, y: 0n };

export function setRenderOriginName(name: string) {
  renderOriginName = name;
  renderOriginPosition = undefined;
  renderOriginOffset = { x: 0n, y: 0n };
}

export function setRenderOriginPosition(position: Position) {
  renderOriginName = undefined;
  renderOriginPosition = position;
  renderOriginOffset = { x: 0n, y: 0n };
}

export function offsetRenderOrigin(x: number, y: number) {
  renderOriginOffset = {
    x: renderOriginOffset.x + BigInt(Math.round(x)),
    y: renderOriginOffset.y + BigInt(Math.round(y)),
  };
}

export function getRenderOriginWorldPosition(): Position {
  const originPosition =
    renderOriginPosition ??
    getWorldPosition({
      x: 0n,
      y: 0n,
      relativeTo: renderOriginName,
    });

  return {
    x: originPosition.x + renderOriginOffset.x,
    y: originPosition.y + renderOriginOffset.y,
  };
}

export function getRenderPosition(position: Position) {
  const worldPosition = getWorldPosition(position);
  const originPosition = getRenderOriginWorldPosition();

  return {
    x: Number(worldPosition.x - originPosition.x),
    y: Number(worldPosition.y - originPosition.y),
  };
}

export function getWorldPositionFromRenderPosition(x: number, y: number) {
  const originPosition = getRenderOriginWorldPosition();

  return {
    x: originPosition.x + BigInt(Math.round(x)),
    y: originPosition.y + BigInt(Math.round(y)),
  };
}
