import { cmp, decode, ZERO } from '../exact.ts'

export function gearCondition(remaining: string) {
  const charge = decode(remaining)
  if (cmp(charge, ZERO) === 0) return 'broken'
  if (cmp(charge, [400n, 1n]) < 0) return 'deteriorated'
  if (cmp(charge, [800n, 1n]) < 0) return 'worn'
  return 'maintained'
}
