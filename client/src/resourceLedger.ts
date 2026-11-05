import { addResource, removeResource, resourceInventoryUsage, type CountItemCatalog, type ResourceCatalog, type ResourceInventory, type ResourceSource } from './resourceInventory.ts'

/** Trusted declarations come from future ship composition, never saved bytes. */
export interface ResourceStoreDefinition { readonly shipId: string; readonly capacityGrams: number }
export type ResourceStoreCatalog = Readonly<Record<string, ResourceStoreDefinition>>
export interface ResourceStore extends ResourceStoreDefinition {
  readonly id: string
  readonly quantities: Readonly<Record<string, number>>
}
export interface ResourceLedger {
  readonly revision: number
  readonly inventory: ResourceInventory
  readonly sources: Readonly<Record<string, ResourceSource>>
  readonly stores: Readonly<Record<string, ResourceStore>>
}
export type ResourceEndpoint = { readonly kind: 'player' } | { readonly kind: 'source' | 'store'; readonly id: string }
export interface ResourceTransfer {
  readonly revision: number
  readonly from: ResourceEndpoint
  readonly to: ResourceEndpoint
  readonly resourceId: string
  readonly grams: number
}
function integer(n: number, minimum = 0): number {
  if (!Number.isSafeInteger(n) || n < minimum) throw new RangeError('Invalid ledger integer')
  return n
}
function sum(a: number, b: number): number { return integer(a + b) }
function record(value: unknown): asserts value is Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new RangeError('Invalid ledger record')
}
function fields(value: object, names: readonly string[]): void {
  if (Object.keys(value).some(key => !names.includes(key))) throw new RangeError('Unknown ledger metadata')
}
function sameKeys(actual: object, expected: object): void {
  record(actual)
  if (Object.keys(actual).length !== Object.keys(expected).length || Object.keys(actual).some(id => !Object.hasOwn(expected, id))) throw new RangeError('Unknown or missing ledger ID')
}
export function emptyResourceStores(definitions: ResourceStoreCatalog): Record<string, ResourceStore> {
  return Object.fromEntries(Object.entries(definitions).map(([id, meta]) => [id, { ...meta, id, quantities: {} }]))
}
/** Raw-only stores have a declared gram cap, no invented slot or volume limit. */
export function resourceStoreMass(store: ResourceStore, resources: ResourceCatalog): number {
  record(store); record(store.quantities)
  fields(store, ['id', 'shipId', 'capacityGrams', 'quantities'])
  integer(store.capacityGrams, 1)
  let grams = 0
  for (const [id, quantity] of Object.entries(store.quantities)) {
    if (!Object.hasOwn(resources, id) || !resources[id]) throw new RangeError('Unknown resource')
    grams = sum(grams, integer(quantity, 1))
  }
  if (grams > store.capacityGrams) throw new RangeError('Store exceeds capacity')
  return grams
}
/** Validate the entire graph, including material identity and exact initial mass. */
export function validateResourceLedger(
  ledger: ResourceLedger, sources: Readonly<Record<string, ResourceSource>>, stores: ResourceStoreCatalog,
  resources: ResourceCatalog, items: CountItemCatalog = {},
): void {
  integer(ledger.revision)
  resourceInventoryUsage(ledger.inventory, resources, items)
  sameKeys(ledger.sources, sources); sameKeys(ledger.stores, stores)
  const totals = new Map<string, number>(), initial = new Map<string, number>()
  const credit = (map: Map<string, number>, id: string, grams: number) => map.set(id, sum(map.get(id) ?? 0, grams))
  const ids = new Set(['player'])
  for (const [id, expected] of Object.entries(sources)) {
    if (!id || ids.has(id) || expected.id !== id || !Object.hasOwn(resources, expected.resourceId)) throw new RangeError('Invalid source identity')
    ids.add(id); integer(expected.remainingGrams)
    const actual = ledger.sources[id]!
    record(actual)
    if (actual.id !== id || actual.resourceId !== expected.resourceId) throw new RangeError('Changed source material identity')
    fields(actual, ['id', 'resourceId', 'remainingGrams'])
    integer(actual.remainingGrams)
    if (actual.remainingGrams > expected.remainingGrams) throw new RangeError('Source exceeds initial mass')
    credit(initial, expected.resourceId, expected.remainingGrams)
    credit(totals, actual.resourceId, actual.remainingGrams)
  }
  for (const [id, expected] of Object.entries(stores)) {
    if (!id || ids.has(id) || !expected.shipId) throw new RangeError('Invalid store identity')
    ids.add(id); integer(expected.capacityGrams, 1)
    const actual = ledger.stores[id]!
    record(actual)
    if (actual.id !== id || actual.shipId !== expected.shipId || actual.capacityGrams !== expected.capacityGrams) throw new RangeError('Changed store metadata')
    resourceStoreMass(actual, resources)
    for (const [material, grams] of Object.entries(actual.quantities)) credit(totals, material, grams)
  }
  for (const [id, grams] of Object.entries(ledger.inventory.quantities)) credit(totals, id, grams)
  for (const id of new Set([...initial.keys(), ...totals.keys()])) {
    if ((totals.get(id) ?? 0) !== (initial.get(id) ?? 0)) throw new RangeError('Material conservation failed')
  }
}
/** Exact all-or-nothing transaction. No mutation, partial transfer or source refill. */
export function transferResource(
  ledger: ResourceLedger, request: ResourceTransfer, sources: Readonly<Record<string, ResourceSource>>,
  stores: ResourceStoreCatalog, resources: ResourceCatalog, items: CountItemCatalog = {},
): ResourceLedger {
  validateResourceLedger(ledger, sources, stores, resources, items)
  integer(request.grams, 1); integer(request.revision)
  if (request.revision !== ledger.revision) throw new RangeError('Stale ledger revision')
  if (!Object.hasOwn(resources, request.resourceId)) throw new RangeError('Unknown resource')
  const endpoint = (value: ResourceEndpoint): string => {
    if (value.kind === 'player') return 'player'
    if (value.kind === 'source' && Object.hasOwn(sources, value.id)) return value.id
    if (value.kind === 'store' && Object.hasOwn(stores, value.id)) return value.id
    throw new RangeError('Unknown endpoint ID')
  }
  const from = endpoint(request.from), to = endpoint(request.to)
  if (from === to || request.to.kind === 'source') throw new RangeError('Invalid transfer route')
  const id = request.resourceId, grams = request.grams
  // Validate the destination before constructing any source debit.
  let inventory = ledger.inventory, nextStores = ledger.stores, nextSources = ledger.sources
  if (request.to.kind === 'player') inventory = addResource(inventory, id, grams, resources, items)
  else {
    const destination = ledger.stores[to]!
    const next = { ...destination, quantities: { ...destination.quantities, [id]: sum(destination.quantities[id] ?? 0, grams) } }
    resourceStoreMass(next, resources)
    nextStores = { ...nextStores, [to]: next }
  }
  if (request.from.kind === 'player') inventory = removeResource(inventory, id, grams, resources, items)
  else if (request.from.kind === 'store') {
    const origin = ledger.stores[from]!, current = origin.quantities[id] ?? 0
    if (grams > current) throw new RangeError('Insufficient store resources')
    const quantities = { ...origin.quantities }
    if (grams === current) delete quantities[id]
    else quantities[id] = current - grams
    nextStores = { ...nextStores, [from]: { ...origin, quantities } }
  } else {
    const origin = ledger.sources[from]!
    if (origin.resourceId !== id || grams > origin.remainingGrams) throw new RangeError('Insufficient or mismatched source material')
    nextSources = { ...nextSources, [from]: { ...origin, remainingGrams: origin.remainingGrams - grams } }
  }
  const next = { revision: integer(ledger.revision + 1), inventory, stores: nextStores, sources: nextSources }
  validateResourceLedger(next, sources, stores, resources, items)
  return next
}
/** Stores are flat raw inventories. Player cargo is never nested in a store. */
export function resourceLedgerMass(ledger: ResourceLedger, shipId: string, playerAboard: boolean, resources: ResourceCatalog, items: CountItemCatalog = {}) {
  const playerGrams = resourceInventoryUsage(ledger.inventory, resources, items).grams
  let storeGrams = 0
  for (const store of Object.values(ledger.stores)) {
    const grams = resourceStoreMass(store, resources)
    if (store.shipId === shipId) storeGrams = sum(storeGrams, grams)
  }
  return { playerGrams, storeGrams, shipGrams: sum(storeGrams, playerAboard ? playerGrams : 0) }
}
