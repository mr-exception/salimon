/** Integer 60 Hz lattice: one point = 38,880 units. No wall clock in active stepping. */
export const METER = 3_888_000
export const MAX_HEALTH = 200
export interface SurvivalState {
  hunger: number
  fatigue: number
  health: number
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
  if (Object.keys(s).length !== Object.keys(limits).length) throw new Error('Unknown survival metadata')
  for (const [key, max] of Object.entries(limits)) {
    const n = s[key as keyof SurvivalState]
    if (!Number.isSafeInteger(n) || n < 0 || n > max) throw new Error('Invalid survival field')
  }
  if (s.hunger > 0 && s.starvationTicks !== 0 || (s.hunger < METER * .7 || s.fatigue < METER * .5) && s.recoveryTicks !== 0) throw new Error('Invalid survival timer')
  return { ...s }
}
export function advanceSurvival(state: SurvivalState, ticks: number): SurvivalState {
  if (!Number.isSafeInteger(ticks) || ticks < 0) throw new RangeError('Invalid survival ticks')
  const s = validateSurvival(state)
  // A run lasts at most 228,000 ticks without consumables. Terminal time freezes.
  for (let i = 0; i < ticks && s.health > 0; i++) {
    if (!Number.isSafeInteger(s.activeTicks + 1)) throw new RangeError('Survival clock overflow')
    const starving = s.hunger === 0
    s.hunger = Math.max(0, s.hunger - 36)
    s.fatigue = Math.max(0, s.fatigue - 1)
    s.activeTicks++
    if (starving && ++s.starvationTicks === 600) { s.health--; s.starvationTicks = 0 }
    if (s.hunger >= METER * .7 && s.fatigue >= METER * .5) {
      if (++s.recoveryTicks === 3600) { if (s.health > 0) s.health = Math.min(MAX_HEALTH, s.health + 1); s.recoveryTicks = 0 }
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
