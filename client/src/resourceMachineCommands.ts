import { BENCH_ID, BENCH_INPUT_ID, BENCH_WATTS, repairCost, repairGrams, validateRepair, GEAR_RECIPES } from './gearBench.ts'
import { GEAR_DEFINITIONS, validateEquipment } from './equipment.ts'
import { FABRICATOR_INPUT_ID, FABRICATOR_RECIPE_ID, FABRICATOR_GRAMS, FABRICATOR_TICKS, MARKER_KIT_ID } from './scoutFabricator.ts'
import { survivalPolicy } from './survival.ts'
import { addCountItem, resourceInventoryUsage } from './resourceInventory.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { PLAYER_ID } from './topDownRuntime.ts'
import { RESOURCE_CATALOG, type ResourceSave } from './resourceCatalog.ts'
import type { ResourceSession } from './resourceSession.ts'
type CommitResourceSave = (next: ResourceSave) => boolean
type SessionState = Pick<ResourceSession, 'error' | 'feedback' | 'itemCatalog' | 'snapshot' | 'survival' | 'transfer' | 'world'>

function benchAdmission(session: SessionState, revision:number): void {
  if(session.error||!survivalPolicy(session.survival).canWork)throw Error(session.error||'Unable to work')
  if(!Number.isSafeInteger(revision)||revision!==session.snapshot.revision)throw Error('Stale bench revision')
  const player=session.world.entities.find(e=>e.id===PLAYER_ID), target=session.world.entities.find(e=>e.id===BENCH_ID)
  if(!player||!target||player.anchor.frame!=='ship:scout'||session.world.cockpit.view!=='world'||session.world.player.mode==='seated'||!interactionInReach(player.anchor,target,session.world.frames))throw Error('Approach the Gear Crafting Bench inside Scout')
}

export function depositBenchInput(session: SessionState, resourceId:string, grams:number, revision:number): boolean {
  try{benchAdmission(session, revision);return session.transfer({revision,from:{kind:'player'},to:{kind:'store',id:BENCH_INPUT_ID},resourceId,grams})}catch(e){session.feedback=(e as Error).message;return false}
}

export function withdrawBenchInput(session: SessionState, resourceId:string, grams:number, revision:number): boolean {
  try{benchAdmission(session, revision);return session.transfer({revision,from:{kind:'store',id:BENCH_INPUT_ID},to:{kind:'player'},resourceId,grams})}catch(e){session.feedback=(e as Error).message;return false}
}

export function startBench(session: SessionState, commit: CommitResourceSave, recipeId:string, revision:number): boolean {
  try{
    benchAdmission(session, revision)
    const b=session.snapshot.bench,r=Object.hasOwn(GEAR_RECIPES,recipeId)?GEAR_RECIPES[recipeId]:null,input=session.snapshot.stores[BENCH_INPUT_ID]!
    if(!r)throw Error('Unknown bench recipe')
    if(b.job||b.tray)throw Error('Bench job or output tray occupied')
    const quantities={...input.quantities}
    for(const id of ['mineral','biomass'] as const){if((quantities[id]??0)<r[id])throw Error('Insufficient bench ingredients');const n=(quantities[id]??0)-r[id];if(n)quantities[id]=n;else delete quantities[id]}
    return commit({...session.snapshot,stores:{...session.snapshot.stores,[BENCH_INPUT_ID]:{...input,quantities}},bench:{...b,started:b.started+1,job:{id:b.started+1,recipeId,remainingTicks:r.ticks,powerPaused:false}}})
  }catch(e){session.feedback=(e as Error).message;return false}
}

export function startSuitRepair(session: SessionState, commit: CommitResourceSave, suitId:string, amount:string, revision:number): boolean {
  try{
    benchAdmission(session, revision)
    const b=session.snapshot.bench,e=session.snapshot.inventory.equipment!,suit=e.instances.find(g=>g.id===suitId)
    if(b.job||b.tray)throw Error('Bench job or output tray occupied')
    if(!suit||GEAR_DEFINITIONS[suit.definitionId]!.kind!=='suit'||suit.location.kind!=='inventory')throw Error('Carry an unequipped suit to the bench')
    const repair={suitId,amount,initial:suit.remaining};validateRepair(repair)
    const input=session.snapshot.stores[BENCH_INPUT_ID]!,grams=repairGrams(repair),quantities={...input.quantities}
    if((quantities.mineral??0)<grams)throw Error('Insufficient unreserved mineral for repair')
    quantities.mineral=(quantities.mineral??0)-grams;if(!quantities.mineral)delete quantities.mineral
    const equipment=validateEquipment({...e,instances:e.instances.map(g=>g.id===suitId?{...g,location:{kind:'bench'}}:g)})
    return commit({...session.snapshot,inventory:{...session.snapshot.inventory,equipment},stores:{...session.snapshot.stores,[BENCH_INPUT_ID]:{...input,quantities}},bench:{...b,started:b.started+1,job:{id:b.started+1,recipeId:'repair',repair,remainingTicks:Math.ceil(repairCost(repair)/BENCH_WATTS),powerPaused:false}}})
  }catch(e){session.feedback=(e as Error).message;return false}
}

export function claimBench(session: SessionState, commit: CommitResourceSave, jobId:number, revision:number): boolean {
  try{
    benchAdmission(session, revision)
    const b=session.snapshot.bench,t=b.tray,e=session.snapshot.inventory.equipment!
    if(!t||jobId!==t.jobId)throw Error('Stale bench claim')
    if(t.repair){
      const inventory={...session.snapshot.inventory,equipment:validateEquipment({...e,instances:e.instances.map(g=>g.id===t.repair!.suitId?{...g,location:{kind:'inventory'}}:g)})}
      resourceInventoryUsage(inventory,RESOURCE_CATALOG,session.itemCatalog)
      return commit({...session.snapshot,inventory,bench:{...b,tray:null,repairs:b.repairs!.map(r=>r.jobId===jobId?{...r,claimed:true}:r)}})
    }
    const gearId=`gear:${e.generation}:bench-${jobId}`
    const inventory={...session.snapshot.inventory,equipment:validateEquipment({...e,instances:[...e.instances,{id:gearId,definitionId:t.recipeId,remaining:'1000/1',location:{kind:'inventory'}}]})}
    resourceInventoryUsage(inventory,RESOURCE_CATALOG,session.itemCatalog)
    return commit({...session.snapshot,inventory,bench:{...b,tray:null,claims:[...b.claims,{jobId,gearId,recipeId:t.recipeId}]}})
  }catch(e){session.feedback=(e as Error).message;return false}
}

export function depositFabricatorInput(session: SessionState, resourceId: string, grams: number, revision: number): boolean {
  return session.transfer({ revision, from: { kind: 'player' }, to: { kind: 'store', id: FABRICATOR_INPUT_ID }, resourceId, grams })
}

export function withdrawFabricatorInput(session: SessionState, resourceId: string, grams: number, revision: number): boolean {
  return session.transfer({ revision, from: { kind: 'store', id: FABRICATOR_INPUT_ID }, to: { kind: 'player' }, resourceId, grams })
}

/** Domain API for E06's reachable console. No arbitrary recipe or item grants. */
export function startFabricator(session: SessionState, commit: CommitResourceSave, recipeId: string, revision: number): boolean {
  if (session.error || !survivalPolicy(session.survival).canWork) return false
  try {
    if (revision !== session.snapshot.revision || !Number.isSafeInteger(revision)) throw Error('Stale Fabricator revision')
    const f = session.snapshot.fabricator, input = session.snapshot.stores[FABRICATOR_INPUT_ID]
    const item = session.itemCatalog[MARKER_KIT_ID]
    if (recipeId !== FABRICATOR_RECIPE_ID || !item || item.gramsPerItem !== FABRICATOR_GRAMS || item.stackSize !== 1) throw Error('Unknown Fabricator recipe')
    if (f.job || f.tray) throw Error('Fabricator job or tray occupied')
    if (!input || (input.quantities.mineral ?? 0) < FABRICATOR_GRAMS) throw Error('Insufficient Fabricator ingredients')
    const quantities: Record<string, number> = { ...input.quantities, mineral: input.quantities.mineral! - FABRICATOR_GRAMS }
    if (!quantities.mineral) delete quantities.mineral
    const id = f.started + 1
    return commit({ ...session.snapshot, stores: { ...session.snapshot.stores, [FABRICATOR_INPUT_ID]: { ...input, quantities } }, fabricator: { ...f, started: id, job: { id, recipeId: FABRICATOR_RECIPE_ID, reservedGrams: FABRICATOR_GRAMS, remainingTicks: FABRICATOR_TICKS, powerPaused: false } } })
  } catch (error) { session.feedback = error instanceof Error ? error.message : 'Invalid Fabricator start'; return false }
}

export function claimFabricator(session: SessionState, commit: CommitResourceSave, jobId: number, revision: number): boolean {
  if (session.error || !survivalPolicy(session.survival).canWork) return false
  try {
    const f = session.snapshot.fabricator
    if (!Number.isSafeInteger(revision) || revision !== session.snapshot.revision || !f.tray || f.tray.jobId !== jobId) throw Error('Stale Fabricator claim')
    const inventory = addCountItem(session.snapshot.inventory, MARKER_KIT_ID, 1, RESOURCE_CATALOG, session.itemCatalog)
    return commit({ ...session.snapshot, inventory, fabricator: { ...f, claimed: jobId, tray: null } })
  } catch (error) { session.feedback = error instanceof Error ? error.message : 'Invalid Fabricator claim'; return false }
}
