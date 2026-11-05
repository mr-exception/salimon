import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
  STARTING_CAPACITY_KG, STARTING_CAPACITY_LITRES, gatherResource,
  increaseResourceInventoryCapacity, removeResource, resourceInventoryUsage,
  startingResourceInventory, type ResourceCatalog, type ResourceSource,
} from './resourceInventory.ts'

const catalog: ResourceCatalog = {
  ore: { gramsPerLitre: 3000 },
  biomass: { gramsPerLitre: 500 },
  ice: { gramsPerLitre: 1000 },
}
const source = (resourceId: string, remainingLitres = 100): ResourceSource =>
  ({ id: 'source:' + resourceId, resourceId, remainingLitres })

test('starter has 10 kg and 10 L independent limits', () => {
  const inv = startingResourceInventory()
  assert.equal(inv.capacityGrams, STARTING_CAPACITY_KG * 1000)
  assert.equal(inv.capacityLitres, STARTING_CAPACITY_LITRES)
  assert.deepEqual(resourceInventoryUsage(inv, catalog), { grams: 0, litres: 0 })
})
test('mass limits mining; only accepted amount leaves the source', () => {
  const inv = startingResourceInventory()
  const r = gatherResource(inv, source('ore'), 10, catalog)
  assert.equal(r.collectedLitres, 3)
  assert.deepEqual(resourceInventoryUsage(r.inventory, catalog), { grams: 9000, litres: 3 })
  assert.equal(r.source.remainingLitres, 97)
  assert.deepEqual(inv.quantities, {})
  assert.equal(gatherResource(r.inventory, r.source, 10, catalog).collectedLitres, 0)
})
test('volume limits harvesting even when weight is available', () => {
  const r = gatherResource(startingResourceInventory(), source('biomass', 15), 15, catalog)
  assert.equal(r.collectedLitres, 10)
  assert.deepEqual(resourceInventoryUsage(r.inventory, catalog), { grams: 5000, litres: 10 })
  assert.equal(r.source.remainingLitres, 5)
})
test('mixed materials are limited by both dimensions', () => {
  const a = gatherResource(startingResourceInventory(), source('ore'), 2, catalog).inventory
  const b = gatherResource(a, source('biomass'), 9, catalog)
  assert.equal(b.collectedLitres, 8)
  assert.deepEqual(resourceInventoryUsage(b.inventory, catalog), { grams: 10000, litres: 10 })
  assert.equal(gatherResource(b.inventory, source('ice'), 1, catalog).collectedLitres, 0)
})
test('zero requests, exhausted sources and insufficient space do not duplicate materials', () => {
  const inv = startingResourceInventory()
  const r = gatherResource(inv, source('ice', 2), 5, catalog)
  assert.equal(r.collectedLitres, 2)
  assert.equal(r.source.remainingLitres, 0)
  assert.equal(gatherResource(r.inventory, r.source, 5, catalog).collectedLitres, 0)
  assert.equal(gatherResource(inv, source('ice'), 0, catalog).collectedLitres, 0)
})
test('exact debit and upgrade preserve independently validated capacities', () => {
  const full = gatherResource(startingResourceInventory(), source('ice'), 10, catalog).inventory
  const freed = removeResource(full, 'ice', 4, catalog)
  assert.deepEqual(resourceInventoryUsage(freed, catalog), { grams: 6000, litres: 6 })
  assert.equal(full.quantities.ice, 10)
  const upgraded = increaseResourceInventoryCapacity(freed, 15000, 20, catalog)
  assert.equal(gatherResource(upgraded, source('ice'), 50, catalog).collectedLitres, 9)
  assert.deepEqual(removeResource(freed, 'ice', 6, catalog).quantities, {})
  assert.throws(() => removeResource(freed, 'ice', 7, catalog))
})
test('reject invalid, fractional, unknown, zero-weight and overflowing quantities', () => {
  assert.throws(() => gatherResource(startingResourceInventory(), source('ore'), 1.5, catalog))
  assert.throws(() => gatherResource(startingResourceInventory(), source('ore', -1), 1, catalog))
  assert.throws(() => gatherResource(startingResourceInventory(), source('missing'), 1, catalog))
  assert.throws(() => removeResource(startingResourceInventory(), 'ore', 0, catalog))
  assert.throws(() => resourceInventoryUsage({ capacityGrams: 10000, capacityLitres: 10, quantities: { ore: 0 } }, catalog))
  assert.throws(() => resourceInventoryUsage({ capacityGrams: 10000, capacityLitres: 10, quantities: { ore: 4 } }, catalog))
  assert.throws(() => resourceInventoryUsage({ capacityGrams: 10000, capacityLitres: 10, quantities: { ore: 1 } }, { ore: { gramsPerLitre: 0 } }))
  assert.throws(() => increaseResourceInventoryCapacity(startingResourceInventory(), 9000, 20, catalog))
  assert.throws(() => gatherResource(startingResourceInventory(), source('ice'), Number.MAX_SAFE_INTEGER + 1, catalog))
})
test('repeated extraction conserves whole litres', () => {
  let inventory = startingResourceInventory(), deposit = source('ice', 12), collected = 0
  for (let i = 0; i < 4; i++) {
    const r = gatherResource(inventory, deposit, 4, catalog)
    inventory = r.inventory
    deposit = r.source
    collected += r.collectedLitres
  }
  assert.equal(collected, 10)
  assert.equal(collected + deposit.remainingLitres, 12)
})
test('gram precision remains exact at a sub-kilogram density and partial final collection', () => {
  const fractional = { ore: { gramsPerLitre: 333 } }
  const inventory = increaseResourceInventoryCapacity(startingResourceInventory(), 10000, 100, fractional)
  const gathered = gatherResource(inventory, source('ore', 31), 31, fractional)
  assert.equal(gathered.collectedLitres, 30)
  assert.equal(gathered.source.remainingLitres, 1)
  assert.deepEqual(resourceInventoryUsage(gathered.inventory, fractional), { grams: 9990, litres: 30 })
  assert.equal(gatherResource(gathered.inventory, gathered.source, 1, fractional).collectedLitres, 0)
  assert.throws(() => gatherResource(inventory, source('ore'), 1, { ore: { gramsPerLitre: .5 } }))
  assert.throws(() => resourceInventoryUsage({ ...inventory, quantities: { ore: Number.MAX_SAFE_INTEGER } }, fractional))
})
