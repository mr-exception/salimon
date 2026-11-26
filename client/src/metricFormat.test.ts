import { test } from 'node:test'
import assert from 'node:assert/strict'
import { formatDistance, formatSpeed } from './metricFormat.ts'

test('adaptive metric distance and velocity examples', () => {
  for (const [input, expected] of [[850, '850 m'], [12500, '12.5 km'], [24500000, '24.5 Mm'], [149600000000, '149.6 Bm'], [4250000000000, '4.25 Tm'], [2e15, '2 Pm']] as const) {
    assert.equal(formatDistance(input), expected)
    assert.equal(formatDistance(-input), `-${expected}`)
    assert.equal(formatSpeed(input), `${expected}/s`)
  }
  assert.equal(formatSpeed(245.5), '245.5 m/s')
  assert.equal(formatSpeed(12400), '12.4 km/s')
  assert.equal(formatSpeed(2500000), '2.5 Mm/s')
})

test('zero, tiny signed values, invalid values and maximum finite distance', () => {
  for (const input of [0, -0, 0.001, -0.001, Number.MIN_VALUE, -Number.MIN_VALUE]) {
    assert.equal(formatDistance(input), '0 m')
    assert.equal(formatSpeed(input), '0 m/s')
  }
  assert.equal(formatDistance(-0.01), '-0.01 m')
  for (const input of [NaN, Infinity, -Infinity]) {
    assert.equal(formatDistance(input), '—')
    assert.equal(formatSpeed(input), '—')
  }
  assert.equal(formatDistance(1e18), '1000 Pm')
  assert.equal(formatDistance(Number.MAX_VALUE), '1.8e+293 Pm')
})

test('each boundary promotes on magnitude and rounding carry in either direction', () => {
  const units = ['m', 'km', 'Mm', 'Bm', 'Tm', 'Pm']
  for (let i = 0; i < units.length - 1; i++) {
    const scale = 1000 ** i
    for (const sign of [-1, 1]) {
      const prefix = sign < 0 ? '-' : ''
      for (const [value, expected] of [[999.994, `999.99 ${units[i]}`], [999.996, `1 ${units[i + 1]}`], [1000, `1 ${units[i + 1]}`]] as const) {
        assert.equal(formatDistance(sign * value * scale), prefix + expected)
        assert.equal(formatSpeed(sign * value * scale), prefix + expected + '/s')
      }
    }
  }
  assert.equal(formatDistance(12.3451), '12.35 m')
  assert.equal(formatDistance(12.3), '12.3 m')
})
