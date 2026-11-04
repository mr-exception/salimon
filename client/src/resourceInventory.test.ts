import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  STARTING_CAPACITY_KG, STARTING_CAPACITY_LITRES, gatherResource,
  increaseResourceInventoryCapacity, removeResource, resourceInventoryUsage,
  startingResourceInventory, type ResourceCatalog, type ResourceSource,
} from './resourceInventory.ts'

const catalog: ResourceCatalog = {
  ore: { kgPerUnit: 3, litresPerUnit: 1 },
  biomass: { kgPerUnit: 1, litresPerUnit: 4 },
  ice: { kgPerUnit: 1, litresPerUnit: 1 },
}
const source = (resourceId: string, remainingUnits = 100): ResourceSource =>
  ({ id: 'source:' + resourceId, resourceId, remainingUnits })

test('starter has 10 kg and 10 L independent limits', () => {
  const inv = startingResourceInventory()
  assert.equal(inv.capacityKg, STARTING_CAPACITY_KG)
  assert.equal(inv.capacityLitres, STARTING_CAPACITY_LITRES)
  assert.deepEqual(resourceInventoryUsage(inv, catalog), { kg: 0, litres: 0 })
})
test('mass limits mining; only accepted amount leaves the source', () => {
  const inv = startingResourceInventory()
  const r = gatherResource(inv, source('ore'), 10, catalog)
  assert.equal(r.collectedUnits, 3)
  assert.deepEqual(resourceInventoryUsage(r.inventory, catalog), { kg: 9, litres: 3 })
  assert.equal(r.source.remainingUnits, 97)
  assert.deepEqual(inv.quantities, {})
  assert.equal(gatherResource(r.inventory, r.source, 10, catalog).collectedUnits, 0)
})
test('volume limits harvesting even when weight is available', () => {
  const r = gatherResource(startingResourceInventory(), source('biomass', 8), 8, catalog)
  assert.equal(r.collectedUnits, 2)
  assert.deepEqual(resourceInventoryUsage(r.inventory, catalog), { kg: 2, litres: 8 })
  assert.equal(r.source.remainingUnits, 6)
})
test('mixed materials are limited by both dimensions', () => {
  const a = gatherResource(startingResourceInventory(), source('ore'), 2, catalog).inventory
  const b = gatherResource(a, source('biomass'), 9, catalog)
  assert.equal(b.collectedUnits, 2)
  assert.deepEqual(resourceInventoryUsage(b.inventory, catalog), { kg: 8, litres: 10 })
  assert.equal(gatherResource(b.inventory, source('ice'), 1, catalog).collectedUnits, 0)
})
test('zero requests, exhausted sources and insufficient space do not duplicate materials', () => {
  const inv = startingResourceInventory()
  const r = gatherResource(inv, source('ice', 2), 5, catalog)
  assert.equal(r.collectedUnits, 2)
  assert.equal(r.source.remainingUnits, 0)
  assert.equal(gatherResource(r.inventory, r.source, 5, catalog).collectedUnits, 0)
  assert.equal(gatherResource(inv, source('ice'), 0, catalog).collectedUnits, 0)
})
test('exact debit and upgrade preserve independently validated capacities', () => {
  const full = gatherResource(startingResourceInventory(), source('ice'), 10, catalog).inventory
  const freed = removeResource(full, 'ice', 4, catalog)
  assert.deepEqual(resourceInventoryUsage(freed, catalog), { kg: 6, litres: 6 })
  assert.equal(full.quantities.ice, 10)
  const upgraded = increaseResourceInventoryCapacity(freed, 15, 20, catalog)
  assert.equal(gatherResource(upgraded, source('ice'), 50, catalog).collectedUnits, 9)
  assert.deepEqual(removeResource(freed, 'ice', 6, catalog).quantities, {})
  assert.throws(() => removeResource(freed, 'ice', 7, catalog))
})
test('reject invalid, fractional, unknown, zero-weight and overflowing quantities', () => {
  assert.throws(() => gatherResource(startingResourceInventory(), source('ore'), 1.5, catalog))
  assert.throws(() => gatherResource(startingResourceInventory(), source('ore', -1), 1, catalog))
  assert.throws(() => gatherResource(startingResourceInventory(), source('missing'), 1, catalog))
  assert.throws(() => removeResource(startingResourceInventory(), 'ore', 0, catalog))
  assert.throws(() => resourceInventoryUsage({ capacityKg: 10, capacityLitres: 10, quantities: { ore: 0 } }, catalog))
  assert.throws(() => resourceInventoryUsage({ capacityKg: 10, capacityLitres: 10, quantities: { ore: 4 } }, catalog))
  assert.throws(() => resourceInventoryUsage({ capacityKg: 10, capacityLitres: 10, quantities: { ore: 1 } }, { ore: { kgPerUnit: 0, litresPerUnit: 1 } }))
  assert.throws(() => increaseResourceInventoryCapacity(startingResourceInventory(), 9, 20, catalog))
  assert.throws(() => gatherResource(startingResourceInventory(), source('ice'), Number.MAX_SAFE_INTEGER + 1, catalog))
})
test('repeated extraction conserves whole units', () => {
  let inventory = startingResourceInventory(), deposit = source('ice', 12), collected = 0
  for (let i = 0; i < 4; i++) {
    const r = gatherResource(inventory, deposit, 4, catalog)
    inventory = r.inventory
    deposit = r.source
    collected += r.collectedUnits
  }
  assert.equal(collected, 10)
  assert.equal(collected + deposit.remainingUnits, 12)
})
