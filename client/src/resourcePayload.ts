import { equipmentMass } from './equipment.ts'
import { type CountItemCatalog, resourceInventoryUsage, type ResourceInventory } from './resourceInventory.ts'
import { DRY_MASS_KG, setMass, type ShipState } from './simulation.ts'
import { PLAYER_ID, type TopDownState } from './topDownRuntime.ts'
import { RESOURCE_CATALOG, ITEM_CATALOG } from './resourceCatalog.ts'
export function resourcePayload(inventory: ResourceInventory, world: TopDownState, items: CountItemCatalog = ITEM_CATALOG) {
  const grams = resourceInventoryUsage(inventory, RESOURCE_CATALOG, items).grams
  const gear = inventory.equipment ? equipmentMass(inventory.equipment) : null
  return { playerGrams: grams, shipGrams: (gear?.storedGrams ?? 0) + (world.entities.find(e => e.id === PLAYER_ID)!.anchor.frame === world.frames.ship.id ? grams + (gear?.equippedGrams ?? 0) : 0) }
}

export function withResourcePayload(ship: ShipState, inventory: ResourceInventory, world: TopDownState, items: CountItemCatalog = ITEM_CATALOG): ShipState {
  // Recompute from dry mass, never add the same nested player payload per tick.
  return setMass(ship, DRY_MASS_KG + resourcePayload(inventory, world, items).shipGrams / 1000)
}
