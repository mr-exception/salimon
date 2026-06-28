const SI_PREFIXES = [
  { minimum: 1_000_000_000_000, symbol: 'T' },
  { minimum: 1_000_000_000, symbol: 'G' },
  { minimum: 1_000_000, symbol: 'M' },
  { minimum: 1_000, symbol: 'k' },
] as const;

const numberFormatter = new Intl.NumberFormat('en-US', {
  maximumFractionDigits: 1,
});

function formatUnitValue(value: number, unit: string) {
  if (!Number.isFinite(value)) return '—';

  return `${numberFormatter.format(value)} ${unit}`;
}

export function formatSiValue(value: number, unit: string) {
  if (!Number.isFinite(value)) return '—';

  const prefix = SI_PREFIXES.find(({ minimum }) => Math.abs(value) >= minimum);
  const divisor = prefix?.minimum ?? 1;

  return formatUnitValue(value / divisor, `${prefix?.symbol ?? ''}${unit}`);
}

export function formatDistance(valueInMeters: number) {
  return formatSiValue(valueInMeters, 'm');
}

export function formatForce(valueInNewtons: number) {
  return formatSiValue(valueInNewtons, 'N');
}

export function formatImpulse(valueInNewtonSeconds: number) {
  return formatSiValue(valueInNewtonSeconds, 'Ns');
}

export function formatSpeed(valueInMetersPerSecond: number) {
  return formatSiValue(valueInMetersPerSecond, 'm/s');
}

export function formatAcceleration(valueInMetersPerSecondSquared: number) {
  return formatSiValue(valueInMetersPerSecondSquared, 'm/s²');
}

export function formatDuration(valueInSeconds: number) {
  return formatUnitValue(valueInSeconds, 's');
}

export function formatPercentage(value: number) {
  return formatUnitValue(value, '%');
}

export function formatAngle(valueInDegrees: number) {
  return formatUnitValue(valueInDegrees, '°').replace(' °', '°');
}
