import * as resourceInventoryCommands from './resourceInventoryCommands.ts'
import { synchronizeDeposits } from './resourceDeposits.ts'
import { advanceResourceSession, type ResourceTickControl } from './resourceTick.ts'
import { type GearCommand } from './scoutEquipment.ts'
import { recordVisit, validateJournal } from './expeditionJournal.ts'
import { type WalkingFix, type FieldMark } from './exploration.ts'
import { isDepositId, type PlanetDeposit } from './planetDeposits.ts'
import { surfaceRegion, type SurfaceLocation } from './surfaceCoordinates.ts'
import { resolveSurface, validateSurfaceClearance, type SiteStates } from './surfaceSite.ts'
import { synchronizeDoor } from './scoutDoor.ts'
import { captureScout, decodeScout } from './scoutSave.ts'
import { type ScoutAction } from './scoutLocation.ts'
import { freshHazards, validateHazards, hazardRates, type HazardState, type HazardSource, type HazardEvent, type ProtectionModule } from './hazards.ts'
import { freshSurvival, recoverOffline, validateSurvival, survivalPolicy, type SurvivalState } from './survival.ts'
import { resourceLedgerMass, validateResourceLedger, type ResourceStoreCatalog, type ResourceTransfer } from './resourceLedger.ts'
import { SCOUT_RESOURCE_CONTAINER_ID, SCOUT_RESOURCE_STORES } from './scoutResourceContainer.ts'
import { SCOUT_CORE_FUEL_ID, validateCoreConservation, type CorePowerRequest, type CorePowerResult } from './scoutEnergyCore.ts'
import { LOCAL_PLAYER, type CockpitState } from './cockpit.ts'
import { registerCountItems, type CountItemCatalog, resourceInventoryUsage, type ResourceSource } from './resourceInventory.ts'
import { walkingFrames } from './topDownFrames.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { PLAYER_ID, type TopDownState } from './topDownRuntime.ts'
import { RESOURCE_SAVE_KEY, RESOURCE_CATALOG, ITEM_CATALOG, RESOURCE_SOURCES, type ResourceSave, type ResourceStorage } from './resourceCatalog.ts'
import { serializeForDecode, serialize, integer, materializeSite, validateMiningWork, sourceDefinitions, safeLocation, captureResourceSites } from './resourceSaveLocation.ts'
import { decodeResourceSave, freshResourceSave } from './resourceSave.ts'
import * as resourceMarkerCommands from './resourceMarkerCommands.ts'
import * as resourceMachineCommands from './resourceMachineCommands.ts'
import * as resourceScoutCommands from './resourceScoutCommands.ts'
export { RESOURCE_SAVE_KEY, RESOURCE_CATALOG_VERSION, RESOURCE_CATALOG, ITEM_CATALOG, RESOURCE_SOURCES, type ResourceSave, type ResourceStorage } from './resourceCatalog.ts'
export { decodeResourceSave } from './resourceSave.ts'
export { resourcePayload, withResourcePayload } from './resourcePayload.ts'

/** Single-writer local session. Browser adapter holds a Web Lock for its lifetime.
 * One synchronous storage replacement commits inventory + source + location.
 * Publish only after it succeeds; a failed/stale write stops gathering.
 */
export class ResourceSession {
  /** Durable observations share the resource/ship transaction and writer. */
  get fieldMarks(): readonly FieldMark[] { return this.snapshot.journal.marks }
  walkingFix: WalkingFix | null = null
  markField(name: string, note: string, landmarkId?: string): boolean {
    return resourceMarkerCommands.markField(this, next => this.commit(next), name, note, landmarkId)
  }
  markerDeploymentDenial(): string | null {
    return resourceMarkerCommands.markerDeploymentDenial(this)
  }
  deployMarker(name: string, revision: number, positionId: string): boolean {
    return resourceMarkerCommands.deployMarker(this, next => this.commit(next), name, revision, positionId)
  }
  renameMarker(id: string, name: string, revision: number): boolean {
    return resourceMarkerCommands.renameMarker(this, next => this.commit(next), id, name, revision)
  }

  world: TopDownState
  snapshot: ResourceSave
  survival: SurvivalState
  hazards: HazardState = freshHazards()
  hazardEvents: HazardEvent[] = []
  environmentDamageHpPerSecond = 0
  private sources: HazardSource[] | null = null
  equipped = false
  activeSourceId: string | null = null
  private miningWork: Record<string, number> = {}
  get progressTicks() { return this.activeSourceId ? (this.miningWork[this.activeSourceId] ?? 0) / 16 : 0 }
  feedback = ''
  error = ''
  private persisted: string | null = null
  private checkpointTick = 0
  private readonly storage: ResourceStorage
  private readonly storeDefinitions: ResourceStoreCatalog
  private readonly now: () => number
  readonly itemCatalog: CountItemCatalog
  constructor(storage: ResourceStorage, world: TopDownState, storeDefinitions: ResourceStoreCatalog = SCOUT_RESOURCE_STORES, itemDefinitions: CountItemCatalog = ITEM_CATALOG, now: () => number = () => Math.floor(Date.now())) {
    this.now = now
    this.survival = freshSurvival(now())
    this.itemCatalog = registerCountItems(itemDefinitions)
    this.storeDefinitions = structuredClone(storeDefinitions)
    this.storage = storage
    this.world = world
    this.snapshot = freshResourceSave(world, storeDefinitions, this.itemCatalog, this.survival, this.hazards)
    try {
      validateResourceLedger(this.snapshot, RESOURCE_SOURCES, this.storeDefinitions, RESOURCE_CATALOG, this.itemCatalog)
      this.persisted = storage.getItem(RESOURCE_SAVE_KEY)
      if (this.persisted !== null) {
        this.snapshot = decodeResourceSave(this.persisted, world, this.storeDefinitions, this.itemCatalog)
        this.survival = this.snapshot.survival
        this.miningWork = { ...this.snapshot.miningWork }
        this.hazards = this.snapshot.hazards
        const restored = decodeScout(serializeForDecode(this.snapshot.scout))
        Object.assign(world.scoutLocation, restored)
        world = { ...world, frames: this.snapshot.location.anchor.chart ? walkingFrames(restored.frames,surfaceRegion(restored.surface,restored.surface.definition),this.snapshot.location.anchor.chart) : restored.frames, tick: Math.max(restored.ship.tick, this.snapshot.core.tick) }
        const location = this.snapshot.location
        this.world = materializeSite(synchronizeDoor({ ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: location.anchor } : e),
          camera: { ...world.camera, anchor: location.anchor }, player: { facing: location.facing, mode: 'standing', targetId: null }, feedback: 'Saved expedition restored. Gathering is stopped.' }), this.snapshot.sites)
      }
      if (this.persisted !== null && JSON.parse(this.persisted).version >= 6) {
        const recovered = recoverOffline(this.survival, now())
        if (recovered.fatigue !== this.survival.fatigue) this.commit({ ...this.snapshot, survival: recovered })
        this.survival = this.snapshot.survival
      } else if (this.persisted !== null) {
        this.survival = freshSurvival(now())
        this.snapshot = { ...this.snapshot, hazards: this.hazards, survival: this.survival }
      }
      this.syncDeposits()
    } catch (error) {
      this.error = `Saved expedition could not be loaded safely (${(error as Error).message}). Existing data kept; export the save before recovery. Gathering paused.`
    }
  }
  private depositKey = ''
  private deposits: readonly PlanetDeposit[] = []
  /** Unmodified baselines remain disposable. Only worked sources enter saves. */
  source(id: string): ResourceSource | undefined {
    return this.snapshot.sources[id] ?? this.deposits.find(p => p.initial.id === id)?.initial
  }
  /** Exact per-source debit journal for the aggregate save adapter (P07). */
  sourceDeltas(): Readonly<Record<string, number>> {
    const definitions = sourceDefinitions(this.snapshot.sources)
    return Object.fromEntries(Object.entries(this.snapshot.sources).map(([id, source]) => [id, definitions[id]!.remainingGrams - source.remainingGrams]).filter(([, grams]) => grams !== 0))
  }
  private syncDeposits(): void {
    const next = synchronizeDeposits(this.world, this.depositKey, this.deposits)
    this.world = next.world
    this.depositKey = next.key
    this.deposits = next.deposits
  }
  private captureSites(sites: SiteStates): SiteStates {
    return captureResourceSites(this.world, sites)
  }
  private commit(next: ResourceSave): boolean {
    if (this.error) return false
    try {
      if (this.storage.getItem(RESOURCE_SAVE_KEY) !== this.persisted) throw new Error('Save changed in another session')
      const sites = this.captureSites(next.sites)
      if (this.world.scoutLocation.binding) validateSurfaceClearance(resolveSurface(this.world.scoutLocation.surface), this.world.entities.filter(e => e.layer === 'terrain'))
      const definitions = sourceDefinitions(next.sources)
      const sources = Object.fromEntries(Object.entries(next.sources).filter(([id, source]) => !isDepositId(id) || source.remainingGrams !== definitions[id]!.remainingGrams || this.miningWork[id] || id === this.activeSourceId).sort(([a], [b]) => a.localeCompare(b)))
      const candidate: ResourceSave = { ...next, sources, journal: validateJournal(recordVisit(next.journal, this.world), sites), sites, siteId: this.world.scoutLocation.siteId, scout: captureScout(this.world.scoutLocation), miningWork: validateMiningWork(this.miningWork, next.sources), survival: { ...next.survival, offlineAtMs: Math.max(next.survival.offlineAtMs, this.now()) }, revision: integer(this.snapshot.revision + 1), location: safeLocation(this.world) }
      validateResourceLedger(candidate, sourceDefinitions(candidate.sources), this.storeDefinitions, RESOURCE_CATALOG, this.itemCatalog)
      validateCoreConservation(candidate.core, candidate.convertedGrams)
      validateSurvival(candidate.survival)
      validateHazards(candidate.hazards)
      // Registered active baselines are useful to the live ledger, but only
      // depleted mass or source-bound partial work belongs in durable storage.
      const durableSources = Object.fromEntries(Object.entries(sources).filter(([id, source]) => !isDepositId(id) || source.remainingGrams !== definitions[id]!.remainingGrams || this.miningWork[id]))
      const raw = serialize({ ...candidate, sources: durableSources })
      // Validate the whole location/site/ship boundary before replacing storage.
      decodeResourceSave(raw, this.world, this.storeDefinitions, this.itemCatalog)
      this.storage.setItem(RESOURCE_SAVE_KEY, raw)
      this.persisted = raw
      this.snapshot = candidate
      this.survival = candidate.survival
      this.hazards = candidate.hazards
      this.checkpointTick = this.world.tick
      return true
    } catch {
      this.activeSourceId = null
      this.miningWork = { ...this.snapshot.miningWork }
      this.survival = this.snapshot.survival
      this.hazards = this.snapshot.hazards
      this.hazardEvents = []
      this.error = 'Saving failed or the save changed elsewhere. Gathering paused; reload after resolving storage access.'
      return false
    }
  }
  scoutCommandDenial(revision = this.snapshot.revision, cockpitSession = this.world.cockpit, actorId = LOCAL_PLAYER, takeoff = true): string | null {
    return resourceScoutCommands.scoutCommandDenial(this, revision, cockpitSession, actorId, takeoff)
  }
  bodyDefinition(bodyId: string) {
    return resourceScoutCommands.bodyDefinition(this, bodyId)
  }
  currentEnvironment(world = this.world) {
    return resourceScoutCommands.currentEnvironment(this, world)
  }
  environmentAt(location: SurfaceLocation) {
    return resourceScoutCommands.environmentAt(this, location)
  }
  environmentSummary(bodyId: string) {
    return resourceScoutCommands.environmentSummary(this, bodyId)
  }
  scoutActionDenial(action: ScoutAction, revision = this.snapshot.revision, cockpitSession = this.world.cockpit, actorId = LOCAL_PLAYER): string | null {
    return resourceScoutCommands.scoutActionDenial(this, action, revision, cockpitSession, actorId)
  }
  landingPreview(bodyId: string): { denial: string | null; surface?: SurfaceLocation } {
    return resourceScoutCommands.landingPreview(this, bodyId)
  }
  transitionScout(action: ScoutAction, revision: number, cockpitSession: CockpitState, actorId = LOCAL_PLAYER): boolean {
    return resourceScoutCommands.transitionScout(this, next => this.commit(next), () => this.syncDeposits(), action, revision, cockpitSession, actorId)
  }
  toggleBoardingDoor(revision: number): boolean {
    return resourceScoutCommands.toggleBoardingDoor(this, next => this.commit(next), revision)
  }
  /** Future environment adapter boundary. Sources are sampled, never restored as live planet data. */
  /** Domain adapter for #920 UI. Revalidate physical admission and carry/storage capacity at publication. */
  changeEquipment(command: GearCommand, revision: number): boolean {
    return resourceInventoryCommands.changeEquipment(this, next => this.commit(next), command, revision)
  }
  setHazardSources(sources: readonly HazardSource[]): void {
    hazardRates(sources)
    this.sources = structuredClone([...sources])
  }
  /** Explicit module declaration boundary; production has no caller or default equipment. */
  addProtection(module: Omit<ProtectionModule, 'remaining'>): boolean {
    return resourceInventoryCommands.addProtection(this, next => this.commit(next), module)
  }
  /** Domain rejection leaves the current save usable; storage failure blocks writes. */
  transfer(request: ResourceTransfer): boolean {
    return resourceInventoryCommands.transfer(this, next => this.commit(next), this.storeDefinitions, request)
  }
  depositBenchInput(resourceId:string, grams:number, revision:number): boolean {
    return resourceMachineCommands.depositBenchInput(this, resourceId, grams, revision)
  }
  withdrawBenchInput(resourceId:string, grams:number, revision:number): boolean {
    return resourceMachineCommands.withdrawBenchInput(this, resourceId, grams, revision)
  }
  startBench(recipeId:string, revision:number): boolean {
    return resourceMachineCommands.startBench(this, next => this.commit(next), recipeId, revision)
  }
  startSuitRepair(suitId:string, amount:string, revision:number): boolean {
    return resourceMachineCommands.startSuitRepair(this, next => this.commit(next), suitId, amount, revision)
  }
  claimBench(jobId:number, revision:number): boolean {
    return resourceMachineCommands.claimBench(this, next => this.commit(next), jobId, revision)
  }
  depositFabricatorInput(resourceId: string, grams: number, revision: number): boolean {
    return resourceMachineCommands.depositFabricatorInput(this, resourceId, grams, revision)
  }
  withdrawFabricatorInput(resourceId: string, grams: number, revision: number): boolean {
    return resourceMachineCommands.withdrawFabricatorInput(this, resourceId, grams, revision)
  }
  startFabricator(recipeId: string, revision: number): boolean {
    return resourceMachineCommands.startFabricator(this, next => this.commit(next), recipeId, revision)
  }
  claimFabricator(jobId: number, revision: number): boolean {
    return resourceMachineCommands.claimFabricator(this, next => this.commit(next), jobId, revision)
  }
  /** Generic adapter boundary for future item producers/consumers. No auto-equip or player grants. */
  changeItem(action: 'add' | 'remove', id: string, count: number, revision: number): boolean {
    return resourceInventoryCommands.changeItem(this, next => this.commit(next), action, id, count, revision)
  }
  /** Explicit exact grams, with the same atomic save and stale-revision guard as mining. */
  deposit(resourceId: string, grams: number, revision: number): boolean {
    return this.transfer({ revision, from: { kind: 'player' }, to: { kind: 'store', id: SCOUT_RESOURCE_CONTAINER_ID }, resourceId, grams })
  }
  withdraw(resourceId: string, grams: number, revision: number): boolean {
    return this.transfer({ revision, from: { kind: 'store', id: SCOUT_RESOURCE_CONTAINER_ID }, to: { kind: 'player' }, resourceId, grams })
  }
  /** Transfer only unconverted Core fuel through player cargo and the shared revision gate. */
  depositCoreFuel(resourceId: string, grams: number, revision: number): boolean {
    return this.transfer({ revision, from: { kind: 'player' }, to: { kind: 'store', id: SCOUT_CORE_FUEL_ID }, resourceId, grams })
  }
  withdrawCoreFuel(resourceId: string, grams: number, revision: number): boolean {
    return this.transfer({ revision, from: { kind: 'store', id: SCOUT_CORE_FUEL_ID }, to: { kind: 'player' }, resourceId, grams })
  }
  /** Explicit terminal-only fresh character. The world ledger and depletion survive. */
  freshRun(): boolean {
    return resourceInventoryCommands.freshRun(this, next => this.commit(next), () => this.now())
  }
  payload() {
    return resourceLedgerMass(this.snapshot, this.world.frames.ship.id,
      this.world.entities.find(e => e.id === PLAYER_ID)!.anchor.frame === this.world.frames.ship.id, RESOURCE_CATALOG, this.itemCatalog)
  }
  checkpoint(): void {
    if (serialize(this.captureSites(this.snapshot.sites)) !== serialize(this.snapshot.sites) || serialize(captureScout(this.world.scoutLocation)) !== serialize(this.snapshot.scout) || serialize(this.miningWork) !== serialize(this.snapshot.miningWork) || serialize(safeLocation(this.world)) !== serialize(this.snapshot.location) || serialize(this.survival) !== serialize(this.snapshot.survival) || serialize(this.hazards) !== serialize(this.snapshot.hazards)) this.commit({ ...this.snapshot, hazards: this.hazards, survival: this.survival })
  }
  upgrade(capacityGrams: number, capacitySlots: number): boolean {
    return resourceInventoryCommands.upgrade(this, next => this.commit(next), capacityGrams, capacitySlots)
  }
  cancel(message = 'Gathering stopped. Completed grams are saved.'): void {
    const wasGathering = this.activeSourceId !== null
    if (wasGathering) this.feedback = message
    this.activeSourceId = null
    if (wasGathering) this.checkpoint()
  }
  toggleTool(): void {
    this.cancel()
    this.equipped = !this.equipped
    this.feedback = this.equipped ? 'Gathering tool equipped · approach a source, then Gather [E].' : 'Gathering tool stowed.'
  }
  private blocked(id: string): string {
    const source = this.source(id)
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
    if (!this.world.scoutLocation.binding || !target?.source || !id || this.world.player.mode === 'seated' || this.world.cockpit.view !== 'world' || !interactionInReach(player.anchor, target, this.world.frames)) {
      this.feedback = 'No resource source in reach.'; return
    }
    const initial = this.source(id)
    if (!initial) { this.feedback = 'No resource source in reach.'; return }
    this.activeSourceId = id
    if (!this.snapshot.sources[id] && !this.commit({ ...this.snapshot, survival: this.survival, hazards: this.hazards, sources: { ...this.snapshot.sources, [id]: initial } })) return
    const blocked = this.blocked(id)
    if (blocked) { this.activeSourceId = null; this.feedback = blocked; return }
    this.activeSourceId = id
    this.world = { ...this.world, selectedId: id, player: { ...this.world.player, mode: 'interacting', targetId: id } }
    this.feedback = 'Gathering. Move or Escape to stop; partial work saved.'
  }
  /** One consumer receipt per committed world tick; no UI-owned energy timer. */
  corePowerTick: number | null = null
  corePower: CorePowerResult = { status: 'idle', consumer: 'idle', drawnJ60: 0, generatedJ: 0, convertedGrams: {} }
  private readonly tickControl: ResourceTickControl = {
    sources: () => this.sources,
    miningWork: () => this.miningWork,
    checkpointTick: () => this.checkpointTick,
    checkpointAt: tick => { this.checkpointTick = tick },
    commit: next => this.commit(next),
    syncDeposits: () => this.syncDeposits(),
    blocked: id => this.blocked(id),
  }
  advance(next: TopDownState, power: CorePowerRequest = { watts: 0 }): TopDownState {
    return advanceResourceSession(this, this.tickControl, next, power)
  }
}