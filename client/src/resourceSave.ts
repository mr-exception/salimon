import { BENCH_INPUT_ID, freshBench, validateBench, benchSpentJ60 } from './gearBench.ts'
import { ENVIRONMENT_PUBLICATION, validateEnvironmentPublication } from './planetEnvironment.ts'
import { freshEquipment, validateEquipment } from './equipment.ts'
import { validateDeployedMarkers } from './fieldMarker.ts'
import { FABRICATOR_INPUT_ID, MARKER_KIT_ID, freshFabricator, validateFabricator, fabricatorSpentJ60 } from './scoutFabricator.ts'
import { emptyJournal, validateJournal } from './expeditionJournal.ts'
import { DEPOSIT_PUBLICATION, depositFromId, isDepositId } from './planetDeposits.ts'
import { captureSite, resolveSurface, siteEntities, type SiteStates } from './surfaceSite.ts'
import { captureScout, decodeScout } from './scoutSave.ts'
import { surfaceScoutLocation, startingScoutLocation } from './scoutLocation.ts'
import { freshHazards, validateHazards, type HazardState } from './hazards.ts'
import { freshSurvival, validateSurvival, type SurvivalState } from './survival.ts'
import { emptyResourceStores, validateResourceLedger, type ResourceLedger, type ResourceStoreCatalog } from './resourceLedger.ts'
import { SCOUT_RESOURCE_CONTAINER_ID, SCOUT_RESOURCE_STORES } from './scoutResourceContainer.ts'
import { SCOUT_CORE_FUEL_ID, freshCoreEnergy, validateCoreEnergy, migrateCoreEnergy, validateCoreConservation } from './scoutEnergyCore.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { registerCountItems, type CountItemCatalog, resourceInventoryUsage, startingResourceInventory, type ResourceInventory, type ResourceSource } from './resourceInventory.ts'
import { type TopDownState } from './topDownRuntime.ts'
import { RESOURCE_CATALOG_VERSION, RESOURCE_CATALOG, ITEM_CATALOG, RESOURCE_SOURCES, type ResourceSave } from './resourceCatalog.ts'
import { integer, object, parseLocation, decodeSites, materializeSite, validateMiningWork, sourceDefinitions, safeLocation } from './resourceSaveLocation.ts'

// The existing v1 localStorage key is retained for safe one-time save migration.
const LEGACY_CATALOG_VERSION = 'landing-resources:v1'

const LEGACY_GRAMS_PER_LITRE = { mineral: 2500, biomass: 500 } as const

const startingLocationForMigration = () => {
  const { definition: _reference, ...address } = startingScoutLocation().surface
  return surfaceScoutLocation(address)
}

/** Reject inconsistent saves, and migrate whole-litre v1 saves to exact grams without loss. */
export function decodeResourceSave(raw: string, world: TopDownState, storeDefinitions: ResourceStoreCatalog = SCOUT_RESOURCE_STORES, itemDefinitions: CountItemCatalog = ITEM_CATALOG): ResourceSave {
  const catalog = registerCountItems(itemDefinitions)
  const data = object(JSON.parse(raw))
  const version = data.version
  if (typeof version !== 'number' || !Number.isInteger(version) || version < 1 || version > 19) throw new Error('Unsupported save version')
  const legacy = version === 1
  const hasStores = version >= 3
  const hasItems = version >= 5
  const hasSurvival = version >= 6
  const hasHazards = version >= 7
  const hasWork = version >= 8
  const hasScout = version >= 9
  const hasSites = version >= 11
  const hasJournal = version >= 13
  const hasCore = version >= 14
  const allowed = [...(version >= 19 ? ['bench'] : []),...(version >= 17 ? ['deployedMarkers'] : []), ...(version >= 16 ? ['fabricator'] : []), ...(version >= 15 ? ['convertedGrams'] : []), ...(hasJournal ? ['journal'] : []), ...(hasCore ? ['core'] : []), 'environmentPublication', 'depositPublication', ...(hasSites ? ['sites'] : []), ...(hasScout ? ['scout'] : []), 'version', 'catalogVersion', 'siteId', 'revision', 'inventory', 'sources', 'location', ...(hasStores ? ['stores'] : []), ...(hasItems ? ['itemDefinitions'] : []), ...(hasSurvival ? ['survival'] : []), ...(hasHazards ? ['hazards'] : []), ...(hasWork ? ['miningWork'] : [])]
  if (Object.keys(data).some(key => !allowed.includes(key))) throw new Error('Unknown save metadata')
  if (data.environmentPublication !== undefined) validateEnvironmentPublication(data.environmentPublication)
  if (data.depositPublication !== undefined && data.depositPublication !== DEPOSIT_PUBLICATION) throw Error('Unsupported resource publication')
  if (data.catalogVersion !== (legacy ? LEGACY_CATALOG_VERSION : RESOURCE_CATALOG_VERSION) || (hasSites ? typeof data.siteId !== 'string' : data.siteId !== STARTING_SITE.id)) {
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
    for (const [id, count] of Object.entries(object(inv.items))) {
      if (!Object.hasOwn(catalog, id) || (version < 16 && id === MARKER_KIT_ID)) throw new Error('Unknown item')
      items[id] = integer(count, 1)
    }
  }
  const inventory: ResourceInventory = {
    capacityGrams: integer(inv.capacityGrams, 10_000),
    capacitySlots: legacy ? 12 : integer(inv.capacitySlots, 12),
    quantities, items, equipment: version >= 18 ? validateEquipment(inv.equipment) : freshEquipment(),
  }
  if (version < 18 && inv.equipment !== undefined) throw Error('Equipment requires schema v18')
  if (hasItems) {
    const saved = registerCountItems(object(data.itemDefinitions) as CountItemCatalog)
    if (Object.keys(saved).length !== Object.keys(catalog).filter(id => version >= 16 || id !== MARKER_KIT_ID).length || Object.entries(saved).some(([id, meta]) =>
      !Object.hasOwn(catalog, id) || (version < 16 && id === MARKER_KIT_ID) || meta.gramsPerItem !== catalog[id]!.gramsPerItem || meta.stackSize !== catalog[id]!.stackSize || meta.name !== catalog[id]!.name)) throw new Error('Changed item definitions')
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
  if (Object.keys(savedSources).some(isDepositId) && data.depositPublication !== DEPOSIT_PUBLICATION) throw Error('Missing resource publication')
  const definitions = sourceDefinitions(savedSources as unknown as Record<string, ResourceSource>)
  if (Object.keys(savedSources).length !== Object.keys(definitions).length) throw new Error('Wrong source set')
  for (const [id, initial] of Object.entries(definitions)) {
    const source = object(savedSources[id])
    const previous = integer(legacy ? source.remainingLitres : source.remainingGrams)
    const remainingGrams = legacy ? integer(previous * LEGACY_GRAMS_PER_LITRE[initial.resourceId as keyof typeof LEGACY_GRAMS_PER_LITRE]) : previous
    if (source.id !== id || source.resourceId !== initial.resourceId) throw new Error('Material conservation failed')
    sources[id] = { id, resourceId: initial.resourceId, remainingGrams }
  }
  const savedStores = hasStores ? object(data.stores) : null
  // Only the newly introduced Scout store may be absent in v3. v4 requires it.
  const stores = savedStores ? { ...savedStores } as Record<string, ResourceLedger['stores'][string]> : emptyResourceStores(storeDefinitions)
  // A legacy save can acquire the new store, but cannot already declare it.
  if (!hasCore && savedStores && Object.hasOwn(savedStores, SCOUT_CORE_FUEL_ID)) throw new Error('Core fuel requires schema v14')
  if (!hasCore && savedStores && Object.hasOwn(storeDefinitions, SCOUT_CORE_FUEL_ID)) {
    stores[SCOUT_CORE_FUEL_ID] = emptyResourceStores({ [SCOUT_CORE_FUEL_ID]: storeDefinitions[SCOUT_CORE_FUEL_ID]! })[SCOUT_CORE_FUEL_ID]!
  }
  if (version === 3 && Object.hasOwn(storeDefinitions, SCOUT_RESOURCE_CONTAINER_ID) && !Object.hasOwn(stores, SCOUT_RESOURCE_CONTAINER_ID)) {
    stores[SCOUT_RESOURCE_CONTAINER_ID] = emptyResourceStores({ [SCOUT_RESOURCE_CONTAINER_ID]: storeDefinitions[SCOUT_RESOURCE_CONTAINER_ID]! })[SCOUT_RESOURCE_CONTAINER_ID]!
  }
  if (version < 16 && savedStores) {
    if (Object.hasOwn(savedStores, FABRICATOR_INPUT_ID)) throw Error('Fabricator input requires schema v16')
    if (Object.hasOwn(storeDefinitions, FABRICATOR_INPUT_ID)) stores[FABRICATOR_INPUT_ID] = emptyResourceStores({ [FABRICATOR_INPUT_ID]: storeDefinitions[FABRICATOR_INPUT_ID]! })[FABRICATOR_INPUT_ID]!
  }
  if(version < 19 && savedStores){
    if(Object.hasOwn(savedStores,BENCH_INPUT_ID))throw Error('Bench input requires schema v19')
    if(Object.hasOwn(storeDefinitions,BENCH_INPUT_ID))stores[BENCH_INPUT_ID]=emptyResourceStores({[BENCH_INPUT_ID]:storeDefinitions[BENCH_INPUT_ID]!})[BENCH_INPUT_ID]!
  }
  const miningWork = hasWork ? validateMiningWork(data.miningWork, sources) : {}
  const scout = hasScout ? decodeScout(data.scout, version === 9) : startingLocationForMigration()
  if (!hasSites && scout.siteId !== STARTING_SITE.id) throw Error('Legacy save cannot bind a generated site')
  if (hasSites && !object(data.scout).surface) throw Error('Missing saved surface mapping')
  const initialSite = resolveSurface(startingLocationForMigration().surface)
  const sites: SiteStates = hasSites ? decodeSites(data.sites) : { [STARTING_SITE.id]: captureSite(initialSite, siteEntities(initialSite)) }
  for (const id of Object.keys(sources).filter(isDepositId)) {
    const chart = depositFromId(id).entity.anchor.chart!
    if (!Object.values(sites).some(site => site.address.bodyId === chart.bodyId && site.address.definition?.digest === chart.definition.digest)) throw Error('Deposit body/publication was not visited')
  }
  if (hasSites && (data.siteId !== scout.siteId || scout.binding && !Object.hasOwn(sites, scout.siteId))) throw Error('Missing active site state')
  const restoredWorld = materializeSite({ ...world, scoutLocation: scout, frames: scout.frames }, sites)
  if (version < 12 && object(object(data.location).anchor).chart !== undefined) throw Error('Walking chart requires schema v12')
  const location = parseLocation(data.location, restoredWorld)
  if (!scout.binding && location.anchor.frame !== 'ship:scout') throw new Error('Airborne player outside Scout')
  const core = version >= 15 ? validateCoreEnergy(data.core) : hasCore ? migrateCoreEnergy(data.core) : freshCoreEnergy()
  const save: ResourceSave = { version: 19, bench: version >= 19 ? validateBench(data.bench) : freshBench(), environmentPublication: ENVIRONMENT_PUBLICATION, deployedMarkers: version >= 17 ? validateDeployedMarkers(data.deployedMarkers, sites) : [], fabricator: version >= 16 ? validateFabricator(data.fabricator) : freshFabricator(core.spentJ60), convertedGrams: version >= 15 ? object(data.convertedGrams) as Record<string, number> : {}, core, journal: hasJournal ? validateJournal(data.journal, sites) : emptyJournal(), depositPublication: DEPOSIT_PUBLICATION, sites, scout: captureScout(scout), miningWork, hazards: hasHazards ? validateHazards(data.hazards) : freshHazards(), survival: hasSurvival ? validateSurvival(data.survival) : freshSurvival(), itemDefinitions: catalog, catalogVersion: RESOURCE_CATALOG_VERSION, siteId: scout.siteId, revision: integer(data.revision), inventory, sources, stores, location }
  validateResourceLedger(save, definitions, storeDefinitions, RESOURCE_CATALOG, catalog)
  validateCoreConservation(save.core, save.convertedGrams)
  if (BigInt(fabricatorSpentJ60(save.fabricator)) + BigInt(save.fabricator.otherSpentJ60) + BigInt(benchSpentJ60(save.bench)) !== BigInt(save.core.spentJ60)) throw Error('Unpaid Fabricator work')
  return save
}
/** Fresh state shares the same trusted catalog and atomic save contract as restores. */
export function freshResourceSave(world: TopDownState, storeDefinitions: ResourceStoreCatalog, itemCatalog: CountItemCatalog, survival: SurvivalState, hazards: HazardState): ResourceSave {
  return { version: 19, bench: freshBench(), environmentPublication: ENVIRONMENT_PUBLICATION, deployedMarkers: [], fabricator: freshFabricator(), convertedGrams: {}, core: freshCoreEnergy(), journal: emptyJournal(), depositPublication: DEPOSIT_PUBLICATION, sites: { [STARTING_SITE.id]: captureSite(resolveSurface(startingScoutLocation().surface), siteEntities(resolveSurface(startingScoutLocation().surface))), ...(world.scoutLocation.binding ? { [world.scoutLocation.siteId]: captureSite(resolveSurface(world.scoutLocation.surface), world.entities) } : {}) }, scout: captureScout(world.scoutLocation), miningWork: {}, hazards: hazards, survival: survival, itemDefinitions: itemCatalog, stores: emptyResourceStores(storeDefinitions), catalogVersion: RESOURCE_CATALOG_VERSION, siteId: world.scoutLocation.siteId, revision: 0, inventory: { ...startingResourceInventory(), equipment: freshEquipment() }, sources: RESOURCE_SOURCES, location: safeLocation(world) }
}
