/** All resource and inventory mass is expressed in integer grams, never volume. */
export interface ResourceDefinition { readonly name?: string }
export type ResourceCatalog = Readonly<Record<string, ResourceDefinition>>
export interface CountItemDefinition { readonly gramsPerItem: number; readonly stackSize: number }
export type CountItemCatalog = Readonly<Record<string, CountItemDefinition>>
export interface ResourceInventory {
  readonly capacityGrams: number
  readonly capacitySlots: number
  /** One slot per resource type; its quantity is integer grams. */
  readonly quantities: Readonly<Record<string, number>>
  /** Counts use as many slots as ceil(count / item stack size). */
  readonly items: Readonly<Record<string, number>>
}
export interface ResourceSource { readonly id: string; readonly resourceId: string; readonly remainingGrams: number }
export interface GatherResult {
  readonly inventory: ResourceInventory
  readonly source: ResourceSource
  readonly collectedGrams: number
}
export const STARTING_CAPACITY_KG = 10
export const STARTING_CAPACITY_SLOTS = 12

function integer(value: number, name: string, min: number): void {
  if (!Number.isSafeInteger(value) || value < min) throw new RangeError(name + ' must be a safe integer >= ' + min)
}
function resource(catalog: ResourceCatalog, id: string): void {
  if (!id || !Object.hasOwn(catalog, id) || !catalog[id]) throw new RangeError('Unknown resource: ' + id)
}
function item(catalog: CountItemCatalog, id: string): CountItemDefinition {
  if (!id || !Object.hasOwn(catalog, id) || !catalog[id]) throw new RangeError('Unknown item: ' + id)
  const value = catalog[id]!
  integer(value.gramsPerItem, 'gramsPerItem', 1)
  integer(value.stackSize, 'stackSize', 1)
  return value
}
function add(a: number, b: number): number {
  const n = a + b
  integer(n, 'total', 0)
  return n
}
function multiply(a: number, b: number): number {
  const n = a * b
  integer(n, 'product', 0)
  return n
}
export function startingResourceInventory(): ResourceInventory {
  return { capacityGrams: STARTING_CAPACITY_KG * 1000, capacitySlots: STARTING_CAPACITY_SLOTS, quantities: {}, items: {} }
}
export function resourceInventoryUsage(
  inv: ResourceInventory, resources: ResourceCatalog, items: CountItemCatalog = {},
): { readonly grams: number; readonly slots: number } {
  integer(inv.capacityGrams, 'capacityGrams', 1)
  integer(inv.capacitySlots, 'capacitySlots', 1)
  let grams = 0, slots = 0
  for (const [id, quantity] of Object.entries(inv.quantities)) {
    resource(resources, id)
    integer(quantity, 'quantityGrams', 1)
    grams = add(grams, quantity)
    slots = add(slots, 1)
  }
  for (const [id, count] of Object.entries(inv.items)) {
    integer(count, 'itemCount', 1)
    const meta = item(items, id)
    grams = add(grams, multiply(count, meta.gramsPerItem))
    slots = add(slots, Math.ceil(count / meta.stackSize))
  }
  if (grams > inv.capacityGrams || slots > inv.capacitySlots) throw new RangeError('Inventory exceeds its capacity')
  return { grams, slots }
}
/** Debits exactly the credited grams; no source depletion on rejected/partial requests beyond credit. */
export function gatherResource(
  inv: ResourceInventory, source: ResourceSource, requestedGrams: number,
  resources: ResourceCatalog, items: CountItemCatalog = {},
): GatherResult {
  if (!source.id) throw new RangeError('Missing source id')
  integer(source.remainingGrams, 'remainingGrams', 0)
  integer(requestedGrams, 'requestedGrams', 0)
  resource(resources, source.resourceId)
  const used = resourceInventoryUsage(inv, resources, items)
  const hasSlot = !!inv.quantities[source.resourceId] || used.slots < inv.capacitySlots
  const collectedGrams = hasSlot ? Math.min(requestedGrams, source.remainingGrams, inv.capacityGrams - used.grams) : 0
  if (!collectedGrams) return { inventory: inv, source, collectedGrams }
  return {
    inventory: { ...inv, quantities: { ...inv.quantities, [source.resourceId]: add(inv.quantities[source.resourceId] ?? 0, collectedGrams) } },
    source: { ...source, remainingGrams: source.remainingGrams - collectedGrams },
    collectedGrams,
  }
}
export function removeResource(inv: ResourceInventory, id: string, grams: number, resources: ResourceCatalog, items: CountItemCatalog = {}): ResourceInventory {
  integer(grams, 'grams', 1)
  resource(resources, id)
  resourceInventoryUsage(inv, resources, items)
  const existing = inv.quantities[id] ?? 0
  if (grams > existing) throw new RangeError('Insufficient resources')
  const quantities = { ...inv.quantities }
  if (grams === existing) delete quantities[id]
  else quantities[id] = existing - grams
  return { ...inv, quantities }
}
/** Count-based items have their own stack limit but share the same inventory mass budget. */
export function addCountItem(inv: ResourceInventory, id: string, count: number, resources: ResourceCatalog, items: CountItemCatalog): ResourceInventory {
  integer(count, 'count', 1)
  item(items, id)
  resourceInventoryUsage(inv, resources, items)
  const next = { ...inv, items: { ...inv.items, [id]: add(inv.items[id] ?? 0, count) } }
  resourceInventoryUsage(next, resources, items)
  return next
}
export function removeCountItem(inv: ResourceInventory, id: string, count: number, resources: ResourceCatalog, items: CountItemCatalog): ResourceInventory {
  integer(count, 'count', 1)
  item(items, id)
  resourceInventoryUsage(inv, resources, items)
  const current = inv.items[id] ?? 0
  if (count > current) throw new RangeError('Insufficient items')
  const after = { ...inv.items }
  if (count === current) delete after[id]
  else after[id] = current - count
  return { ...inv, items: after }
}
export function increaseResourceInventoryCapacity(
  inv: ResourceInventory, capacityGrams: number, capacitySlots: number,
  resources: ResourceCatalog, items: CountItemCatalog = {},
): ResourceInventory {
  resourceInventoryUsage(inv, resources, items)
  integer(capacityGrams, 'capacityGrams', 1)
  integer(capacitySlots, 'capacitySlots', 1)
  if (capacityGrams < inv.capacityGrams || capacitySlots < inv.capacitySlots) throw new RangeError('Upgrade cannot reduce either capacity')
  return { ...inv, capacityGrams, capacitySlots }
}
