import { test } from 'node:test'
import assert from 'node:assert/strict'
import { meterValue } from './ui/values.ts'
test('UI readings bound malformed observations without changing the source', () => {
  assert.deepEqual(meterValue(250, 200), { value: 200, max: 200 })
  assert.deepEqual(meterValue(-10, 200), { value: 0, max: 200 })
  assert.deepEqual(meterValue(NaN, Infinity), { value: 0, max: 1 })
  assert.deepEqual(meterValue(5, 0), { value: 1, max: 1 })
  assert.deepEqual(meterValue(4.25, 10), { value: 4.25, max: 10 })
})
