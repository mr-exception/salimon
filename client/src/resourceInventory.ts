/** Exact whole-litre resources. All mass arithmetic uses integer grams. */
export interface ResourceDefinition {
  readonly gramsPerLitre: number
}
export type ResourceCatalog = Readonly<Record<string, ResourceDefinition>>
export interface ResourceInventory {
  readonly capacityGrams: number
  readonly capacityLitres: number
  readonly quantities: Readonly<Record<string, number>>
}
export interface ResourceSource {
  readonly id: string
  readonly resourceId: string
  readonly remainingLitres: number
}
export interface GatherResult {
  readonly inventory: ResourceInventory
  readonly source: ResourceSource
  readonly collectedLitres: number
}
export const STARTING_CAPACITY_KG = 10
export const STARTING_CAPACITY_LITRES = 10

function integer(value: number, name: string, minimum: number): void {
  if (!Number.isSafeInteger(value) || value < minimum) {
    throw new RangeError(name + ' must be a safe integer >= ' + minimum)
  }
}
function definition(catalog: ResourceCatalog, id: string): ResourceDefinition {
  if (!id || !Object.prototype.hasOwnProperty.call(catalog, id)) {
    throw new RangeError('Unknown resource: ' + id)
  }
  const entry = catalog[id]
  if (!entry) throw new RangeError('Missing definition: ' + id)
  integer(entry.gramsPerLitre, 'gramsPerLitre', 1)
  return entry
}
function safeAdd(a: number, b: number, label: string): number {
  const result = a + b
  integer(result, label, 0)
  return result
}
function safeMultiply(a: number, b: number, label: string): number {
  const result = a * b
  integer(result, label, 0)
  return result
}
export function startingResourceInventory(): ResourceInventory {
  return { capacityGrams: STARTING_CAPACITY_KG * 1000, capacityLitres: STARTING_CAPACITY_LITRES, quantities: {} }
}
export function resourceInventoryUsage(inventory: ResourceInventory, catalog: ResourceCatalog): {
  readonly grams: number; readonly litres: number
} {
  integer(inventory.capacityGrams, 'capacityGrams', 1)
  integer(inventory.capacityLitres, 'capacityLitres', 1)
  let grams = 0
  let litres = 0
  for (const [id, quantity] of Object.entries(inventory.quantities)) {
    integer(quantity, 'quantity', 1)
    const entry = definition(catalog, id)
    grams = safeAdd(grams, safeMultiply(quantity, entry.gramsPerLitre, 'grams'), 'grams')
    litres = safeAdd(litres, quantity, 'litres')
  }
  if (grams > inventory.capacityGrams || litres > inventory.capacityLitres) {
    throw new RangeError('Inventory exceeds its capacity')
  }
  return { grams, litres }
}
/** Credit only whole units that fit in BOTH limits, debiting a persistent source by the same amount. */
export function gatherResource(
  inventory: ResourceInventory,
  source: ResourceSource,
  requestedLitres: number,
  catalog: ResourceCatalog,
): GatherResult {
  if (!source.id) throw new RangeError('Missing source id')
  integer(source.remainingLitres, 'remainingLitres', 0)
  integer(requestedLitres, 'requestedLitres', 0)
  const entry = definition(catalog, source.resourceId)
  const used = resourceInventoryUsage(inventory, catalog)
  const allowedByMass = Math.floor((inventory.capacityGrams - used.grams) / entry.gramsPerLitre)
  const allowedByLitres = inventory.capacityLitres - used.litres
  const collectedLitres = Math.min(requestedLitres, source.remainingLitres, allowedByMass, allowedByLitres)
  if (collectedLitres === 0) return { inventory, source, collectedLitres }
  const total = safeAdd(inventory.quantities[source.resourceId] ?? 0, collectedLitres, 'quantity')
  return {
    inventory: { ...inventory, quantities: { ...inventory.quantities, [source.resourceId]: total } },
    source: { ...source, remainingLitres: source.remainingLitres - collectedLitres },
    collectedLitres,
  }
}
/** Exact spending/transfer debit, with no partial success. */
export function removeResource(
  inventory: ResourceInventory, resourceId: string, units: number, catalog: ResourceCatalog,
): ResourceInventory {
  integer(units, 'units', 1)
  definition(catalog, resourceId)
  resourceInventoryUsage(inventory, catalog)
  const current = inventory.quantities[resourceId] ?? 0
  if (units > current) throw new RangeError('Insufficient resources')
  const quantities = { ...inventory.quantities }
  if (current === units) delete quantities[resourceId]
  else quantities[resourceId] = current - units
  return { ...inventory, quantities }
}
/** Future upgrades can expand either capacity independently. */
export function increaseResourceInventoryCapacity(
  inventory: ResourceInventory, capacityGrams: number, capacityLitres: number, catalog: ResourceCatalog,
): ResourceInventory {
  resourceInventoryUsage(inventory, catalog)
  integer(capacityGrams, 'capacityGrams', 1)
  integer(capacityLitres, 'capacityLitres', 1)
  if (capacityGrams < inventory.capacityGrams || capacityLitres < inventory.capacityLitres) {
    throw new RangeError('Upgrade cannot reduce either capacity')
  }
  return { ...inventory, capacityGrams, capacityLitres }
}
