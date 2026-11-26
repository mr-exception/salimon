import { functioningEquipment, GEAR_DEFINITIONS, type EquipmentState } from './equipment.ts'
import { hazardRates } from './hazards.ts'
import * as exact from './exact.ts'
import { PLANET_DEFINITIONS } from './planetDefinition.ts'
import type { Body } from './sampleUniverse.ts'
import type { SurfaceLocation } from './surfaceCoordinates.ts'
import type { EnvironmentSample, bodyEnvironmentSummary } from './planetEnvironment.ts'

export interface ScannerEnvironmentSource {
  environmentSummary(bodyId: string): ReturnType<typeof bodyEnvironmentSummary>
  environmentAt(location: SurfaceLocation): EnvironmentSample
  snapshot: { inventory: { equipment?: EquipmentState } }
}

export function assessScannerEnvironment(selected: Body | undefined, source: ScannerEnvironmentSource, surface?: SurfaceLocation, equipment: EquipmentState | undefined = source.snapshot.inventory.equipment) {
  if (!selected || !PLANET_DEFINITIONS.some(planet => planet.bodyId === selected.id)) return
  const summary = source.environmentSummary(selected.id)
  const local = surface?.bodyId === selected.id ? source.environmentAt(surface) : undefined
  const protection = equipment ? functioningEquipment(equipment) : []
  const rates = local && hazardRates([{ id: 'scanner:temperature', type: 'temperature', temperatureC: local.temperatureC }])
  const rows = [
    { label: 'Temperature', type: 'temperature', incoming: rates ? exact.number(rates.temperature) : undefined },
    { label: 'Radiation', type: 'radiation', incoming: local?.radiationHpPerSecond ?? (summary.safeStartingRegionOverride ? undefined : summary.radiationHpPerSecond) },
    { label: 'Toxicity', type: 'poison', incoming: local?.toxicityHpPerSecond ?? (summary.safeStartingRegionOverride ? undefined : summary.toxicityHpPerSecond) },
  ].map(({ label, type, incoming }) => {
    const available = protection.reduce((total, gear) => {
      const definition = GEAR_DEFINITIONS[gear.definitionId]!
      return total + (definition.hazard === type ? definition.protection : 0)
    }, 0)
    const status = incoming === undefined ? 'Site requirement unknown'
      : incoming === 0 ? 'No protection needed'
      : available === 0 ? 'Missing protection'
      : available < incoming ? 'Insufficient protection'
      : 'Protection sufficient'
    return { label, incoming, available, status }
  })
  return { summary, local, rows }
}
