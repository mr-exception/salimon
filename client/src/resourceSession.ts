import { advanceSurvival, freshSurvival, recoverOffline, validateSurvival, survivalPolicy, type SurvivalState } from './survival.ts'
import { emptyResourceStores, resourceLedgerMass, transferResource, validateResourceLedger, type ResourceLedger, type ResourceStoreCatalog, type ResourceTransfer } from './resourceLedger.ts'
import { SCOUT_RESOURCE_CONTAINER_ID, SCOUT_RESOURCE_STORES } from './scoutResourceContainer.ts'
import { CELL_SIZE_M } from './coordinates.ts'
import { leaveCockpit } from './cockpit.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { addCountItem, removeCountItem, registerCountItems, type CountItemCatalog, increaseResourceInventoryCapacity, resourceInventoryUsage, startingResourceInventory, type ResourceInventory, type ResourceSource } from './resourceInventory.ts'
import { DRY_MASS_KG, setMass, type ShipState } from './simulation.ts'
import { relativeAnchor, rotate2, type Anchor } from './topDownFrames.ts'
import { interactionInReach, PLAYER_RADIUS_M } from './topDownInteraction.ts'
import { capturePlayerLocation, PLAYER_ID, type TopDownState } from './topDownRuntime.ts'
import type { PlayerLocation } from './topDownTraversal.ts'

export const RESOURCE_SAVE_KEY = 'salimon:expedition:v1'
export const RESOURCE_CATALOG_VERSION = 'landing-resources:v2'
// The existing v1 localStorage key is retained for safe one-time save migration.
const LEGACY_CATALOG_VERSION = 'landing-resources:v1'
const LEGACY_GRAMS_PER_LITRE = { mineral: 2500, biomass: 500 } as const
export const RESOURCE_CATALOG = {
  mineral: { name: 'Banded mineral' },
  biomass: { name: 'Plate biomass' },
} as const
// Count items are supported by the inventory domain; actual item catalog weights are TBD.
export const ITEM_CATALOG = registerCountItems({})
export const RESOURCE_SOURCES: Readonly<Record<string, ResourceSource>> = {
  'source:landing:mineral:01': { id: 'source:landing:mineral:01', resourceId: 'mineral', remainingGrams: 60_000 },
  'source:landing:biology:01': { id: 'source:landing:biology:01', resourceId: 'biomass', remainingGrams: 15_000 },
}
export interface ResourceSave extends ResourceLedger {
  readonly version: 6
  readonly survival: SurvivalState
  readonly itemDefinitions: CountItemCatalog
  readonly catalogVersion: typeof RESOURCE_CATALOG_VERSION
  readonly siteId: typeof STARTING_SITE.id
  readonly revision: number
  readonly inventory: ResourceInventory
  readonly sources: Readonly<Record<string, ResourceSource>>
  readonly location: PlayerLocation
}
export interface ResourceStorage { getItem(key: string): string | null; setItem(key: string, value: string): void }
const serialize = (value: unknown) => JSON.stringify(value, (_, v) => typeof v === 'bigint' ? v.toString() : v)
function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Expected object')
  return value as Record<string, unknown>
}
function integer(value: unknown, min = 0): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < min) throw new Error('Invalid integer')
  return value
}
function parseLocation(value: unknown, world: TopDownState): PlayerLocation {
  const location = object(value), anchor = object(location.anchor), point = object(anchor.point)
  if (anchor.frame !== world.frames.planet.id && anchor.frame !== world.frames.ship.id) throw new Error('Unknown frame')
  if (!Array.isArray(point.cell) || point.cell.length !== 3 || !Array.isArray(point.offsetM) || point.offsetM.length !== 3) throw new Error('Invalid position')
  const cell = point.cell.map((v: unknown) => {
    if (typeof v !== 'string' || !/^-?(0|[1-9][0-9]{0,127})$/.test(v)) throw new Error('Invalid cell')
    return BigInt(v)
  }) as [bigint, bigint, bigint]
  const offsetM = point.offsetM.map((v: unknown) => {
    if (typeof v !== 'number' || !Number.isFinite(v) || v < -CELL_SIZE_M / 2 || v >= CELL_SIZE_M / 2) throw new Error('Invalid offset')
    return v
  }) as [number, number, number]
  if (cell[2] !== 0n || offsetM[2] !== 0 || typeof location.facing !== 'number' || !Number.isFinite(location.facing)) throw new Error('Invalid planar location')
  const parsed: Anchor = { frame: anchor.frame, point: { cell, offsetM } }
  for (const entity of world.entities) {
    if (!entity.footprint) continue
    const [x, y] = rotate2(relativeAnchor(parsed, entity.anchor, world.frames), entity.anchor.frame === world.frames.ship.id ? -world.frames.ship.angle : 0)
    if (![x, y].every(Number.isFinite) || (Math.abs(x) < entity.footprint.halfWidth + PLAYER_RADIUS_M - 1e-8 && Math.abs(y) < entity.footprint.halfHeight + PLAYER_RADIUS_M - 1e-8)) throw new Error('Unsafe saved location')
  }
  return { anchor: parsed, facing: location.facing }
}
/** Reject inconsistent saves, and migrate whole-litre v1 saves to exact grams without loss. */
export function decodeResourceSave(raw: string, world: TopDownState, storeDefinitions: ResourceStoreCatalog = SCOUT_RESOURCE_STORES, itemDefinitions: CountItemCatalog = ITEM_CATALOG): ResourceSave {
  const catalog = registerCountItems(itemDefinitions)
  const data = object(JSON.parse(raw))
  const allowed = ['version', 'catalogVersion', 'siteId', 'revision', 'inventory', 'sources', 'location', ...((data.version === 3 || data.version === 4 || (data.version === 5 || data.version === 6)) ? ['stores'] : []), ...((data.version === 5 || data.version === 6) ? ['itemDefinitions'] : []), ...(data.version === 6 ? ['survival'] : [])]
  if (Object.keys(data).some(key => !allowed.includes(key))) throw new Error('Unknown save metadata')
  const legacy = data.version === 1 && data.catalogVersion === LEGACY_CATALOG_VERSION
  if ((!legacy && ((data.version !== 2 && data.version !== 3 && data.version !== 4 && data.version !== 5 && data.version !== 6) || data.catalogVersion !== RESOURCE_CATALOG_VERSION)) || data.siteId !== STARTING_SITE.id) {
    throw new Error('Unsupported save version')
  }
  if (data.version !== 3 && data.version !== 4 && data.version !== 5 && data.version !== 6 && Object.hasOwn(data, 'stores')) throw new Error('Stores require schema v3')
  const inv = object(data.inventory), quantities: Record<string, number> = {}
  for (const [id, quantity] of Object.entries(object(inv.quantities))) {
    if (!Object.hasOwn(RESOURCE_CATALOG, id)) throw new Error('Unknown resource')
    const count = integer(quantity, 1)
    quantities[id] = legacy ? integer(count * LEGACY_GRAMS_PER_LITRE[id as keyof typeof LEGACY_GRAMS_PER_LITRE], 1) : count
  }
  const items: Record<string, number> = {}
  if (!legacy) {
    for (const [id, count] of Object.entries(object(inv.items))) {
      if (!Object.hasOwn(catalog, id)) throw new Error('Unknown item')
      items[id] = integer(count, 1)
    }
  }
  const inventory: ResourceInventory = {
    capacityGrams: integer(inv.capacityGrams, 10_000),
    capacitySlots: legacy ? 12 : integer(inv.capacitySlots, 12),
    quantities, items,
  }
  if ((data.version === 5 || data.version === 6)) {
    const saved = registerCountItems(object(data.itemDefinitions) as CountItemCatalog)
    if (Object.keys(saved).length !== Object.keys(catalog).length || Object.entries(saved).some(([id, meta]) =>
      !Object.hasOwn(catalog, id) || meta.gramsPerItem !== catalog[id]!.gramsPerItem || meta.stackSize !== catalog[id]!.stackSize || meta.name !== catalog[id]!.name)) throw new Error('Changed item definitions')
  } else if (Object.keys(items).length || (legacy && Object.hasOwn(inv, 'items'))) {
    throw new Error('Items require schema v5')
  }
  resourceInventoryUsage(inventory, RESOURCE_CATALOG, catalog)
  if (legacy) {
    const capacityLitres = integer(inv.capacityLitres, 10)
    const litres = Object.values(object(inv.quantities)).reduce<number>((n, q) => integer(n + integer(q, 1)), 0)
    if (litres > capacityLitres) throw new Error('Legacy volume capacity exceeded')
  }
  const savedSources = object(data.sources), sources: Record<string, ResourceSource> = {}
  if (Object.keys(savedSources).length !== Object.keys(RESOURCE_SOURCES).length) throw new Error('Wrong source set')
  for (const [id, initial] of Object.entries(RESOURCE_SOURCES)) {
    const source = object(savedSources[id])
    const previous = integer(legacy ? source.remainingLitres : source.remainingGrams)
    const remainingGrams = legacy ? integer(previous * LEGACY_GRAMS_PER_LITRE[initial.resourceId as keyof typeof LEGACY_GRAMS_PER_LITRE]) : previous
    if (source.id !== id || source.resourceId !== initial.resourceId) throw new Error('Material conservation failed')
    sources[id] = { id, resourceId: initial.resourceId, remainingGrams }
  }
  const savedStores = data.version === 3 || data.version === 4 || (data.version === 5 || data.version === 6) ? object(data.stores) : null
  // Only the newly introduced Scout store may be absent in v3. v4 requires it.
  const stores = savedStores ? { ...savedStores } as Record<string, ResourceLedger['stores'][string]> : emptyResourceStores(storeDefinitions)
  if (data.version === 3 && Object.hasOwn(storeDefinitions, SCOUT_RESOURCE_CONTAINER_ID) && !Object.hasOwn(stores, SCOUT_RESOURCE_CONTAINER_ID)) {
    stores[SCOUT_RESOURCE_CONTAINER_ID] = emptyResourceStores({ [SCOUT_RESOURCE_CONTAINER_ID]: storeDefinitions[SCOUT_RESOURCE_CONTAINER_ID]! })[SCOUT_RESOURCE_CONTAINER_ID]!
  }
  const save: ResourceSave = { version: 6, survival: data.version === 6 ? validateSurvival(data.survival) : freshSurvival(), itemDefinitions: catalog, catalogVersion: RESOURCE_CATALOG_VERSION, siteId: STARTING_SITE.id, revision: integer(data.revision), inventory, sources, stores, location: parseLocation(data.location, world) }
  validateResourceLedger(save, RESOURCE_SOURCES, storeDefinitions, RESOURCE_CATALOG, catalog)
  return save
}
export function resourcePayload(inventory: ResourceInventory, world: TopDownState, items: CountItemCatalog = ITEM_CATALOG) {
  const grams = resourceInventoryUsage(inventory, RESOURCE_CATALOG, items).grams
  return { playerGrams: grams, shipGrams: world.entities.find(e => e.id === PLAYER_ID)!.anchor.frame === world.frames.ship.id ? grams : 0 }
}
export function withResourcePayload(ship: ShipState, inventory: ResourceInventory, world: TopDownState, items: CountItemCatalog = ITEM_CATALOG): ShipState {
  // Recompute from dry mass, never add the same nested player payload per tick.
  return setMass(ship, DRY_MASS_KG + resourcePayload(inventory, world, items).shipGrams / 1000)
}
function safeLocation(world: TopDownState): PlayerLocation {
  // A reload returns standing beside the chair, never inside its collider.
  return capturePlayerLocation(world.player.mode === 'seated' ? leaveCockpit(world) : world)
}

/** Single-writer local session. Browser adapter holds a Web Lock for its lifetime.
 * One synchronous storage replacement commits inventory + source + location.
 * Publish only after it succeeds; a failed/stale write stops gathering.
 */
export class ResourceSession {
  world: TopDownState
  snapshot: ResourceSave
  survival: SurvivalState
  equipped = false
  activeSourceId: string | null = null
  progressTicks = 0
  feedback = ''
  error = ''
  private persisted: string | null = null
  private checkpointTick = 0
  private readonly storage: ResourceStorage
  private readonly storeDefinitions: ResourceStoreCatalog
  private readonly now: () => number
  readonly itemCatalog: CountItemCatalog
  constructor(storage: ResourceStorage, world: TopDownState, storeDefinitions: ResourceStoreCatalog = SCOUT_RESOURCE_STORES, itemDefinitions: CountItemCatalog = ITEM_CATALOG, now: () => number = Date.now) {
    this.now = now
    this.survival = freshSurvival(now())
    this.itemCatalog = registerCountItems(itemDefinitions)
    this.storeDefinitions = structuredClone(storeDefinitions)
    this.storage = storage
    this.world = world
    this.snapshot = { version: 6, survival: this.survival, itemDefinitions: this.itemCatalog, stores: emptyResourceStores(storeDefinitions), catalogVersion: RESOURCE_CATALOG_VERSION, siteId: STARTING_SITE.id, revision: 0, inventory: startingResourceInventory(), sources: RESOURCE_SOURCES, location: safeLocation(world) }
    try {
      validateResourceLedger(this.snapshot, RESOURCE_SOURCES, this.storeDefinitions, RESOURCE_CATALOG, this.itemCatalog)
      this.persisted = storage.getItem(RESOURCE_SAVE_KEY)
      if (this.persisted !== null) {
        this.snapshot = decodeResourceSave(this.persisted, world, this.storeDefinitions, this.itemCatalog)
        this.survival = this.snapshot.survival
        const location = this.snapshot.location
        this.world = { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: location.anchor } : e),
          camera: { ...world.camera, anchor: location.anchor }, player: { facing: location.facing, mode: 'standing', targetId: null }, feedback: 'Saved expedition restored. Gathering is stopped.' }
      }
      if (this.persisted !== null && JSON.parse(this.persisted).version === 6) {
        const recovered = recoverOffline(this.survival, now())
        if (recovered.fatigue !== this.survival.fatigue) this.commit({ ...this.snapshot, survival: recovered })
        this.survival = this.snapshot.survival
      } else if (this.persisted !== null) {
        this.survival = freshSurvival(now())
        this.snapshot = { ...this.snapshot, survival: this.survival }
      }
    } catch {
      this.error = 'Saved expedition could not be loaded safely. Existing data kept; gathering paused.'
    }
  }
  private commit(next: ResourceSave): boolean {
    if (this.error) return false
    try {
      if (this.storage.getItem(RESOURCE_SAVE_KEY) !== this.persisted) throw new Error('Save changed in another session')
      const candidate = { ...next, survival: { ...next.survival, offlineAtMs: Math.max(next.survival.offlineAtMs, this.now()) }, revision: integer(this.snapshot.revision + 1), location: safeLocation(this.world) }
      validateResourceLedger(candidate, RESOURCE_SOURCES, this.storeDefinitions, RESOURCE_CATALOG, this.itemCatalog)
      validateSurvival(candidate.survival)
      const raw = serialize(candidate)
      this.storage.setItem(RESOURCE_SAVE_KEY, raw)
      this.persisted = raw
      this.snapshot = candidate
      this.survival = candidate.survival
      this.checkpointTick = this.world.tick
      return true
    } catch {
      this.cancel()
      this.survival = this.snapshot.survival
      this.error = 'Saving failed or the save changed elsewhere. Gathering paused; reload after resolving storage access.'
      return false
    }
  }
  /** Domain rejection leaves the current save usable; storage failure blocks writes. */
  transfer(request: ResourceTransfer): boolean {
    if (this.error || this.survival.health === 0) return false
    try {
      const next = transferResource(this.snapshot, request, RESOURCE_SOURCES, this.storeDefinitions, RESOURCE_CATALOG, this.itemCatalog)
      return this.commit({ ...this.snapshot, ...next, survival: this.survival })
    } catch (error) {
      this.feedback = error instanceof Error ? error.message : 'Invalid transfer'
      return false
    }
  }
  /** Generic adapter boundary for future item producers/consumers. No auto-equip or player grants. */
  changeItem(action: 'add' | 'remove', id: string, count: number, revision: number): boolean {
    if (this.error || this.survival.health === 0) return false
    try {
      if (!Number.isSafeInteger(revision) || revision !== this.snapshot.revision) throw new RangeError('Stale item revision')
      if (action !== 'add' && action !== 'remove') throw new RangeError('Unknown item action')
      const inventory = (action === 'add' ? addCountItem : removeCountItem)(this.snapshot.inventory, id, count, RESOURCE_CATALOG, this.itemCatalog)
      const saved = this.commit({ ...this.snapshot, survival: this.survival, inventory })
      if (saved) this.feedback = `Saved: ${action === 'add' ? 'added' : 'removed'} ${count} × ${this.itemCatalog[id]!.name ?? id}.`
      return saved
    } catch (error) {
      this.feedback = error instanceof Error ? error.message : 'Invalid item transaction'
      return false
    }
  }
  /** Explicit exact grams, with the same atomic save and stale-revision guard as mining. */
  deposit(resourceId: string, grams: number, revision: number): boolean {
    return this.transfer({ revision, from: { kind: 'player' }, to: { kind: 'store', id: SCOUT_RESOURCE_CONTAINER_ID }, resourceId, grams })
  }
  withdraw(resourceId: string, grams: number, revision: number): boolean {
    return this.transfer({ revision, from: { kind: 'store', id: SCOUT_RESOURCE_CONTAINER_ID }, to: { kind: 'player' }, resourceId, grams })
  }
  /** Explicit terminal-only fresh character. The world ledger and depletion survive. */
  freshRun(): boolean {
    if (this.error || this.survival.health !== 0) return false
    const next = freshSurvival(this.now())
    if (!this.commit({ ...this.snapshot, survival: next })) return false
    this.survival = next
    this.equipped = false
    return true
  }
  payload() {
    return resourceLedgerMass(this.snapshot, this.world.frames.ship.id,
      this.world.entities.find(e => e.id === PLAYER_ID)!.anchor.frame === this.world.frames.ship.id, RESOURCE_CATALOG, this.itemCatalog)
  }
  checkpoint(): void {
    if (serialize(safeLocation(this.world)) !== serialize(this.snapshot.location) || serialize(this.survival) !== serialize(this.snapshot.survival)) this.commit({ ...this.snapshot, survival: this.survival })
  }
  upgrade(capacityGrams: number, capacitySlots: number): boolean {
    if (this.error || this.survival.health === 0) return false
    return this.commit({ ...this.snapshot, survival: this.survival, inventory: increaseResourceInventoryCapacity(this.snapshot.inventory, capacityGrams, capacitySlots, RESOURCE_CATALOG, this.itemCatalog) })
  }
  cancel(message = 'Gathering stopped. Completed grams are saved.'): void {
    if (this.activeSourceId) this.feedback = message
    this.activeSourceId = null
    this.progressTicks = 0
  }
  toggleTool(): void {
    this.cancel()
    this.equipped = !this.equipped
    this.feedback = this.equipped ? 'Gathering tool equipped · approach a source, then Gather [E].' : 'Gathering tool stowed.'
  }
  private blocked(id: string): string {
    const source = this.snapshot.sources[id]
    if (!source || source.remainingGrams === 0) return 'Source depleted. No material remains.'
    const inventory = this.snapshot.inventory
    const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, this.itemCatalog)
    const mass = used.grams >= inventory.capacityGrams
    const slots = !inventory.quantities[source.resourceId] && used.slots >= inventory.capacitySlots
    return mass || slots ? `Inventory full: ${mass && slots ? 'weight and slot' : mass ? 'weight' : 'slot'} limit. Remaining material stays in the source.` : ''
  }
  start(id: string | null): void {
    if (this.activeSourceId) { this.cancel(); return }
    if (this.error || !survivalPolicy(this.survival).canWork || !this.equipped) return
    const target = this.world.entities.find(e => e.id === id), player = this.world.entities.find(e => e.id === PLAYER_ID)!
    if (!target?.source || !id || this.world.player.mode === 'seated' || this.world.cockpit.view !== 'world' || !interactionInReach(player.anchor, target, this.world.frames)) {
      this.feedback = 'No resource source in reach.'; return
    }
    const blocked = this.blocked(id)
    if (blocked) { this.feedback = blocked; return }
    this.activeSourceId = id
    this.progressTicks = 0
    this.world = { ...this.world, selectedId: id, player: { ...this.world.player, mode: 'interacting', targetId: id } }
    this.feedback = 'Gathering · 1 kg/s. Move or Escape to stop.'
  }
  advance(next: TopDownState): TopDownState {
    const elapsed = Math.max(0, next.tick - this.world.tick), wasWalking = this.world.player.mode === 'walking'
    if (this.error || this.survival.health === 0) return this.world
    this.survival = advanceSurvival(this.survival, elapsed)
    this.world = next
    if (!survivalPolicy(this.survival).canWork) {
      this.cancel('Exhausted: gathering and piloting unavailable.')
      if (this.world.player.mode === 'seated') this.world = leaveCockpit(this.world)
    }
    if (this.survival.health === 0) { this.world = { ...this.world, player: { ...this.world.player, mode: 'standing', targetId: null } }; this.checkpoint(); return this.world }
    const id = this.activeSourceId
    if (id) {
      const target = next.entities.find(e => e.id === id), player = next.entities.find(e => e.id === PLAYER_ID)!
      if (!target || next.player.mode !== 'interacting' || next.player.targetId !== id || next.cockpit.view !== 'world' || !interactionInReach(player.anchor, target, next.frames)) this.cancel('Gathering interrupted. Unfinished second discarded.')
      else {
        this.progressTicks += elapsed
        while (this.activeSourceId && this.progressTicks >= 60) {
          const source = this.snapshot.sources[id]!, inventory = this.snapshot.inventory
          const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, this.itemCatalog)
          const grams = inventory.quantities[source.resourceId] || used.slots < inventory.capacitySlots
            ? Math.min(1000, source.remainingGrams, inventory.capacityGrams - used.grams) : 0
          if (grams && this.transfer({ revision: this.snapshot.revision, from: { kind: 'source', id }, to: { kind: 'player' }, resourceId: source.resourceId, grams })) {
            this.progressTicks -= 60
            this.feedback = `Gathered ${grams / 1000} kg · ${RESOURCE_CATALOG[source.resourceId as keyof typeof RESOURCE_CATALOG].name}. Saved.`
          } else if (grams) this.cancel('Gathering transaction rejected.')
          const blocked = this.blocked(id)
          if (blocked) { this.cancel(); this.feedback = blocked }
        }
      }
    }
    if (wasWalking && next.player.mode !== 'walking' || next.tick - this.checkpointTick >= 60) {
      this.checkpoint()
      this.checkpointTick = next.tick
    }
    return this.world
  }
}
