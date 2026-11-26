import { validateEquipment, GEAR_DEFINITIONS, type EquipmentState, type GearLocation } from './equipment.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { PLAYER_ID, type TopDownState } from './topDownRuntime.ts'

/** Tier one is preinstalled. Upgrades are deliberately not exposed as commands. */
export const SUIT_DRAWER_CAPACITIES = Object.freeze([4, 8, 12] as const)
export const MODULE_STORAGE_CAPACITIES = Object.freeze([12, 16, 24] as const)
export const SCOUT_GEAR_STORAGE = Object.freeze({
  drawer: Object.freeze({ id: 'scout:suit-drawer', tier: 1, slots: SUIT_DRAWER_CAPACITIES[0], fixtureId: 'scout:storage' }),
  modules: Object.freeze({ id: 'scout:module-storage', tier: 1, slots: MODULE_STORAGE_CAPACITIES[0], fixtureId: 'scout:storage' }),
})
export type GearCommand = {
  readonly kind: 'move'; readonly id: string; readonly from: GearLocation; readonly to: GearLocation
} | {
  readonly kind: 'switch'; readonly id: string; readonly from: GearLocation
  readonly equippedId: string; readonly unequipTo: { readonly kind: 'inventory' | 'drawer' }
}
const sameLocation = (a: GearLocation, b: GearLocation) => a.kind === b?.kind && (a.kind !== 'installed' || b.kind === 'installed' && a.suitId === b.suitId && a.slot === b.slot)
function validDestination(location: GearLocation): void {
  if (!location || !['inventory', 'equipped', 'drawer', 'module-storage', 'installed'].includes(location.kind)) throw Error('Invalid equipment destination')
  const expected = location.kind === 'installed' ? ['kind', 'slot', 'suitId'] : ['kind']
  if (Object.keys(location).sort().join() !== expected.sort().join()) throw Error('Invalid equipment destination fields')
}
/** Explicit all-or-nothing ownership move. Suit switching names the old suit's destination. */
export function applyGearCommand(value: EquipmentState, command: GearCommand): EquipmentState {
  const state = validateEquipment(value)
  if (!command || !['move', 'switch'].includes(command.kind)) throw Error('Unknown equipment command')
  const expected = command.kind === 'move' ? ['kind', 'id', 'from', 'to'] : ['kind', 'id', 'from', 'equippedId', 'unequipTo']
  if (Object.keys(command).sort().join() !== expected.sort().join()) throw Error('Invalid equipment command fields')
  const item = state.instances.find(g => g.id === command.id)
  if (!item || item.location.kind === 'destroyed' || !sameLocation(item.location, command.from)) throw Error('Equipment source changed or unavailable')
  validDestination(command.from)
  if (command.kind === 'move') {
    validDestination(command.to)
    if (sameLocation(command.from, command.to)) throw Error('Equipment already at destination')
    return validateEquipment({ ...state, instances: state.instances.map(g => g.id === item.id ? { ...g, location: command.to } : g) })
  }
  validDestination(command.unequipTo)
  if (!['inventory', 'drawer'].includes(command.unequipTo.kind)) throw Error('Choose the removed suit destination')
  const old = state.instances.find(g => g.location.kind === 'equipped')
  if (!old || old.id !== command.equippedId || old.id === item.id || GEAR_DEFINITIONS[item.definitionId]!.kind !== 'suit' || !['inventory', 'drawer'].includes(item.location.kind)) throw Error('Invalid suit switch')
  return validateEquipment({ ...state, instances: state.instances.map(g => g.id === old.id ? { ...g, location: command.unequipTo } : g.id === item.id ? { ...g, location: { kind: 'equipped' } } : g) })
}
/** Check current physical frame and all affected compartment endpoints immediately before commit. */
export function gearCommandDenial(world: TopDownState, before: EquipmentState, after: EquipmentState): string | null {
  const player = world.entities.find(e => e.id === PLAYER_ID)
  if (!player || player.anchor.frame !== 'ship:scout' || world.cockpit.view !== 'world' || world.player.mode === 'seated' || world.scoutLocation.ship.mode === 'destroyed') return 'Equipment changes require standing inside Scout.'
  const changed = after.instances.filter(g => !sameLocation(g.location, before.instances.find(old => old.id === g.id)!.location))
  const storageOwner = (location: GearLocation, state: EquipmentState): boolean => location.kind === 'drawer' || location.kind === 'module-storage' || location.kind === 'installed' && storageOwner(state.instances.find(g => g.id === location.suitId)!.location, state)
  if (changed.some(g => storageOwner(g.location, after) || storageOwner(before.instances.find(old => old.id === g.id)!.location, before))) {
    const cabinet = world.entities.find(e => e.id === SCOUT_GEAR_STORAGE.drawer.fixtureId)
    if (!cabinet || cabinet.anchor.frame !== 'ship:scout' || !interactionInReach(player.anchor, cabinet, world.frames)) return 'Approach the Scout equipment storage cabinet.'
  }
  return null
}
