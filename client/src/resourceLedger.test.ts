import { test } from 'node:test'
import assert from 'node:assert/strict'
import { startingResourceInventory, type ResourceSource } from './resourceInventory.ts'
import { emptyResourceStores, resourceLedgerMass, transferResource, validateResourceLedger, type ResourceEndpoint, type ResourceLedger, type ResourceTransfer } from './resourceLedger.ts'
const resources = { ore: {}, ice: {} }, items = { module: { gramsPerItem: 17, stackSize: 1 } }
const sources = { rock: { id: 'rock', resourceId: 'ore', remainingGrams: 20_333 }, frost: { id: 'frost', resourceId: 'ice', remainingGrams: 500 } }
// Deliberately synthetic test capacities; no production container is enabled.
const stores = { cargo: { shipId: 'ship:a', capacityGrams: 12_333 }, buffer: { shipId: 'ship:a', capacityGrams: 2000 }, remote: { shipId: 'ship:b', capacityGrams: 1000 } }
const player: ResourceEndpoint = { kind: 'player' }
const store = (id: string): ResourceEndpoint => ({ kind: 'store', id })
const source = (id: string): ResourceEndpoint => ({ kind: 'source', id })
const initial = (): ResourceLedger => ({ revision: 0, inventory: startingResourceInventory(), sources, stores: emptyResourceStores(stores) })
const move = (state: ResourceLedger, from: ResourceEndpoint, to: ResourceEndpoint, grams: number, resourceId = 'ore') => transferResource(state, { revision: state.revision, from, to, grams, resourceId }, sources, stores, resources, items)
const immutable = <T>(value: T): T => { if (value && typeof value === 'object') { Object.freeze(value); Object.values(value).forEach(immutable) } return value }

test('exact raw add, removal and all supported routes conserve grams without mutating inputs', () => {
  const start = immutable(initial())
  let state = move(start, source('rock'), player, 2333)
  state = move(state, player, store('cargo'), 2000)
  state = move(state, store('cargo'), store('buffer'), 1999)
  state = move(state, store('buffer'), player, 1999)
  state = move(state, source('frost'), store('remote'), 500, 'ice')
  assert.equal(state.revision, 5)
  assert.equal(state.inventory.quantities.ore, 2332)
  assert.equal(state.stores.cargo!.quantities.ore, 1)
  assert.deepEqual(state.stores.buffer!.quantities, {})
  assert.equal(state.stores.remote!.quantities.ice, 500)
  assert.equal(state.sources.rock!.remainingGrams, 18000)
  assert.deepEqual(start, initial())
  validateResourceLedger(state, sources, stores, resources, items)
})
test('all gram amounts around capacities and available mass succeed exactly or leave inputs untouched', () => {
  for (const destination of [player, store('buffer')]) {
    const cap = destination.kind === 'player' ? 10_000 : 2000
    for (const grams of [1, 333, cap - 1, cap, cap + 1, 20_334]) {
      const before = initial(), copy = structuredClone(before)
      if (grams <= cap) {
        const result = move(before, source('rock'), destination, grams)
        assert.equal(result.sources.rock!.remainingGrams, sources.rock.remainingGrams - grams)
      } else assert.throws(() => move(before, source('rock'), destination, grams))
      assert.deepEqual(before, copy)
    }
  }
})
test('shared count-item slots and mass constrain raw transfers while existing raw stacks reuse a slot', () => {
  const state = initial()
  const occupied = { ...state, inventory: { ...state.inventory, items: { module: 12 } } }
  assert.throws(() => move(occupied, source('rock'), player, 1), /capacity/)
  const weighted = { ...state, inventory: { ...state.inventory, items: { module: 1 } } }
  assert.throws(() => move(weighted, source('rock'), player, 9984), /capacity/)
  let stacked = move(state, source('rock'), player, 1)
  stacked = { ...stacked, inventory: { ...stacked.inventory, items: { module: 11 } } }
  assert.equal(move(stacked, source('rock'), player, 333).inventory.quantities.ore, 334)
})
test('invalid IDs, routes, values, source identity, revision and overflows never publish changes', () => {
  const state = immutable(move(initial(), source('rock'), store('cargo'), 1000))
  const valid: ResourceTransfer = { revision: state.revision, from: store('cargo'), to: player, resourceId: 'ore', grams: 1 }
  const bad: ResourceTransfer[] = [
    ...[0, -1, 0.5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1, 1001].map(grams => ({ ...valid, grams })),
    { ...valid, revision: 0 }, { ...valid, revision: 0.5 }, { ...valid, resourceId: 'unknown' },
    { ...valid, from: store('missing') }, { ...valid, to: store('missing') },
    { ...valid, from: source('missing') }, { ...valid, to: source('rock') },
    { ...valid, to: store('cargo') }, { ...valid, from: source('frost') },
  ]
  for (const request of bad) assert.throws(() => transferResource(state, request, sources, stores, resources, items))
  const overflow = { ...state, revision: Number.MAX_SAFE_INTEGER }
  assert.throws(() => move(overflow, store('cargo'), player, 1))
  for (const mutate of [
    (s: any) => { s.sources.rock.resourceId = 'ice' },
    (s: any) => { s.sources.rock.id = 'other' },
    (s: any) => { s.sources.rock.remainingGrams++ },
    (s: any) => { s.sources.extra = s.sources.rock },
    (s: any) => { s.stores.cargo.shipId = 'ship:b' },
    (s: any) => { s.stores.cargo.capacityGrams++ },
    (s: any) => { s.stores.extra = s.stores.cargo },
    (s: any) => { s.stores.cargo.quantities.ore = -1 },
    (s: any) => { s.stores.cargo.quantities.unknown = 1 },
    (s: any) => { delete s.stores.buffer },
  ]) { const copy = structuredClone(state); mutate(copy); assert.throws(() => move(copy, store('cargo'), player, 1)) }
  const huge: Record<string, ResourceSource> = { a: { id: 'a', resourceId: 'ore', remainingGrams: Number.MAX_SAFE_INTEGER }, b: { id: 'b', resourceId: 'ore', remainingGrams: 1 } }
  assert.throws(() => validateResourceLedger({ ...initial(), sources: huge }, huge, stores, resources))
})
test('flat ledger mass counts only the selected ship stores and onboard player once', () => {
  let state = move(initial(), source('rock'), player, 333)
  state = move(state, source('rock'), store('cargo'), 1233)
  state = move(state, source('frost'), store('remote'), 500, 'ice')
  assert.deepEqual(resourceLedgerMass(state, 'ship:a', false, resources), { playerGrams: 333, storeGrams: 1233, shipGrams: 1233 })
  for (let i = 0; i < 300; i++) assert.deepEqual(resourceLedgerMass(state, 'ship:a', true, resources), { playerGrams: 333, storeGrams: 1233, shipGrams: 1566 })
  assert.equal(resourceLedgerMass(state, 'ship:b', false, resources).shipGrams, 500)
})
test('exhaustive small transfer sequences conserve each material through mixed destinations', () => {
  const endpoints = [player, store('cargo'), store('buffer')]
  let state = move(initial(), source('rock'), player, 1000)
  for (let i = 0; i < 500; i++) {
    const from = endpoints[i % 3]!, to = endpoints[(i + 1) % 3]!
    try { state = move(state, from, to, (i % 37) + 1) } catch { /* insufficient source: state stays valid */ }
    validateResourceLedger(state, sources, stores, resources, items)
    assert.equal(resourceLedgerMass(state, 'ship:a', true, resources).shipGrams, 1000)
  }
})
