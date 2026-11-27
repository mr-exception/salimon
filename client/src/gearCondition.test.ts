import { strict as assert } from 'node:assert'
import { test } from 'node:test'
import { gearCondition } from './rendering/gearCondition.ts'

test('equipment artwork resolves exact condition at all band edges', () => {
  assert.equal(gearCondition('0/1'), 'broken')
  assert.equal(gearCondition('1/1000'), 'deteriorated')
  assert.equal(gearCondition('399999/1000'), 'deteriorated')
  assert.equal(gearCondition('400/1'), 'worn')
  assert.equal(gearCondition('799999/1000'), 'worn')
  assert.equal(gearCondition('800/1'), 'maintained')
  assert.throws(() => gearCondition('400/0'))
})
