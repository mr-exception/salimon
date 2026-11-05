import { CELL_SIZE_M } from './coordinates.ts'
import { leaveCockpit } from './cockpit.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { gatherResource, increaseResourceInventoryCapacity, resourceInventoryUsage, startingResourceInventory, type ResourceInventory, type ResourceSource } from './resourceInventory.ts'
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
export const ITEM_CATALOG = {} as const
export const RESOURCE_SOURCES: Readonly<Record<string, ResourceSource>> = {
  'source:landing:mineral:01': { id: 'source:landing:mineral:01', resourceId: 'mineral', remainingGrams: 60_000 },
  'source:landing:biology:01': { id: 'source:landing:biology:01', resourceId: 'biomass', remainingGrams: 15_000 },
}
export interface ResourceSave {
  readonly version: 2
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
export function decodeResourceSave(raw: string, world: TopDownState): ResourceSave {
  const data = object(JSON.parse(raw))
  const legacy = data.version === 1 && data.catalogVersion === LEGACY_CATALOG_VERSION
  if ((!legacy && (data.version !== 2 || data.catalogVersion !== RESOURCE_CATALOG_VERSION)) || data.siteId !== STARTING_SITE.id) {
    throw new Error('Unsupported save version')
  }
  const inv = object(data.inventory), quantities: Record<string, number> = {}
  for (const [id, quantity] of Object.entries(object(inv.quantities))) {
    if (!Object.hasOwn(RESOURCE_CATALOG, id)) throw new Error('Unknown resource')
    const count = integer(quantity, 1)
    quantities[id] = legacy ? integer(count * LEGACY_GRAMS_PER_LITRE[id as keyof typeof LEGACY_GRAMS_PER_LITRE], 1) : count
  }
  const items: Record<string, number> = {}
  if (!legacy) {
    for (const [id, count] of Object.entries(object(inv.items))) items[id] = integer(count, 1)
  }
  const inventory: ResourceInventory = {
    capacityGrams: integer(inv.capacityGrams, 10_000),
    capacitySlots: legacy ? 12 : integer(inv.capacitySlots, 12),
    quantities, items,
  }
  resourceInventoryUsage(inventory, RESOURCE_CATALOG, ITEM_CATALOG)
  const savedSources = object(data.sources), sources: Record<string, ResourceSource> = {}
  if (Object.keys(savedSources).length !== Object.keys(RESOURCE_SOURCES).length) throw new Error('Wrong source set')
  for (const [id, initial] of Object.entries(RESOURCE_SOURCES)) {
    const source = object(savedSources[id])
    const previous = integer(legacy ? source.remainingLitres : source.remainingGrams)
    const remainingGrams = legacy ? integer(previous * LEGACY_GRAMS_PER_LITRE[initial.resourceId as keyof typeof LEGACY_GRAMS_PER_LITRE]) : previous
    if (source.id !== id || source.resourceId !== initial.resourceId || remainingGrams + (quantities[initial.resourceId] ?? 0) !== initial.remainingGrams) throw new Error('Material conservation failed')
    sources[id] = { id, resourceId: initial.resourceId, remainingGrams }
  }
  return { version: 2, catalogVersion: RESOURCE_CATALOG_VERSION, siteId: STARTING_SITE.id, revision: integer(data.revision), inventory, sources, location: parseLocation(data.location, world) }
}
export function resourcePayload(inventory: ResourceInventory, world: TopDownState) {
  const grams = resourceInventoryUsage(inventory, RESOURCE_CATALOG).grams
  return { playerGrams: grams, shipGrams: world.entities.find(e => e.id === PLAYER_ID)!.anchor.frame === world.frames.ship.id ? grams : 0 }
}
export function withResourcePayload(ship: ShipState, inventory: ResourceInventory, world: TopDownState): ShipState {
  // Recompute from dry mass, never add the same nested player payload per tick.
  return setMass(ship, DRY_MASS_KG + resourcePayload(inventory, world).shipGrams / 1000)
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
  equipped = false
  activeSourceId: string | null = null
  progressTicks = 0
  feedback = ''
  error = ''
  private persisted: string | null = null
  private checkpointTick = 0
  private readonly storage: ResourceStorage
  constructor(storage: ResourceStorage, world: TopDownState) {
    this.storage = storage
    this.world = world
    this.snapshot = { version: 2, catalogVersion: RESOURCE_CATALOG_VERSION, siteId: STARTING_SITE.id, revision: 0, inventory: startingResourceInventory(), sources: RESOURCE_SOURCES, location: safeLocation(world) }
    try {
      this.persisted = storage.getItem(RESOURCE_SAVE_KEY)
      if (this.persisted !== null) {
        this.snapshot = decodeResourceSave(this.persisted, world)
        const location = this.snapshot.location
        this.world = { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: location.anchor } : e),
          camera: { ...world.camera, anchor: location.anchor }, player: { facing: location.facing, mode: 'standing', targetId: null }, feedback: 'Saved expedition restored. Gathering is stopped.' }
      }
    } catch {
      this.error = 'Saved expedition could not be loaded safely. Existing data kept; gathering paused.'
    }
  }
  private commit(next: ResourceSave): boolean {
    if (this.error) return false
    try {
      if (this.storage.getItem(RESOURCE_SAVE_KEY) !== this.persisted) throw new Error('Save changed in another session')
      const candidate = { ...next, revision: integer(this.snapshot.revision + 1), location: safeLocation(this.world) }
      const raw = serialize(candidate)
      this.storage.setItem(RESOURCE_SAVE_KEY, raw)
      this.persisted = raw
      this.snapshot = candidate
      this.checkpointTick = this.world.tick
      return true
    } catch {
      this.cancel()
      this.error = 'Saving failed or the save changed elsewhere. Gathering paused; reload after resolving storage access.'
      return false
    }
  }
  checkpoint(): void {
    if (serialize(safeLocation(this.world)) !== serialize(this.snapshot.location)) this.commit(this.snapshot)
  }
  upgrade(capacityGrams: number, capacitySlots: number): boolean {
    return this.commit({ ...this.snapshot, inventory: increaseResourceInventoryCapacity(this.snapshot.inventory, capacityGrams, capacitySlots, RESOURCE_CATALOG, ITEM_CATALOG) })
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
    const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, ITEM_CATALOG)
    const mass = used.grams >= inventory.capacityGrams
    const slots = !inventory.quantities[source.resourceId] && used.slots >= inventory.capacitySlots
    return mass || slots ? `Inventory full: ${mass && slots ? 'weight and slot' : mass ? 'weight' : 'slot'} limit. Remaining material stays in the source.` : ''
  }
  start(id: string | null): void {
    if (this.activeSourceId) { this.cancel(); return }
    if (this.error || !this.equipped) return
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
    this.world = next
    const id = this.activeSourceId
    if (id) {
      const target = next.entities.find(e => e.id === id), player = next.entities.find(e => e.id === PLAYER_ID)!
      if (!target || next.player.mode !== 'interacting' || next.player.targetId !== id || next.cockpit.view !== 'world' || !interactionInReach(player.anchor, target, next.frames)) this.cancel('Gathering interrupted. Unfinished second discarded.')
      else {
        this.progressTicks += elapsed
        while (this.activeSourceId && this.progressTicks >= 60) {
          const gathered = gatherResource(this.snapshot.inventory, this.snapshot.sources[id]!, 1000, RESOURCE_CATALOG, ITEM_CATALOG)
          if (gathered.collectedGrams && this.commit({ ...this.snapshot, inventory: gathered.inventory, sources: { ...this.snapshot.sources, [id]: gathered.source } })) {
            this.progressTicks -= 60
            this.feedback = `Gathered ${gathered.collectedGrams / 1000} kg · ${RESOURCE_CATALOG[gathered.source.resourceId as keyof typeof RESOURCE_CATALOG].name}. Saved.`
          }
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
