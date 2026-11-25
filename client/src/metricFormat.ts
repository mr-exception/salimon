/** Display only: Bm is the game's billion metres (SI: Gm). SI inputs stay unchanged. */
const UNITS = ['m', 'km', 'Mm', 'Bm', 'Tm', 'Pm'] as const

function formatMetric(value: number, velocity: boolean): string {
  if (!Number.isFinite(value)) return '—'
  let unit = 0
  while (unit < UNITS.length - 1 && Math.abs(value) >= 1000 ** (unit + 1)) unit++
  let rounded = Number((value / 1000 ** unit).toFixed(2))
  // Rounding may cross a unit boundary, including for negative values.
  if (Math.abs(rounded) >= 1000 && unit < UNITS.length - 1) {
    unit++
    rounded = Number((value / 1000 ** unit).toFixed(2))
  }
  // Beyond fixed notation, keep the scientific mantissa to two decimals too.
  const text = Math.abs(rounded) >= 1e21 ? String(Number(rounded.toExponential(2))) : String(rounded)
  // Number removes trailing zeros; its string conversion also normalizes -0.
  return `${text} ${UNITS[unit]}${velocity ? '/s' : ''}`
}

export const formatDistance = (metres: number): string => formatMetric(metres, false)
export const formatSpeed = (metresPerSecond: number): string => formatMetric(metresPerSecond, true)
