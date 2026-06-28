import Phaser from 'phaser';

type PatternElement =
  | {
      kind: 'circle';
      x: number;
      y: number;
      radius: number;
      filled?: boolean;
    }
  | {
      kind: 'line';
      fromX: number;
      fromY: number;
      toX: number;
      toY: number;
    };

type Pattern = readonly PatternElement[];

const STAR_PATTERNS: readonly Pattern[] = [
  [
    { kind: 'line', fromX: -0.75, fromY: 0, toX: 0.75, toY: 0 },
    { kind: 'line', fromX: 0, fromY: -0.75, toX: 0, toY: 0.75 },
  ],
  [
    { kind: 'line', fromX: -0.75, fromY: 0, toX: 0.75, toY: 0 },
    { kind: 'line', fromX: -0.38, fromY: -0.65, toX: 0.38, toY: 0.65 },
    { kind: 'line', fromX: 0.38, fromY: -0.65, toX: -0.38, toY: 0.65 },
  ],
  [
    { kind: 'circle', x: 0, y: 0, radius: 0.55 },
    { kind: 'circle', x: 0, y: 0, radius: 0.14, filled: true },
  ],
  [
    { kind: 'circle', x: 0.48, y: 0, radius: 0.1, filled: true },
    { kind: 'circle', x: 0, y: 0.48, radius: 0.1, filled: true },
    { kind: 'circle', x: -0.48, y: 0, radius: 0.1, filled: true },
    { kind: 'circle', x: 0, y: -0.48, radius: 0.1, filled: true },
  ],
];

export function drawStarPattern(
  graphics: Phaser.GameObjects.Graphics,
  variant: number,
  color: number,
  radius: number,
) {
  drawPattern(
    graphics,
    STAR_PATTERNS[variant] ?? STAR_PATTERNS[0],
    color,
    radius,
  );
}

function drawPattern(
  graphics: Phaser.GameObjects.Graphics,
  pattern: Pattern,
  color: number,
  radius: number,
) {
  const lineWidth = Math.max(1, radius * 0.09);
  graphics.fillStyle(color, 0.95).lineStyle(lineWidth, color, 0.95);

  for (const element of pattern) {
    if (element.kind === 'circle') {
      const x = element.x * radius;
      const y = element.y * radius;
      const elementRadius = element.radius * radius;
      if (element.filled) {
        graphics.fillCircle(x, y, elementRadius);
      } else {
        graphics.strokeCircle(x, y, elementRadius);
      }
      continue;
    }

    graphics
      .beginPath()
      .moveTo(element.fromX * radius, element.fromY * radius)
      .lineTo(element.toX * radius, element.toY * radius)
      .strokePath();
  }
}
