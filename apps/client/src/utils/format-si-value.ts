const SI_PREFIX_SYMBOLS = [
  '',
  'k',
  'M',
  'G',
  'T',
  'P',
  'E',
  'Z',
  'Y',
  'R',
  'Q',
];
const QUETTA_PREFIX_INDEX = SI_PREFIX_SYMBOLS.length - 1;

const numberFormatter = new Intl.NumberFormat('en-US', {
  maximumFractionDigits: 1,
});

function formatUnitValue(value: number, unit: string) {
  if (!Number.isFinite(value)) return '—';

  return `${numberFormatter.format(value)} ${unit}`;
}

export function formatSiValue(value: number, unit: string) {
  if (!Number.isFinite(value)) return '—';

  let formattedValue = value;
  let prefixIndex = 0;

  while (Math.abs(formattedValue) >= 1_000) {
    formattedValue /= 1_000;
    prefixIndex += 1;
  }

  if (Math.abs(Math.round(formattedValue * 10) / 10) >= 1_000) {
    formattedValue /= 1_000;
    prefixIndex += 1;
  }

  return formatUnitValue(formattedValue, `${getSiPrefix(prefixIndex)}${unit}`);
}

function getSiPrefix(index: number): string {
  if (index <= 0) return '';

  const prefix = SI_PREFIX_SYMBOLS[index];
  if (prefix) return prefix;

  return `${getCompoundSiPrefix(index - QUETTA_PREFIX_INDEX)}Q`;
}

function getCompoundSiPrefix(index: number): string {
  if (index === 1) return 'K';

  const prefix = SI_PREFIX_SYMBOLS[index];
  if (prefix) return prefix.toUpperCase();

  return `${getCompoundSiPrefix(index - QUETTA_PREFIX_INDEX)}Q`;
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
