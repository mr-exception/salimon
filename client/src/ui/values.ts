// Defensive presentation bounds never write back to authoritative state.
export function meterValue(value: number, max: number) {
  const bound = Number.isFinite(max) && max > 0 ? max : 1
  return { value: Number.isFinite(value) ? Math.min(bound, Math.max(0, value)) : 0, max: bound }
}
