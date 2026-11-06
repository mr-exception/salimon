import * as exact from './exact.ts'
/** Integer 60 Hz lattice: one point = 38,880 units. No wall clock in active stepping. */
export const METER = 3_888_000
export const MAX_HEALTH = 200
export interface SurvivalState {
  hunger: number
  fatigue: number
  health: number
  /** Present only for fractional HP; health is its numerical presentation. */
  healthExact?: string
  activeTicks: number
  starvationTicks: number
  recoveryTicks: number
  offlineAtMs: number
}
export const freshSurvival = (now = 0): SurvivalState => ({ hunger: METER, fatigue: METER, health: MAX_HEALTH, activeTicks: 0, starvationTicks: 0, recoveryTicks: 0, offlineAtMs: now })
export function validateSurvival(value: unknown): SurvivalState {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid survival')
  const s = value as SurvivalState
  const limits = { hunger: METER, fatigue: METER, health: MAX_HEALTH, activeTicks: Number.MAX_SAFE_INTEGER, starvationTicks: 599, recoveryTicks: 3599, offlineAtMs: Number.MAX_SAFE_INTEGER }
  if (Object.keys(s).length !== Object.keys(limits).length + (s.healthExact === undefined ? 0 : 1)) throw new Error('Unknown survival metadata')
  for (const [key, max] of Object.entries(limits)) {
    const n = s[key as keyof SurvivalState]
    if (key === 'health' && s.healthExact !== undefined) continue
    if (typeof n !== 'number' || !Number.isSafeInteger(n) || n < 0 || n > max) throw new Error('Invalid survival field')
  }
  if (s.healthExact !== undefined) {
    const hp = exact.decode(s.healthExact)
    if (exact.cmp(hp, exact.ZERO) < 0 || exact.cmp(hp, exact.decimal(MAX_HEALTH)) > 0 || hp[1] === 1n || exact.number(hp) !== s.health) throw new Error('Invalid exact health')
  }
  if (s.hunger > 0 && s.starvationTicks !== 0 || (s.hunger < METER * .7 || s.fatigue < METER * .5) && s.recoveryTicks !== 0) throw new Error('Invalid survival timer')
  return { ...s }
}
export function advanceSurvival(state: SurvivalState, ticks: number, damage?: (state: SurvivalState) => void): SurvivalState {
  if (!Number.isSafeInteger(ticks) || ticks < 0) throw new RangeError('Invalid survival ticks')
  const s = validateSurvival(state)
  // A run lasts at most 228,000 ticks without consumables. Terminal time freezes.
  for (let i = 0; i < ticks && s.health > 0; i++) {
    if (!Number.isSafeInteger(s.activeTicks + 1)) throw new RangeError('Survival clock overflow')
    const starving = s.hunger === 0
    s.hunger = Math.max(0, s.hunger - 36)
    s.fatigue = Math.max(0, s.fatigue - 1)
    s.activeTicks++
    if (starving && ++s.starvationTicks === 600) { setHealth(s, exact.sub(healthValue(s), exact.ONE)); s.starvationTicks = 0 }
    if (s.health > 0) damage?.(s)
    if (s.hunger >= METER * .7 && s.fatigue >= METER * .5) {
      if (++s.recoveryTicks === 3600) { if (s.health > 0) setHealth(s, exact.add(healthValue(s), exact.ONE)); s.recoveryTicks = 0 }
    } else s.recoveryTicks = 0
  }
  return s
}
/** Launch only; persist before publishing credit. Rollbacks retain the high watermark. */
export function recoverOffline(state: SurvivalState, now: number): SurvivalState {
  const s = validateSurvival(state)
  if (!Number.isSafeInteger(now) || now < 0) throw new RangeError('Invalid UTC time')
  return { ...s, fatigue: s.health === 0 ? s.fatigue : Math.min(METER, s.fatigue + Math.floor(Math.min(21_600_000, Math.max(0, now - s.offlineAtMs)) * 9 / 50)), offlineAtMs: Math.max(now, s.offlineAtMs) }
}
export function survivalPolicy(s: SurvivalState) {
  const hunger = s.hunger / METER * 100, fatigue = s.fatigue / METER * 100
  const hungerRate = hunger >= 50 ? 1 : hunger >= 20 ? .9 : .75
  const fatigueRate = fatigue > 50 ? 1 : fatigue > 20 ? .85 : fatigue > 0 ? .65 : .3
  return { movement: s.health === 0 ? 0 : hungerRate * fatigueRate, canWork: s.health > 0 && s.fatigue > 0 }
}

export const healthValue = (s: SurvivalState) => s.healthExact === undefined ? exact.decimal(s.health) : exact.decode(s.healthExact)
export function setHealth(s: SurvivalState, hp: exact.Exact): void {
  hp = exact.max(exact.ZERO, exact.min(exact.decimal(MAX_HEALTH), hp))
  s.health = exact.number(hp)
  if (hp[1] === 1n) delete s.healthExact
  else s.healthExact = exact.encode(hp)
}
