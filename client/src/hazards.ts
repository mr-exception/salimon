import * as q from './exact.ts'
import { healthValue, setHealth, type SurvivalState } from './survival.ts'
export const HAZARD_TYPES = ['temperature', 'radiation', 'poison'] as const
export type HazardType = typeof HAZARD_TYPES[number]
export type HazardSource = { id: string; type: 'temperature'; temperatureC: number } | { id: string; type: 'radiation' | 'poison'; intensity: number }
/** Explicit declarations only. No suit, mass, recipe, slot count or world strength defaults. */
export interface ProtectionModule { id: string; type: HazardType; protection: number; remaining: string }
export interface HazardState { version: 1; exposure: { radiation: string; poison: string }; modules: ProtectionModule[] }
export interface HazardEvent { type: HazardType; damage: string; blocked: string; destroyed: string[] }
export const freshHazards = (): HazardState => ({ version: 1, exposure: { radiation: '0/1', poison: '0/1' }, modules: [] })
const bound = (s: unknown, limit: number) => { const v = q.decode(s); if (q.cmp(v, q.ZERO) < 0 || q.cmp(v, q.decimal(limit)) > 0) throw Error('Exact value out of range'); return v }
const idValid = (id: unknown) => typeof id === 'string' && /^[a-zA-Z0-9:_-]{1,128}$/.test(id)
const rateValid = (n: unknown) => typeof n === 'number' && Number.isFinite(n) && n >= 0 && n <= Number.MAX_SAFE_INTEGER
export function validateHazards(value: unknown): HazardState {
  if (!value || typeof value !== 'object') throw Error('Invalid hazards')
  const s = value as HazardState
  if (Object.keys(s).sort().join() !== 'exposure,modules,version' || s.version !== 1 || !s.exposure || Object.keys(s.exposure).sort().join() !== 'poison,radiation' || !Array.isArray(s.modules)) throw Error('Invalid hazard metadata')
  bound(s.exposure.poison, 100); bound(s.exposure.radiation, 100)
  const ids = new Set()
  for (const m of s.modules) {
    if (!m || Object.keys(m).sort().join() !== 'id,protection,remaining,type' || !idValid(m.id) || ids.has(m.id) || !HAZARD_TYPES.includes(m.type) || !rateValid(m.protection)) throw Error('Invalid module')
    ids.add(m.id); bound(m.remaining, 1000)
  }
  return structuredClone(s)
}
export function hazardRates(sources: readonly HazardSource[]): Record<HazardType, q.Exact> {
  const result = { temperature: q.ZERO, radiation: q.ZERO, poison: q.ZERO }, ids = new Set()
  for (const s of sources) {
    if (!s || !idValid(s.id) || ids.has(s.id)) throw Error('Invalid source identity')
    ids.add(s.id)
    if (s.type === 'temperature') {
      if (Object.keys(s).sort().join() !== 'id,temperatureC,type' || !Number.isFinite(s.temperatureC) || Math.abs(s.temperatureC) > Number.MAX_SAFE_INTEGER) throw Error('Invalid temperature')
      const difference = q.sub(q.decimal(s.temperatureC), q.decimal(24))
      result.temperature = q.add(result.temperature, q.div(difference[0] < 0n ? q.neg(difference) : difference, q.decimal(10)))
    } else if ((s.type === 'radiation' || s.type === 'poison') && Object.keys(s).sort().join() === 'id,intensity,type' && rateValid(s.intensity)) result[s.type] = q.add(result[s.type], q.decimal(s.intensity))
    else throw Error('Unsupported hazard source')
  }
  return result
}
/** One authoritative 60 Hz step. Split exactly at budget exhaustion and exposure threshold.
 * Multiple modules allocate blocked intensity in stable ID order; no slot/install policy implied.
 */
export function stepHazards(state: HazardState, rates: Record<HazardType, q.Exact>, survival: SurvivalState): { state: HazardState; events: HazardEvent[] } {
  const next = structuredClone(state), events: HazardEvent[] = []
  let totalDamage = q.ZERO
  for (const type of HAZARD_TYPES) {
    const incoming = rates[type], modules = next.modules.filter(m => m.type === type).sort((a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0)
    let remainingTime: q.Exact = [1n, 60n], damage = q.ZERO, blocked = q.ZERO
    const destroyed: string[] = []
    while (q.cmp(remainingTime, q.ZERO) > 0) {
      let available = incoming, duration = remainingTime
      const allocations = modules.map(m => {
        const rate = q.cmp(q.decode(m.remaining), q.ZERO) > 0 ? q.min(available, q.decimal(m.protection)) : q.ZERO
        available = q.sub(available, rate)
        if (q.cmp(rate, q.ZERO) > 0) duration = q.min(duration, q.div(q.decode(m.remaining), rate))
        return { m, rate }
      })
      if (type === 'temperature') damage = q.add(damage, q.mul(available, duration))
      else {
        const exposure = q.decode(next.exposure[type])
        if (q.cmp(incoming, q.ZERO) > 0) {
          const buildup = q.mul(available, duration), capacity = q.sub(q.decimal(100), exposure)
          // Only the part of the interval after reaching 100 deals damage.
          damage = q.add(damage, q.max(q.ZERO, q.sub(buildup, capacity)))
          next.exposure[type] = q.encode(q.min(q.decimal(100), q.add(exposure, buildup)))
        } else {
          const recovery = modules.reduce((v, m) => q.cmp(q.decode(m.remaining), q.ZERO) > 0 ? q.add(v, q.decimal(m.protection)) : v, q.decimal(5))
          next.exposure[type] = q.encode(q.max(q.ZERO, q.sub(exposure, q.mul(recovery, duration))))
        }
      }
      for (const { m, rate } of allocations) {
        if (q.cmp(rate, q.ZERO) === 0) continue
        const absorbed = q.mul(rate, duration)
        m.remaining = q.encode(q.sub(q.decode(m.remaining), absorbed)); blocked = q.add(blocked, absorbed)
        if (m.remaining === '0/1') destroyed.push(m.id)
      }
      remainingTime = q.sub(remainingTime, duration)
    }
    totalDamage = q.add(totalDamage, damage)
    events.push({ type, damage: q.encode(damage), blocked: q.encode(blocked), destroyed })
  }
  setHealth(survival, q.sub(healthValue(survival), totalDamage))
  return { state: next, events }
}
