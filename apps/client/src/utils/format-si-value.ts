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
const SUBMETER_UNITS = [
  { unit: 'mm', meters: 0.001 },
  { unit: 'µm', meters: 0.000001 },
  { unit: 'nm', meters: 0.000000001 },
  { unit: 'pm', meters: 0.000000000001 },
];
const METERS_PER_LIGHT_SECOND = 299_792_458;
const METERS_PER_LIGHT_MINUTE = METERS_PER_LIGHT_SECOND * 60;
const METERS_PER_LIGHT_HOUR = METERS_PER_LIGHT_MINUTE * 60;
const METERS_PER_LIGHT_DAY = METERS_PER_LIGHT_HOUR * 24;
const METERS_PER_LIGHT_YEAR = 9_460_730_472_580_800;
const LIGHT_DISTANCE_UNITS = [
  { unit: 'ly', meters: METERS_PER_LIGHT_YEAR },
  { unit: 'ld', meters: METERS_PER_LIGHT_DAY },
  { unit: 'lh', meters: METERS_PER_LIGHT_HOUR },
  { unit: 'lm', meters: METERS_PER_LIGHT_MINUTE },
  { unit: 'ls', meters: METERS_PER_LIGHT_SECOND },
];

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
  if (!Number.isFinite(valueInMeters)) return '—';

  const absoluteValue = Math.abs(valueInMeters);
  if (absoluteValue >= METERS_PER_LIGHT_YEAR) {
    return formatUnitValue(valueInMeters / METERS_PER_LIGHT_YEAR, 'ly');
  }

  if (absoluteValue > 0 && absoluteValue < 1) {
    const unit =
      SUBMETER_UNITS.find(({ meters }) => absoluteValue >= meters) ??
      SUBMETER_UNITS.at(-1);
    if (unit) return formatUnitValue(valueInMeters / unit.meters, unit.unit);
  }

  return formatSiValue(valueInMeters, 'm');
}

export function formatLightDistance(valueInMeters: number) {
  if (!Number.isFinite(valueInMeters)) return '—';

  const absoluteValue = Math.abs(valueInMeters);
  const unit =
    LIGHT_DISTANCE_UNITS.find(({ meters }) => absoluteValue >= meters) ??
    LIGHT_DISTANCE_UNITS.at(-1);

  if (!unit) return '—';

  return formatUnitValue(valueInMeters / unit.meters, unit.unit);
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
