import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  STARTING_CAPACITY_KG, STARTING_CAPACITY_SLOTS, gatherResource,
  increaseResourceInventoryCapacity, removeResource, resourceInventoryUsage,
  addCountItem, removeCountItem, startingResourceInventory,
  type ResourceCatalog, type ResourceSource, type CountItemCatalog,
} from './resourceInventory.ts'

const resources: ResourceCatalog = { ore: {}, biomass: {}, ice: {} }
const items: CountItemCatalog = {
  module: { gramsPerItem: 1500, stackSize: 1 },
  ration: { gramsPerItem: 200, stackSize: 5 },
}
const source = (id: string, remainingGrams = 100_000): ResourceSource =>
  ({ id: 'source:' + id, resourceId: id, remainingGrams })

test('starter weight is 10 kg and personal inventory has 12 shared slots', () => {
  const inv = startingResourceInventory()
  assert.equal(inv.capacityGrams, STARTING_CAPACITY_KG * 1000)
  assert.equal(inv.capacitySlots, STARTING_CAPACITY_SLOTS)
  assert.deepEqual(resourceInventoryUsage(inv, resources), { grams: 0, slots: 0 })
})
test('mining transfers precise grams, bounded by remaining weight', () => {
  const initial = startingResourceInventory()
  const result = gatherResource(initial, source('ore'), 25_000, resources)
  assert.equal(result.collectedGrams, 10_000)
  assert.deepEqual(resourceInventoryUsage(result.inventory, resources), { grams: 10_000, slots: 1 })
  assert.equal(result.source.remainingGrams, 90_000)
  assert.deepEqual(initial.quantities, {})
  assert.equal(gatherResource(result.inventory, result.source, 1000, resources).collectedGrams, 0)
})
test('biomass has no independent volume cap', () => {
  const result = gatherResource(startingResourceInventory(), source('biomass', 15_000), 15_000, resources)
  assert.equal(result.collectedGrams, 10_000)
  assert.equal(result.source.remainingGrams, 5000)
})
test('multiple mined resources contribute to the same weight budget', () => {
  const mined = gatherResource(startingResourceInventory(), source('ore'), 3000, resources).inventory
  const result = gatherResource(mined, source('biomass'), 9000, resources)
  assert.equal(result.collectedGrams, 7000)
  assert.deepEqual(resourceInventoryUsage(result.inventory, resources), { grams: 10_000, slots: 2 })
})
test('single-stack modules and stacked count items consume slots and weight', () => {
  const base = startingResourceInventory()
  const withModules = addCountItem(base, 'module', 2, resources, items)
  assert.deepEqual(resourceInventoryUsage(withModules, resources, items), { grams: 3000, slots: 2 })
  const withRations = addCountItem(withModules, 'ration', 6, resources, items)
  assert.deepEqual(resourceInventoryUsage(withRations, resources, items), { grams: 4200, slots: 4 })
  const mined = gatherResource(withRations, source('ore'), 10_000, resources, items)
  assert.equal(mined.collectedGrams, 5800)
  assert.deepEqual(resourceInventoryUsage(mined.inventory, resources, items), { grams: 10_000, slots: 5 })
  assert.deepEqual(resourceInventoryUsage(removeCountItem(withRations, 'module', 1, resources, items), resources, items), { grams: 2700, slots: 3 })
  assert.throws(() => addCountItem(withRations, 'module', 5, resources, items), /capacity/)
})
test('full slots prevent collecting a new resource type, independent of weight', () => {
  const lightItems: CountItemCatalog = { ration: { gramsPerItem: 1, stackSize: 5 } }
  const occupied = addCountItem(startingResourceInventory(), 'ration', 60, resources, lightItems)
  assert.deepEqual(resourceInventoryUsage(occupied, resources, lightItems), { grams: 60, slots: 12 })
  assert.equal(gatherResource(occupied, source('ore'), 1000, resources, lightItems).collectedGrams, 0)
})
test('debit exact raw mass and independently upgrade weight/slots', () => {
  const full = gatherResource(startingResourceInventory(), source('ice'), 10_000, resources).inventory
  const freed = removeResource(full, 'ice', 4000, resources)
  assert.deepEqual(resourceInventoryUsage(freed, resources), { grams: 6000, slots: 1 })
  assert.equal(full.quantities.ice, 10_000)
  const upgraded = increaseResourceInventoryCapacity(freed, 15_000, 16, resources)
  assert.equal(gatherResource(upgraded, source('ice'), 50_000, resources).collectedGrams, 9000)
  assert.deepEqual(removeResource(freed, 'ice', 6000, resources).quantities, {})
  assert.throws(() => removeResource(freed, 'ice', 7000, resources))
})
test('reject malformed resources, count stacks, overflows and shrinking upgrades', () => {
  assert.throws(() => gatherResource(startingResourceInventory(), source('ore'), 1.5, resources))
  assert.throws(() => gatherResource(startingResourceInventory(), source('ore', -1), 1, resources))
  assert.throws(() => gatherResource(startingResourceInventory(), source('missing'), 1, resources))
  assert.throws(() => removeResource(startingResourceInventory(), 'ore', 0, resources))
  assert.throws(() => resourceInventoryUsage({ ...startingResourceInventory(), quantities: { ore: 0 } }, resources))
  assert.throws(() => resourceInventoryUsage({ ...startingResourceInventory(), quantities: { ore: 11_000 } }, resources))
  assert.throws(() => resourceInventoryUsage({ ...startingResourceInventory(), items: { unknown: 1 } }, resources, items))
  assert.throws(() => addCountItem(startingResourceInventory(), 'module', 1.5, resources, items))
  assert.throws(() => increaseResourceInventoryCapacity(startingResourceInventory(), 9000, 16, resources))
  assert.throws(() => increaseResourceInventoryCapacity(startingResourceInventory(), 20_000, 11, resources))
  assert.throws(() => gatherResource(startingResourceInventory(), source('ice'), Number.MAX_SAFE_INTEGER + 1, resources))
})
test('repeated extraction conserves every gram and partial final capacity', () => {
  let inventory = startingResourceInventory(), deposit = source('ice', 12_333), collected = 0
  for (let i = 0; i < 4; i++) {
    const result = gatherResource(inventory, deposit, 4000, resources)
    inventory = result.inventory
    deposit = result.source
    collected += result.collectedGrams
  }
  assert.equal(collected, 10_000)
  assert.equal(collected + deposit.remainingGrams, 12_333)
})
test('fractional kilogram precision is exact at gram resolution', () => {
  const inv = increaseResourceInventoryCapacity(startingResourceInventory(), 12_333, 12, resources)
  const result = gatherResource(inv, source('ore', 20_000), 20_000, resources)
  assert.equal(result.collectedGrams, 12_333)
  assert.equal(result.source.remainingGrams, 7667)
  assert.deepEqual(resourceInventoryUsage(result.inventory, resources), { grams: 12_333, slots: 1 })
})
