import Phaser from 'phaser';

const FULL_CIRCLE_MAX_SCREEN_RADIUS = 128;
const ARC_SEGMENT_SCREEN_LENGTH = 4;

export function drawCelestialBody(
  graphics: Phaser.GameObjects.Graphics,
  color: number,
  radius: number,
  zoom: number,
  viewport: Phaser.Geom.Rectangle,
  centerX: number,
  centerY: number,
) {
  graphics.clear().fillStyle(color);
  const screenRadius = radius * zoom;
  if (screenRadius <= FULL_CIRCLE_MAX_SCREEN_RADIUS) {
    graphics.fillCircle(0, 0, radius);
    return;
  }

  const centerDistance = Math.hypot(
    viewport.centerX - centerX,
    viewport.centerY - centerY,
  );
  const viewportRadius = Math.hypot(viewport.width, viewport.height) / 2;
  if (centerDistance + viewportRadius <= radius) {
    graphics.fillRect(
      viewport.left - centerX,
      viewport.top - centerY,
      viewport.width,
      viewport.height,
    );
    return;
  }
  if (centerDistance <= viewportRadius) {
    const segmentCount = Phaser.Math.Clamp(
      Math.ceil((Math.PI * 2 * screenRadius) / ARC_SEGMENT_SCREEN_LENGTH),
      32,
      2_048,
    );
    const points: Phaser.Math.Vector2[] = [];
    for (let index = 0; index < segmentCount; index += 1) {
      const angle = (Math.PI * 2 * index) / segmentCount;
      points.push(
        new Phaser.Math.Vector2(
          Math.cos(angle) * radius,
          Math.sin(angle) * radius,
        ),
      );
    }
    graphics.fillPoints(points, true);
    return;
  }

  const direction = Math.atan2(
    viewport.centerY - centerY,
    viewport.centerX - centerX,
  );
  const halfAngle = Math.asin(
    Math.min(
      1,
      (viewportRadius + ARC_SEGMENT_SCREEN_LENGTH / zoom) / centerDistance,
    ),
  );
  const segmentCount = Phaser.Math.Clamp(
    Math.ceil((2 * halfAngle * screenRadius) / ARC_SEGMENT_SCREEN_LENGTH),
    2,
    2_048,
  );
  const points = [new Phaser.Math.Vector2(0, 0)];
  for (let index = 0; index <= segmentCount; index += 1) {
    const angle =
      direction - halfAngle + (2 * halfAngle * index) / segmentCount;
    points.push(
      new Phaser.Math.Vector2(
        Math.cos(angle) * radius,
        Math.sin(angle) * radius,
      ),
    );
  }
  graphics.fillPoints(points, true);
}
