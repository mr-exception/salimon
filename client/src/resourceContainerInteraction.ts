import { BENCH_ID, BENCH_INPUT_ID, benchReserved } from './gearBench.ts'
import { RESOURCE_CATALOG, type ResourceSession } from './resourceSession.ts'
import type { ResourceInventory } from './resourceInventory.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { resourceStoreMass } from './resourceLedger.ts'
import { SCOUT_RESOURCE_CONTAINER_ID as containerId } from './scoutResourceContainer.ts'
import { FABRICATOR_INPUT_ID, FABRICATOR_RECIPE_ID, FABRICATOR_GRAMS, MARKER_KIT_ID } from './scoutFabricator.ts'
import { survivalPolicy } from './survival.ts'
import { SCOUT_CORE_FUEL_ID } from './scoutEnergyCore.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { cancelInteraction, PLAYER_ID, requestInteraction, type TopDownState } from './topDownRuntime.ts'

export type ContainerDirection = 'deposit' | 'withdraw'
export interface ContainerCommand { readonly direction: ContainerDirection; readonly resourceId: string; readonly grams: number; readonly revision: number }
/** Explain the two independent shared player constraints using validated usage. */
export function withdrawalCapacityError(inventory: ResourceInventory, used: { grams: number; slots: number }, resourceId: string, grams: number): string {
  if (used.grams + grams > inventory.capacityGrams) return `Player overweight: only ${inventory.capacityGrams - used.grams} g free.`
  if (!inventory.quantities[resourceId] && used.slots >= inventory.capacitySlots) return `Player slots full: ${used.slots} / ${inventory.capacitySlots}. Free a slot first.`
  return ''
}
/** UI draft/confirmation state only. ResourceSession remains the sole save authority. */
export class ResourceContainerInteraction {
  readonly session: ResourceSession
  readonly equipment: 'container' | 'core' | 'fabricator' | 'bench'
  readonly targetId: string
  readonly storeId: string
  get title(): string { return this.equipment === 'bench' ? 'Gear Crafting Bench' : this.equipment === 'fabricator' ? 'Fabricator' : this.equipment === 'core' ? 'Energy Core' : 'Resource Container' }
  opened = false
  pending: ContainerCommand | null = null
  message = ''
  failed = false
  private reviewedResources = ''
  private resourceSignature(): string {
    const { inventory, sources, stores, itemDefinitions } = this.session.snapshot
    // Conversion is allowed while reviewing: normalize fuel + converted grams.
    // Other material transactions still invalidate the reviewed revision.
    const normalized = { ...stores, [SCOUT_CORE_FUEL_ID]: {
      ...stores[SCOUT_CORE_FUEL_ID], quantities: Object.fromEntries(Object.keys(RESOURCE_CATALOG).map(id =>
        [id, (stores[SCOUT_CORE_FUEL_ID]?.quantities[id] ?? 0) + (this.session.snapshot.convertedGrams[id] ?? 0)])),
    } }
    return JSON.stringify({ inventory, sources, stores: normalized, itemDefinitions })
  }
  constructor(session: ResourceSession, equipment: 'container' | 'core' | 'fabricator' | 'bench' = 'container') {
    this.session = session; this.equipment = equipment
    this.targetId = equipment === 'bench' ? BENCH_ID : equipment === 'fabricator' ? 'scout:fabricator' : equipment === 'core' ? 'scout:core' : containerId
    this.storeId = equipment === 'bench' ? BENCH_INPUT_ID : equipment === 'fabricator' ? FABRICATOR_INPUT_ID : equipment === 'core' ? SCOUT_CORE_FUEL_ID : containerId
  }
  get bufferGrams(): number {
    const r = this.equipment === 'bench' ? benchReserved(this.session.snapshot.bench) : null
    return (r ? r.mineral + r.biomass : 0) + resourceStoreMass(this.session.snapshot.stores[this.storeId]!, RESOURCE_CATALOG)
      + (this.equipment === 'fabricator' ? this.session.snapshot.fabricator.job?.reservedGrams ?? 0 : 0)
  }
  get machineBlocked(): string | undefined {
    return this.session.error || (!survivalPolicy(this.session.survival).canWork ? 'Unable to work in the current survival condition.' : undefined)
  }
  get startBlocked(): string | undefined {
    const { fabricator, stores } = this.session.snapshot
    return this.machineBlocked || (fabricator.job ? 'One job is already active.' : fabricator.tray ? 'Claim the output tray before starting another job.' :
      (stores[FABRICATOR_INPUT_ID]?.quantities.mineral ?? 0) < FABRICATOR_GRAMS ? 'Requires 1,000 g of unreserved mineral in the input buffer.' : undefined)
  }
  private claimCapacityError(): string | undefined {
    const inventory = this.session.snapshot.inventory
    const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, this.session.itemCatalog)
    const item = this.session.itemCatalog[MARKER_KIT_ID]!
    if (used.grams + item.gramsPerItem > inventory.capacityGrams) return `Player overweight: only ${inventory.capacityGrams - used.grams} g free. Output kept in tray.`
    if (used.slots + 1 > inventory.capacitySlots) return `Player slots full: ${used.slots} / ${inventory.capacitySlots}. Output kept in tray.`
    return undefined
  }
  private machineAction(kind: 'start' | 'claim'): boolean {
    this.synchronize(this.session.world)
    if (!this.opened || this.equipment !== 'fabricator') return false
    this.discard()
    const tray = this.session.snapshot.fabricator.tray
    const ok = kind === 'start' ? this.session.startFabricator(FABRICATOR_RECIPE_ID, this.session.snapshot.revision)
      : this.session.claimFabricator(tray?.jobId ?? -1, this.session.snapshot.revision)
    this.failed = !ok
    this.message = ok ? kind === 'start' ? 'Saved: job started. Inputs reserved once.' : 'Saved: output claimed to carried inventory.'
      : this.machineBlocked || (kind === 'claim' && tray ? this.claimCapacityError() : undefined) || this.session.feedback
    return ok
  }
  startBench(recipeId:string):boolean {
    this.synchronize(this.session.world)
    if(!this.opened||this.equipment!=='bench')return false
    this.discard()
    const ok=this.session.startBench(recipeId,this.session.snapshot.revision)
    this.failed=!ok;this.message=ok?'Saved: gear job started. Inputs reserved once.':this.session.feedback
    return ok
  }
  claimBench():boolean {
    this.synchronize(this.session.world)
    if(!this.opened||this.equipment!=='bench')return false
    this.discard()
    const ok=this.session.claimBench(this.session.snapshot.bench.tray?.jobId??-1,this.session.snapshot.revision)
    this.failed=!ok;this.message=ok?'Saved: equipment claimed to inventory.':this.session.feedback
    return ok
  }
  start(): boolean { return this.machineAction('start') }
  claim(): boolean { return this.machineAction('claim') }
  private reachable(world: TopDownState): boolean {
    const player = world.entities.find(e => e.id === PLAYER_ID), target = world.entities.find(e => e.id === this.targetId)
    return world.cockpit.view === 'world' && world.player.mode !== 'seated' && !!player && !!target
      && interactionInReach(player.anchor, target, world.frames)
  }
  open(world: TopDownState): boolean {
    if (world.selectedId !== this.targetId || !this.reachable(world)) return false
    this.session.cancel()
    this.session.world = requestInteraction(world, this.targetId)
    // Persist arrival before capturing any draft revision. A later location-only
    // autosave must not invalidate an otherwise unchanged transfer confirmation.
    this.session.checkpoint()
    this.opened = true; this.pending = null; this.message = ''; this.failed = false
    return true
  }
  close(): void {
    this.opened = false; this.pending = null
    if (this.session.world.player.targetId === this.targetId) this.session.world = cancelInteraction(this.session.world)
  }
  synchronize(world: TopDownState): void {
    this.session.world = world
    if (this.opened && (!this.reachable(world) || world.player.mode !== 'interacting' || world.player.targetId !== this.targetId)) this.close()
  }
  discard(): void { this.pending = null; this.message = ''; this.failed = false }
  review(direction: ContainerDirection, resourceId: string, amount: string): boolean {
    this.pending = null; this.failed = true
    this.synchronize(this.session.world)
    if (!this.opened) { this.message = `${this.title} is out of reach. Approach it and use E.`; return false }
    if (this.session.error) { this.message = this.session.error; return false }
    if (!Object.hasOwn(RESOURCE_CATALOG, resourceId)) { this.message = 'Invalid raw resource. Count-based items cannot be transferred.'; return false }
    const grams = Number(amount.trim())
    if (!/^\d+$/.test(amount.trim()) || !Number.isSafeInteger(grams) || grams <= 0) {
      this.message = 'Enter a positive whole number of grams (for example 333).'; return false
    }
    const store = this.session.snapshot.stores[this.storeId]!, inv = this.session.snapshot.inventory
    const available = direction === 'deposit' ? inv.quantities[resourceId] ?? 0 : store.quantities[resourceId] ?? 0
    if (grams > available) { this.message = `Insufficient amount: only ${available} g available to ${direction}.`; return false }
    if (direction === 'deposit' && this.bufferGrams + grams > store.capacityGrams) {
      this.message = `${this.equipment === 'fabricator' || this.equipment === 'bench' ? 'Input buffer' : this.equipment === 'core' ? 'Fuel buffer' : 'Container'} overweight: only ${store.capacityGrams - this.bufferGrams} g free (50 kg maximum).`; return false
    }
    if (direction === 'withdraw') {
      const used = resourceInventoryUsage(inv, RESOURCE_CATALOG, this.session.itemCatalog)
      const error = withdrawalCapacityError(inv, used, resourceId, grams)
      if (error) { this.message = error; return false }
    }
    this.reviewedResources = this.resourceSignature()
    this.pending = { direction, resourceId, grams, revision: this.session.snapshot.revision }
    this.failed = false; this.message = 'Review the exact amount, then confirm. Nothing has moved yet.'
    return true
  }
  confirm(): boolean {
    this.synchronize(this.session.world)
    const command = this.pending
    this.pending = null // Consume before committing: repeated events cannot duplicate a command.
    if (!this.opened || !command) return false
    // Clock/location-only commits do not alter the reviewed transfer. A material
    // change still uses its stale revision and is rejected by the session.
    const revision = this.reviewedResources === this.resourceSignature() ? this.session.snapshot.revision : command.revision
    const method = this.equipment === 'bench' ? command.direction === 'deposit' ? 'depositBenchInput' : 'withdrawBenchInput' : this.equipment === 'fabricator' ? command.direction === 'deposit' ? 'depositFabricatorInput' : 'withdrawFabricatorInput' : this.equipment === 'core' ? command.direction === 'deposit' ? 'depositCoreFuel' : 'withdrawCoreFuel' : command.direction
    const ok = this.session[method](command.resourceId, command.grams, revision)
    this.failed = !ok
    this.message = ok ? `Saved: ${command.direction === 'deposit' ? 'deposited' : 'withdrew'} ${command.grams} g (${command.grams / 1000} kg).`
      : this.session.error || (/Stale/.test(this.session.feedback) ? 'Stale transaction. Amounts changed; review a new transfer.' : this.session.feedback)
    return ok
  }
}
