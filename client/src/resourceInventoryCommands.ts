import { applyGearCommand, gearCommandDenial, type GearCommand } from './scoutEquipment.ts'
import { freshEquipment } from './equipment.ts'
import { MARKER_KIT_ID } from './scoutFabricator.ts'
import { freshHazards, validateHazards, type ProtectionModule } from './hazards.ts'
import { freshSurvival, survivalPolicy } from './survival.ts'
import { transferResource, type ResourceStoreCatalog, type ResourceTransfer } from './resourceLedger.ts'
import { addCountItem, removeCountItem, increaseResourceInventoryCapacity, resourceInventoryUsage } from './resourceInventory.ts'
import { RESOURCE_CATALOG, type ResourceSave } from './resourceCatalog.ts'
import { sourceDefinitions } from './resourceSaveLocation.ts'
import type { ResourceSession } from './resourceSession.ts'
type SessionState = Pick<ResourceSession, 'equipped' | 'error' | 'feedback' | 'hazards' | 'itemCatalog' | 'snapshot' | 'survival' | 'world'>
type CommitResourceSave = (next: ResourceSave) => boolean

export function changeEquipment(session: SessionState, commit: CommitResourceSave, command: GearCommand, revision: number): boolean {
  if (session.error || !survivalPolicy(session.survival).canWork) return false
  try {
    if (!Number.isSafeInteger(revision) || revision !== session.snapshot.revision) throw Error('Stale equipment revision')
    const before = session.snapshot.inventory.equipment!
    const equipment = applyGearCommand(before, command)
    const denial = gearCommandDenial(session.world, before, equipment)
    if (denial) throw Error(denial)
    const inventory = { ...session.snapshot.inventory, equipment }
    resourceInventoryUsage(inventory, RESOURCE_CATALOG, session.itemCatalog)
    return commit({ ...session.snapshot, inventory, survival: session.survival, hazards: session.hazards })
  } catch (error) { session.feedback = (error as Error).message; return false }
}

export function addProtection(session: SessionState, commit: CommitResourceSave, module: Omit<ProtectionModule, 'remaining'>): boolean {
  if (session.error || session.survival.health === 0 || session.hazards.modules.some(m => m.id === module.id)) return false
  const hazards = validateHazards({ ...session.hazards, modules: [...session.hazards.modules, { ...module, remaining: '1000/1' }] })
  return commit({ ...session.snapshot, survival: session.survival, hazards })
}

export function transfer(session: SessionState, commit: CommitResourceSave, storeDefinitions: ResourceStoreCatalog, request: ResourceTransfer): boolean {
  if (session.error || session.survival.health === 0) return false
  try {
    const sourceId = request.from.kind === 'source' ? request.from.id : null
    if (sourceId !== null && (!session.world.scoutLocation.binding || !session.world.entities.some(e => e.id === sourceId && e.source))) throw Error('Source is unavailable at this surface site.')
    const next = transferResource(session.snapshot, request, sourceDefinitions(session.snapshot.sources), storeDefinitions, RESOURCE_CATALOG, session.itemCatalog)
    return commit({ ...session.snapshot, ...next, hazards: session.hazards, survival: session.survival })
  } catch (error) {
    session.feedback = error instanceof Error ? error.message : 'Invalid transfer'
    return false
  }
}

export function changeItem(session: SessionState, commit: CommitResourceSave, action: 'add' | 'remove', id: string, count: number, revision: number): boolean {
  if (session.error || session.survival.health === 0) return false
  try {
    if (!Number.isSafeInteger(revision) || revision !== session.snapshot.revision) throw new RangeError('Stale item revision')
    if (id === MARKER_KIT_ID) throw Error('Manufactured items require a Fabricator transaction')
    if (action !== 'add' && action !== 'remove') throw new RangeError('Unknown item action')
    const inventory = (action === 'add' ? addCountItem : removeCountItem)(session.snapshot.inventory, id, count, RESOURCE_CATALOG, session.itemCatalog)
    const saved = commit({ ...session.snapshot, hazards: session.hazards, survival: session.survival, inventory })
    if (saved) session.feedback = `Saved: ${action === 'add' ? 'added' : 'removed'} ${count} × ${session.itemCatalog[id]!.name ?? id}.`
    return saved
  } catch (error) {
    session.feedback = error instanceof Error ? error.message : 'Invalid item transaction'
    return false
  }
}

export function freshRun(session: SessionState, commit: CommitResourceSave, now: () => number): boolean {
  if (session.error || session.survival.health !== 0) return false
  const next = freshSurvival(now())
  if (!commit({ ...session.snapshot, inventory: { ...session.snapshot.inventory, equipment: { ...freshEquipment(session.snapshot.inventory.equipment!.generation + 1), instances: [...freshEquipment(session.snapshot.inventory.equipment!.generation + 1).instances, ...session.snapshot.inventory.equipment!.instances.filter(g=>g.location.kind==='bench'||g.location.kind==='installed'&&session.snapshot.inventory.equipment!.instances.some(s=>s.id===(g.location as {suitId:string}).suitId&&s.location.kind==='bench'))] } }, hazards: { ...session.hazards, exposure: freshHazards().exposure }, survival: next })) return false
  session.survival = next
  session.equipped = false
  return true
}

export function upgrade(session: SessionState, commit: CommitResourceSave, capacityGrams: number, capacitySlots: number): boolean {
  if (session.error || session.survival.health === 0) return false
  return commit({ ...session.snapshot, hazards: session.hazards, survival: session.survival, inventory: increaseResourceInventoryCapacity(session.snapshot.inventory, capacityGrams, capacitySlots, RESOURCE_CATALOG, session.itemCatalog) })
}
