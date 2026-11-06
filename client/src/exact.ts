/** Reduced rational arithmetic. Decimal input means exactly the declared decimal, not a rounded tick. */
export type Exact = readonly [bigint, bigint]
export const ZERO: Exact = [0n, 1n]
export const ONE: Exact = [1n, 1n]
export function ratio(n: bigint, d = 1n): Exact {
  if (d <= 0n) throw new RangeError('Invalid denominator')
  let a = n < 0n ? -n : n, b = d
  while (b) [a, b] = [b, a % b]
  return [n / a, d / a]
}
export const add = (a: Exact, b: Exact) => ratio(a[0] * b[1] + b[0] * a[1], a[1] * b[1])
export const neg = (a: Exact): Exact => [-a[0], a[1]]
export const sub = (a: Exact, b: Exact) => add(a, neg(b))
export const mul = (a: Exact, b: Exact) => ratio(a[0] * b[0], a[1] * b[1])
export const div = (a: Exact, b: Exact) => { if (b[0] <= 0n) throw new RangeError('Invalid divisor'); return ratio(a[0] * b[1], a[1] * b[0]) }
export const cmp = (a: Exact, b: Exact) => a[0] * b[1] < b[0] * a[1] ? -1 : a[0] * b[1] > b[0] * a[1] ? 1 : 0
export const min = (a: Exact, b: Exact) => cmp(a, b) < 0 ? a : b
export const max = (a: Exact, b: Exact) => cmp(a, b) > 0 ? a : b
export const encode = (a: Exact) => `${a[0]}/${a[1]}`
export const number = (a: Exact) => {
  const sign = a[0] < 0n ? '-' : '', n = a[0] < 0n ? -a[0] : a[0]
  const fraction = ((n % a[1]) * 10n ** 340n / a[1]).toString().padStart(340, '0')
  return Number(`${sign}${n / a[1]}.${fraction}`)
}
export function decode(s: unknown): Exact {
  if (typeof s !== 'string' || !/^-?\d{1,2048}\/[1-9]\d{0,2047}$/.test(s)) throw new Error('Invalid exact value')
  const [n, d] = s.split('/').map(BigInt), r = ratio(n!, d!)
  if (encode(r) !== s) throw new Error('Noncanonical exact value')
  return r
}
export function decimal(n: number): Exact {
  if (!Number.isFinite(n)) throw new RangeError('Invalid decimal')
  const [mantissa, exponent = '0'] = String(n).split('e'), [whole, fraction = ''] = mantissa!.split('.')
  const scale = fraction.length - Number(exponent), value = BigInt(whole! + fraction)
  return scale >= 0 ? ratio(value, 10n ** BigInt(scale)) : ratio(value * 10n ** BigInt(-scale))
}
