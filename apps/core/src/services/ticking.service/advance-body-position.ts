import type { SerializedPosition, WorldBodyDocument } from '@models';
import { FULL_ROTATION_RADIANS } from './constants';

export function advanceBodyPosition(
  body: WorldBodyDocument,
  elapsedSeconds: number,
): SerializedPosition {
  const x = BigInt(body.position.x);
  const y = BigInt(body.position.y);
  const orbitalRadius = Math.hypot(Number(x), Number(y));
  const speed = Number(BigInt(body.speed));

  if (
    !body.orbitalCenter ||
    orbitalRadius === 0 ||
    speed === 0 ||
    elapsedSeconds <= 0
  ) {
    return body.position;
  }

  const direction = body.clockwise ? 1 : -1;
  const angle =
    ((direction * speed * elapsedSeconds) / orbitalRadius) %
    FULL_ROTATION_RADIANS;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);

  return {
    x: BigInt(Math.round(Number(x) * cos - Number(y) * sin)).toString(),
    y: BigInt(Math.round(Number(x) * sin + Number(y) * cos)).toString(),
    ...(body.position.relativeTo
      ? { relativeTo: body.position.relativeTo }
      : {}),
  };
}
