import { type BenchState } from './gearBench.ts'
import { ENVIRONMENT_PUBLICATION } from './planetEnvironment.ts'
import { FABRICATOR_ITEMS, type FabricatorState } from './scoutFabricator.ts'
import { type ExpeditionJournal } from './expeditionJournal.ts'
import { type FieldMark } from './exploration.ts'
import { DEPOSIT_PUBLICATION } from './planetDeposits.ts'
import { type SiteStates } from './surfaceSite.ts'
import { type ScoutSave } from './scoutSave.ts'
import { type HazardState } from './hazards.ts'
import { type SurvivalState } from './survival.ts'
import { type ResourceLedger } from './resourceLedger.ts'
import { type CoreEnergyState } from './scoutEnergyCore.ts'
import { registerCountItems, type CountItemCatalog, type ResourceInventory, type ResourceSource } from './resourceInventory.ts'
import type { PlayerLocation } from './topDownTraversal.ts'
export const RESOURCE_SAVE_KEY = 'salimon:expedition:v1'

export const RESOURCE_CATALOG_VERSION = 'landing-resources:v2'

export const RESOURCE_CATALOG = {
  mineral: { name: 'Banded mineral' },
  biomass: { name: 'Plate biomass' },
} as const

// Trusted manufactured starter item; deployment is an atomic E07 transaction.
export const ITEM_CATALOG = registerCountItems(FABRICATOR_ITEMS)

export const RESOURCE_SOURCES: Readonly<Record<string, ResourceSource>> = {
  'source:landing:mineral:01': { id: 'source:landing:mineral:01', resourceId: 'mineral', remainingGrams: 60_000 },
  'source:landing:biology:01': { id: 'source:landing:biology:01', resourceId: 'biomass', remainingGrams: 15_000 },
}

export interface ResourceSave extends ResourceLedger {
  readonly environmentPublication: typeof ENVIRONMENT_PUBLICATION
  readonly depositPublication: typeof DEPOSIT_PUBLICATION
  readonly version: 19
  readonly deployedMarkers: readonly FieldMark[]
  readonly bench: BenchState
  readonly fabricator: FabricatorState
  readonly convertedGrams: Readonly<Record<string, number>>
  readonly core: CoreEnergyState
  readonly journal: ExpeditionJournal
  readonly sites: SiteStates
  readonly scout: ScoutSave
  readonly miningWork: Readonly<Record<string, number>>
  readonly hazards: HazardState
  readonly survival: SurvivalState
  readonly itemDefinitions: CountItemCatalog
  readonly catalogVersion: typeof RESOURCE_CATALOG_VERSION
  readonly siteId: string
  readonly revision: number
  readonly inventory: ResourceInventory
  readonly sources: Readonly<Record<string, ResourceSource>>
  readonly location: PlayerLocation
}

export interface ResourceStorage { getItem(key: string): string | null; setItem(key: string, value: string): void }
