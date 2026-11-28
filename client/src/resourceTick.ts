import * as q from './exact.ts'
import { benchTickCost, stepBench, type BenchState } from './gearBench.ts'
import { functioningEquipment, GEAR_DEFINITIONS, validateEquipment } from './equipment.ts'
import { FABRICATOR_WATTS, stepFabricator } from './scoutFabricator.ts'
import { updateWalkingFix } from './exploration.ts'
import { hazardRates, stepHazards, type HazardSource } from './hazards.ts'
import { advanceSurvival, survivalPolicy } from './survival.ts'
import { SCOUT_CORE_FUEL_ID, stepCoreEnergy, type CorePowerRequest } from './scoutEnergyCore.ts'
import { leaveCockpit } from './cockpit.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { synchronizeScout, PLAYER_ID, type TopDownState } from './topDownRuntime.ts'
import { RESOURCE_CATALOG, type ResourceSave } from './resourceCatalog.ts'
import { materializeSite } from './resourceSaveLocation.ts'
import type { ResourceSession } from './resourceSession.ts'

/** Compute conversion and consumer charges before advancing any world owner. */
export function stepResourceMachines(snapshot: ResourceSave, elapsed: number, tick: number, power: CorePowerRequest) {
    // Reject malformed consumer commands before advancing any session owner.
    if(power.paused !== undefined && typeof power.paused !== 'boolean')throw Error('Invalid Core pause')
    // H01 order: convert once, then pay Fabricator before bench from one reserve.
    const machine = elapsed > 0 && tick > snapshot.core.tick ? stepCoreEnergy(snapshot.core,snapshot.stores[SCOUT_CORE_FUEL_ID]?.quantities??{},{watts:0}) : null
    let fabricatorPaid=0,benchPaid=0,otherPaid=0
    if(machine){
      let reserve=machine.core.reserveJ60
      const pay=(watts:number,paused=false)=>{if(!Number.isSafeInteger(watts)||watts<0||watts>machine.core.capacityJ*60)throw Error('Invalid Core power request');if(paused||watts>reserve)return 0;reserve-=watts;return watts}
      if(snapshot.fabricator.job)fabricatorPaid=pay(FABRICATOR_WATTS)
      if(snapshot.bench.job)benchPaid=pay(benchTickCost(snapshot.bench.job))
      if(!snapshot.fabricator.job&&!snapshot.bench.job)otherPaid=pay(power.watts,power.paused)
      const drawn=fabricatorPaid+benchPaid+otherPaid
      machine.core={...machine.core,tick:tick,reserveJ60:reserve,spentJ60:machine.core.spentJ60+drawn}
      machine.result={...machine.result,drawnJ60:drawn,consumer:drawn?'powered':snapshot.fabricator.job||snapshot.bench.job||power.watts?'insufficient-power':'idle'}
    }
    const fabricator=machine?stepFabricator(snapshot.fabricator,fabricatorPaid+otherPaid):snapshot.fabricator
    const bench=machine?stepBench(snapshot.bench,benchPaid):snapshot.bench
    return { machine, fabricator, bench }
}

/** Apply exposure and equipment wear on the session's existing sole clock. */
export function advanceResourceHazards(session: Pick<ResourceSession, 'snapshot' | 'survival' | 'hazards' | 'hazardEvents'>, sources: readonly HazardSource[] | null, environment: ReturnType<ResourceSession['currentEnvironment']>, elapsed: number, bench: BenchState) {
    const live = sources === null
    const rates = hazardRates(sources ?? [
      { id: 'planet:temperature', type: 'temperature', temperatureC: environment.temperatureC },
      { id: 'planet:radiation', type: 'radiation', intensity: environment.radiationHpPerSecond },
      { id: 'planet:poison', type: 'poison', intensity: environment.toxicityHpPerSecond },
    ])
    const equipment = session.snapshot.inventory.equipment!
    const modules = live ? functioningEquipment(equipment).map(g => ({ id: g.id, type: GEAR_DEFINITIONS[g.definitionId]!.hazard!, protection: GEAR_DEFINITIONS[g.definitionId]!.protection, remaining: g.remaining })) : session.hazards.modules
    const needsHazards = Object.values(rates).some(v => v[0] > 0n) || Object.values(session.hazards.exposure).some(v => v !== '0/1')
    let worn = modules
    session.hazardEvents = []
    session.survival = advanceSurvival(session.survival, elapsed, needsHazards ? survival => {
      const result = stepHazards({ ...session.hazards, modules: worn }, rates, survival)
      worn = result.state.modules
      session.hazards = live ? { ...result.state, modules: session.hazards.modules } : result.state
      session.hazardEvents.push(...result.events)
      return result.events.some(e => e.damage !== '0/1')
    } : undefined)
    let inventory = live ? { ...session.snapshot.inventory, equipment: { ...equipment, instances: equipment.instances.map(g => {
      const changed = worn.find(m => m.id === g.id)
      return changed ? { ...g, remaining: changed.remaining, location: changed.remaining === '0/1' ? { kind: 'destroyed' as const } : g.location } : g
    }) } } : session.snapshot.inventory
    if(bench.tray?.repair&&!session.snapshot.bench.tray){
      const repair=bench.tray.repair
      inventory={...inventory,equipment:validateEquipment({...inventory.equipment!,instances:inventory.equipment!.instances.map(g=>g.id===repair.suitId?{...g,remaining:q.encode(q.add(q.decode(repair.initial),q.decode(repair.amount)))}:g)})}
    }
    return { inventory, needsHazards }
}
/** Live access to the writer's private gathering/checkpoint state. */
export interface ResourceTickControl {
  sources(): readonly HazardSource[] | null
  miningWork(): Record<string, number>
  checkpointTick(): number
  checkpointAt(tick: number): void
  commit(next: ResourceSave): boolean
  syncDeposits(): void
  blocked(id: string): string
}

/** Sole-clock ordering, rollback, gathering and checkpoint orchestration. */
export function advanceResourceSession(session: Pick<ResourceSession, 'activeSourceId' | 'advance' | 'cancel' | 'checkpoint' | 'corePower' | 'corePowerTick' | 'currentEnvironment' | 'environmentDamageHpPerSecond' | 'error' | 'feedback' | 'hazardEvents' | 'hazards' | 'itemCatalog' | 'snapshot' | 'survival' | 'transfer' | 'walkingFix' | 'world'>, control: ResourceTickControl, next: TopDownState, power: CorePowerRequest): TopDownState {
  session.corePowerTick = null
  session.corePower = { status: 'idle', consumer: 'idle', drawnJ60: 0, generatedJ: 0, convertedGrams: {} }

  if (!Number.isSafeInteger(next.tick) || next.tick < session.world.tick) return session.world
  const elapsed = next.tick - session.world.tick, wasWalking = session.world.player.mode === 'walking'
  if (session.error || session.survival.health === 0) return session.world
  // The same per-tick ordering applies to batched callers and foreground RAF.
  if (elapsed > 1) {
    for (let tick = session.world.tick + 1; tick <= next.tick && !session.error && session.survival.health > 0; tick++) session.advance({ ...next, tick }, power)
    return session.world
  }
  const { machine, fabricator, bench } = stepResourceMachines(session.snapshot, elapsed, next.tick, power)
  const workUnits = survivalPolicy(session.survival).miningUnits
  const previousWorld = session.world
  const environment = session.currentEnvironment(next)
  const { inventory, needsHazards } = advanceResourceHazards(session, control.sources(), environment, elapsed, bench)
  session.world = (next.frames.surface?.source ?? next.frames) === next.scoutLocation.frames ? next
    : next.frames.ship === next.scoutLocation.frames.ship ? { ...next, frames: next.scoutLocation.frames }
    : materializeSite(synchronizeScout(next), session.snapshot.sites)
  session.walkingFix = updateWalkingFix(previousWorld, session.world, session.walkingFix)
  control.syncDeposits()
  // A verified touchdown returns the pilot to the existing ship-local interior.
  // A power interruption keeps its seated/world semantics instead.
  const touchdown = previousWorld.frames.ship.parent === null && session.world.scoutLocation.binding !== null
  if (touchdown && session.world.cockpit.view === 'radar') {
    session.world = leaveCockpit(session.world)
    session.feedback = 'Touchdown verified. Open the aft door to explore this surface site.'
  }
  if (touchdown && !machine) {
    session.checkpoint()
    if (session.error) { session.world = previousWorld; return session.world }
  }
  // Publish hazard progress only after the whole health/exposure/budget transaction is durable.
  const machineChanged = machine && (machine.core.spentJ60 !== session.snapshot.core.spentJ60 || machine.core.reserveJ60 !== session.snapshot.core.reserveJ60 || machine.core.conversionBudgetJ60 !== session.snapshot.core.conversionBudgetJ60)
  if (elapsed > 0 && (needsHazards || machineChanged || touchdown || JSON.stringify(bench) !== JSON.stringify(session.snapshot.bench) || JSON.stringify(fabricator) !== JSON.stringify(session.snapshot.fabricator))) {
    const convertedGrams = { ...session.snapshot.convertedGrams }
    for (const [id, grams] of Object.entries(machine?.result.convertedGrams ?? {})) convertedGrams[id] = (convertedGrams[id] ?? 0) + grams
    const fuel = session.snapshot.stores[SCOUT_CORE_FUEL_ID]
    const machines = machine ? { core: machine.core, convertedGrams, ...(fuel ? { stores: { ...session.snapshot.stores, [SCOUT_CORE_FUEL_ID]: { ...fuel, quantities: machine.quantities } } } : {}) } : {}
    if (!control.commit({ ...session.snapshot, ...machines, fabricator, bench, inventory, survival: session.survival, hazards: session.hazards })) { session.world = previousWorld; return session.world }
  }
  if(elapsed>0)session.environmentDamageHpPerSecond=q.number(session.hazardEvents.reduce((total,e)=>q.add(total,q.mul(q.decode(e.damage),q.decimal(60))),q.ZERO))
  if (machine) { session.corePower = machine.result; session.corePowerTick = next.tick }
  if (!survivalPolicy(session.survival).canWork) {
    session.cancel('Exhausted: gathering and piloting unavailable.')
    if (session.world.player.mode === 'seated') session.world = leaveCockpit(session.world)
  }
  if (session.survival.health === 0) { session.world = { ...session.world, player: { ...session.world.player, mode: 'standing', targetId: null } }; session.checkpoint(); return session.world }
  const id = session.activeSourceId
  if (id) {
    const target = next.entities.find(e => e.id === id), player = next.entities.find(e => e.id === PLAYER_ID)!
    if (!target || next.player.mode !== 'interacting' || next.player.targetId !== id || next.cockpit.view !== 'world' || !interactionInReach(player.anchor, target, next.frames)) session.cancel('Gathering interrupted. Partial work saved for this source.')
    else {
      if (elapsed > 0) control.miningWork()[id] = (control.miningWork()[id] ?? 0) + elapsed * workUnits
      while (session.activeSourceId && (control.miningWork()[id] ?? 0) >= 960) {
        const source = session.snapshot.sources[id]!, inventory = session.snapshot.inventory
        const used = resourceInventoryUsage(inventory, RESOURCE_CATALOG, session.itemCatalog)
        const grams = inventory.quantities[source.resourceId] || used.slots < inventory.capacitySlots
          ? Math.min(1000, source.remainingGrams, inventory.capacityGrams - used.grams) : 0
        // Debit work before the same atomic replacement that credits grams.
        control.miningWork()[id] -= 960
        if (!control.miningWork()[id]) delete control.miningWork()[id]
        if (grams && session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id }, to: { kind: 'player' }, resourceId: source.resourceId, grams })) {
          session.feedback = `Gathered ${grams / 1000} kg · ${RESOURCE_CATALOG[source.resourceId as keyof typeof RESOURCE_CATALOG].name}. Saved.`
        } else if (grams) session.cancel('Gathering transaction rejected.')
        const blocked = control.blocked(id)
        if (blocked) { session.cancel(); session.feedback = blocked }
      }
    }
  }
  if (wasWalking && next.player.mode !== 'walking' || next.tick - control.checkpointTick() >= 60) {
    session.checkpoint()
    control.checkpointAt(next.tick)
  }
  return session.world
}
