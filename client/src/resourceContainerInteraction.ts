import { RESOURCE_CATALOG, ITEM_CATALOG, type ResourceSession } from './resourceSession.ts'
import type { ResourceInventory } from './resourceInventory.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { resourceStoreMass } from './resourceLedger.ts'
import { SCOUT_RESOURCE_CONTAINER_ID as containerId } from './scoutResourceContainer.ts'
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
  opened = false
  pending: ContainerCommand | null = null
  message = ''
  failed = false
  constructor(session: ResourceSession) { this.session = session }
  private reachable(world: TopDownState): boolean {
    const player = world.entities.find(e => e.id === PLAYER_ID), target = world.entities.find(e => e.id === containerId)
    return world.cockpit.view === 'world' && world.player.mode !== 'seated' && !!player && !!target
      && interactionInReach(player.anchor, target, world.frames)
  }
  open(world: TopDownState): boolean {
    if (world.selectedId !== containerId || !this.reachable(world)) return false
    this.session.cancel()
    this.session.world = requestInteraction(world, containerId)
    // Persist arrival before capturing any draft revision. A later location-only
    // autosave must not invalidate an otherwise unchanged transfer confirmation.
    this.session.checkpoint()
    this.opened = true; this.pending = null; this.message = ''; this.failed = false
    return true
  }
  close(): void {
    this.opened = false; this.pending = null
    if (this.session.world.player.targetId === containerId) this.session.world = cancelInteraction(this.session.world)
  }
  synchronize(world: TopDownState): void {
    this.session.world = world
    if (this.opened && (!this.reachable(world) || world.player.mode !== 'interacting' || world.player.targetId !== containerId)) this.close()
  }
  discard(): void { this.pending = null; this.message = ''; this.failed = false }
  review(direction: ContainerDirection, resourceId: string, amount: string): boolean {
    this.pending = null; this.failed = true
    this.synchronize(this.session.world)
    if (!this.opened) { this.message = 'Container is out of reach. Approach it and use E.'; return false }
    if (this.session.error) { this.message = this.session.error; return false }
    if (!Object.hasOwn(RESOURCE_CATALOG, resourceId)) { this.message = 'Invalid raw resource. Count-based items cannot be transferred.'; return false }
    const grams = Number(amount.trim())
    if (!/^\d+$/.test(amount.trim()) || !Number.isSafeInteger(grams) || grams <= 0) {
      this.message = 'Enter a positive whole number of grams (for example 333).'; return false
    }
    const store = this.session.snapshot.stores[containerId]!, inv = this.session.snapshot.inventory
    const available = direction === 'deposit' ? inv.quantities[resourceId] ?? 0 : store.quantities[resourceId] ?? 0
    if (grams > available) { this.message = `Insufficient amount: only ${available} g available to ${direction}.`; return false }
    if (direction === 'deposit' && resourceStoreMass(store, RESOURCE_CATALOG) + grams > store.capacityGrams) {
      this.message = `Container overweight: only ${store.capacityGrams - resourceStoreMass(store, RESOURCE_CATALOG)} g free (50 kg maximum).`; return false
    }
    if (direction === 'withdraw') {
      const used = resourceInventoryUsage(inv, RESOURCE_CATALOG, ITEM_CATALOG)
      const error = withdrawalCapacityError(inv, used, resourceId, grams)
      if (error) { this.message = error; return false }
    }
    this.pending = { direction, resourceId, grams, revision: this.session.snapshot.revision }
    this.failed = false; this.message = 'Review the exact amount, then confirm. Nothing has moved yet.'
    return true
  }
  confirm(): boolean {
    this.synchronize(this.session.world)
    const command = this.pending
    this.pending = null // Consume before committing: repeated events cannot duplicate a command.
    if (!this.opened || !command) return false
    const ok = this.session[command.direction](command.resourceId, command.grams, command.revision)
    this.failed = !ok
    this.message = ok ? `Saved: ${command.direction === 'deposit' ? 'deposited' : 'withdrew'} ${command.grams} g (${command.grams / 1000} kg).`
      : this.session.error || (/Stale/.test(this.session.feedback) ? 'Stale transaction. Amounts changed; review a new transfer.' : this.session.feedback)
    return ok
  }
}
