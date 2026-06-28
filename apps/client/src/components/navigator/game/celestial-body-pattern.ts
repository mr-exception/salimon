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

const PLANET_PATTERNS: readonly Pattern[] = [
  [{ kind: 'circle', x: 0.45, y: 0, radius: 0.08, filled: true }],
  [{ kind: 'line', fromX: -0.85, fromY: 0, toX: 0.85, toY: 0 }],
  [{ kind: 'line', fromX: 0, fromY: -0.85, toX: 0, toY: 0.85 }],
  [{ kind: 'line', fromX: -0.6, fromY: 0.6, toX: 0.6, toY: -0.6 }],
  [
    { kind: 'circle', x: -0.42, y: 0, radius: 0.1, filled: true },
    { kind: 'circle', x: 0.42, y: 0, radius: 0.1, filled: true },
  ],
  [{ kind: 'circle', x: 0, y: 0, radius: 0.55 }],
  [
    { kind: 'circle', x: 0, y: -0.42, radius: 0.09, filled: true },
    { kind: 'circle', x: -0.36, y: 0.24, radius: 0.09, filled: true },
    { kind: 'circle', x: 0.36, y: 0.24, radius: 0.09, filled: true },
  ],
  [
    { kind: 'line', fromX: -0.72, fromY: 0, toX: 0.72, toY: 0 },
    { kind: 'line', fromX: 0, fromY: -0.72, toX: 0, toY: 0.72 },
  ],
  [
    { kind: 'circle', x: 0, y: 0, radius: 0.5 },
    { kind: 'circle', x: 0, y: 0, radius: 0.13, filled: true },
  ],
  [
    { kind: 'line', fromX: -0.55, fromY: -0.45, toX: 0, toY: 0.1 },
    { kind: 'line', fromX: 0, fromY: 0.1, toX: 0.55, toY: -0.45 },
    { kind: 'line', fromX: -0.55, fromY: 0.15, toX: 0, toY: 0.7 },
    { kind: 'line', fromX: 0, fromY: 0.7, toX: 0.55, toY: 0.15 },
  ],
];

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

export function drawPlanetPattern(
  graphics: Phaser.GameObjects.Graphics,
  variant: number,
  color: number,
  radius: number,
) {
  graphics.clear();
  drawPattern(
    graphics,
    PLANET_PATTERNS[variant] ?? PLANET_PATTERNS[0],
    color,
    radius,
  );
}

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
