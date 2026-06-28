export type DistUnit = 'm' | 'km' | 'Mm' | 'Gm' | 'Tm';

const METERS_PER_UNIT: Record<DistUnit, number> = {
  m: 1,
  km: 1_000,
  Mm: 1_000_000,
  Gm: 1_000_000_000,
  Tm: 1_000_000_000_000,
};

/**
 * Converts a distance to pixels using `zoomLevel` as the number of pixels per
 * meter.
 */
export function distToPx(
  amount: number,
  unit: DistUnit,
  zoomLevel: number,
): number {
  return amount * METERS_PER_UNIT[unit] * zoomLevel;
}
