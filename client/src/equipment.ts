import { cmp, decode, decimal, ZERO } from './exact.ts'

export type GearLocation = { readonly kind: 'inventory' | 'equipped' | 'drawer' | 'module-storage' | 'destroyed' } | { readonly kind: 'installed'; readonly suitId: string; readonly slot: 0 | 1 }
export interface GearInstance { readonly id: string; readonly definitionId: string; readonly remaining: string; readonly location: GearLocation }
export interface EquipmentState { readonly version: 1; readonly generation: number; readonly instances: readonly GearInstance[] }
export interface GearDefinition { readonly kind: 'suit' | 'module'; readonly grams: number; readonly protection: number; readonly hazard?: 'temperature' | 'radiation' | 'poison'; readonly tier?: number }
export const SUIT_ID = 'suit:basic'
export const GEAR_DEFINITIONS: Readonly<Record<string, GearDefinition>> = Object.freeze(Object.fromEntries([
  [SUIT_ID, Object.freeze({ kind: 'suit', grams: 3000, protection: 0 })],
  ...(['temperature', 'radiation', 'poison'] as const).flatMap(hazard => [1, 2, 3].map(tier => [`module:${hazard}:t${tier}`, Object.freeze({ kind: 'module', hazard, tier, grams: tier * 500, protection: [2, 5, 10][tier - 1]! })])),
]))
export function freshEquipment(generation = 0): EquipmentState {
  if (!Number.isSafeInteger(generation) || generation < 0) throw Error('Invalid equipment generation')
  const suitId = `gear:${generation}:starter-suit`
  return { version: 1, generation, instances: [
    { id: suitId, definitionId: SUIT_ID, remaining: '1000/1', location: { kind: 'equipped' } },
    { id: `gear:${generation}:starter-temperature`, definitionId: 'module:temperature:t1', remaining: '1000/1', location: { kind: 'installed', suitId, slot: 0 } },
  ] }
}
function record(value: unknown, keys: string[]): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).sort().join() !== keys.sort().join()) throw Error('Invalid equipment fields')
  return value as Record<string, unknown>
}
/** One authoritative ownership edge per identity. Slot occupancy is derived, never duplicated. */
export function validateEquipment(value: unknown): EquipmentState {
  const state = record(value, ['version', 'generation', 'instances'])
  if (state.version !== 1 || !Number.isSafeInteger(state.generation) || (state.generation as number) < 0 || !Array.isArray(state.instances) || state.instances.length > 4096) throw Error('Invalid equipment state')
  const ids = new Set<string>(), occupied = new Set<string>()
  const instances: GearInstance[] = state.instances.map(raw => {
    const g = record(raw, ['id', 'definitionId', 'remaining', 'location'])
    if (typeof g.id !== 'string' || !/^gear:[0-9]+:[a-zA-Z0-9_-]{1,100}$/.test(g.id) || !g.id.startsWith(`gear:${state.generation}:`) || ids.has(g.id)) throw Error('Invalid or duplicate equipment identity')
    ids.add(g.id)
    if (typeof g.definitionId !== 'string' || !Object.hasOwn(GEAR_DEFINITIONS, g.definitionId)) throw Error('Unknown equipment definition')
    const definition = GEAR_DEFINITIONS[g.definitionId]!, hp = decode(g.remaining)
    if (cmp(hp, ZERO) < 0 || cmp(hp, decimal(1000)) > 0) throw Error('Invalid equipment durability')
    if (!g.location || typeof g.location !== 'object') throw Error('Invalid equipment location')
    const kind = (g.location as GearLocation).kind
    const location = record(g.location, kind === 'installed' ? ['kind', 'suitId', 'slot'] : ['kind'])
    if (definition.kind === 'suit') {
      if (!['inventory', 'equipped', 'drawer'].includes(kind)) throw Error('Invalid suit owner')
      if (kind === 'equipped') { if (occupied.has('equipped')) throw Error('Duplicate equipped suit'); occupied.add('equipped') }
    } else {
      if (!['inventory', 'module-storage', 'installed', 'destroyed'].includes(kind) || (cmp(hp, ZERO) === 0) !== (kind === 'destroyed')) throw Error('Invalid module owner/durability')
      if (kind === 'installed') {
        if (typeof location.suitId !== 'string' || ![0, 1].includes(location.slot as number)) throw Error('Invalid installed slot')
        const key = `${location.suitId}:${location.slot}`
        if (occupied.has(key)) throw Error('Aliased equipment slot')
        occupied.add(key)
      }
    }
    return { id: g.id, definitionId: g.definitionId, remaining: g.remaining as string, location: { ...location } as GearLocation }
  })
  for (const g of instances) if (g.location.kind === 'installed' && !instances.some(s => s.id === (g.location as { suitId: string }).suitId && GEAR_DEFINITIONS[s.definitionId]!.kind === 'suit')) throw Error('Missing parent suit')
  if (instances.filter(g => g.location.kind === 'drawer').length > 4 || instances.filter(g => g.location.kind === 'module-storage').length > 12) throw Error('Equipment storage full')
  return { version: 1, generation: state.generation as number, instances }
}
/** Installed mass follows its suit, without another slot or a second payload credit. */
export function equipmentMass(value: EquipmentState) {
  const state = validateEquipment(value)
  let carriedGrams = 0, carriedSlots = 0, equippedGrams = 0, storedGrams = 0
  for (const gear of state.instances) {
    let owner = gear.location.kind
    if (gear.location.kind === 'installed') owner = state.instances.find(s => s.id === (gear.location as { suitId: string }).suitId)!.location.kind
    const grams = GEAR_DEFINITIONS[gear.definitionId]!.grams
    if (owner === 'inventory') { carriedGrams += grams; if (gear.location.kind === 'inventory') carriedSlots++ }
    else if (owner === 'equipped') equippedGrams += grams
    else if (owner !== 'destroyed') storedGrams += grams
  }
  return { carriedGrams, carriedSlots, equippedGrams, storedGrams }
}
export function functioningEquipment(value: EquipmentState): readonly GearInstance[] {
  const state = validateEquipment(value), suit = state.instances.find(g => g.location.kind === 'equipped')
  return !suit || cmp(decode(suit.remaining), ZERO) === 0 ? [] : state.instances.filter(g => g.location.kind === 'installed' && g.location.suitId === suit.id)
}
