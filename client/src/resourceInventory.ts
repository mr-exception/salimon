/** Whole-unit resources for future 2D exploration; not yet wired to navigation UI. */
export interface ResourceDefinition {
  readonly kgPerUnit: number
  readonly litresPerUnit: number
}
export type ResourceCatalog = Readonly<Record<string, ResourceDefinition>>
export interface ResourceInventory {
  readonly capacityKg: number
  readonly capacityLitres: number
  readonly quantities: Readonly<Record<string, number>>
}
export interface ResourceSource {
  readonly id: string
  readonly resourceId: string
  readonly remainingUnits: number
}
export interface GatherResult {
  readonly inventory: ResourceInventory
  readonly source: ResourceSource
  readonly collectedUnits: number
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
  integer(entry.kgPerUnit, 'kgPerUnit', 1)
  integer(entry.litresPerUnit, 'litresPerUnit', 1)
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
  return { capacityKg: STARTING_CAPACITY_KG, capacityLitres: STARTING_CAPACITY_LITRES, quantities: {} }
}
export function resourceInventoryUsage(inventory: ResourceInventory, catalog: ResourceCatalog): {
  readonly kg: number; readonly litres: number
} {
  integer(inventory.capacityKg, 'capacityKg', 1)
  integer(inventory.capacityLitres, 'capacityLitres', 1)
  let kg = 0
  let litres = 0
  for (const [id, quantity] of Object.entries(inventory.quantities)) {
    integer(quantity, 'quantity', 1)
    const entry = definition(catalog, id)
    kg = safeAdd(kg, safeMultiply(quantity, entry.kgPerUnit, 'kg'), 'kg')
    litres = safeAdd(litres, safeMultiply(quantity, entry.litresPerUnit, 'litres'), 'litres')
  }
  if (kg > inventory.capacityKg || litres > inventory.capacityLitres) {
    throw new RangeError('Inventory exceeds its capacity')
  }
  return { kg, litres }
}
/** Credit only whole units that fit in BOTH limits, debiting a persistent source by the same amount. */
export function gatherResource(
  inventory: ResourceInventory,
  source: ResourceSource,
  requestedUnits: number,
  catalog: ResourceCatalog,
): GatherResult {
  if (!source.id) throw new RangeError('Missing source id')
  integer(source.remainingUnits, 'remainingUnits', 0)
  integer(requestedUnits, 'requestedUnits', 0)
  const entry = definition(catalog, source.resourceId)
  const used = resourceInventoryUsage(inventory, catalog)
  const allowedByKg = Math.floor((inventory.capacityKg - used.kg) / entry.kgPerUnit)
  const allowedByLitres = Math.floor((inventory.capacityLitres - used.litres) / entry.litresPerUnit)
  const collectedUnits = Math.min(requestedUnits, source.remainingUnits, allowedByKg, allowedByLitres)
  if (collectedUnits === 0) return { inventory, source, collectedUnits }
  const total = safeAdd(inventory.quantities[source.resourceId] ?? 0, collectedUnits, 'quantity')
  return {
    inventory: { ...inventory, quantities: { ...inventory.quantities, [source.resourceId]: total } },
    source: { ...source, remainingUnits: source.remainingUnits - collectedUnits },
    collectedUnits,
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
  inventory: ResourceInventory, capacityKg: number, capacityLitres: number, catalog: ResourceCatalog,
): ResourceInventory {
  resourceInventoryUsage(inventory, catalog)
  integer(capacityKg, 'capacityKg', 1)
  integer(capacityLitres, 'capacityLitres', 1)
  if (capacityKg < inventory.capacityKg || capacityLitres < inventory.capacityLitres) {
    throw new RangeError('Upgrade cannot reduce either capacity')
  }
  return { ...inventory, capacityKg, capacityLitres }
}
