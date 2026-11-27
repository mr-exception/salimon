export const GRAPHICS_KEY = 'salimon:graphics:v1'
export const QUALITY = {
  Low: { scale: 75, particles: 25, environment: 50, lighting: 'reduced', shadows: 'contact', dprCap: 1 },
  Medium: { scale: 100, particles: 60, environment: 75, lighting: 'standard', shadows: 'contact', dprCap: 2 },
  High: { scale: 100, particles: 100, environment: 100, lighting: 'full', shadows: 'dynamic', dprCap: 2 },
} as const
type Preset = keyof typeof QUALITY
export type WorldQuality = {
  readonly scale: number
  readonly particles: number
  readonly environment: number
  readonly lighting: 'reduced' | 'standard' | 'full'
  readonly shadows: 'contact' | 'dynamic'
  readonly dprCap: number
}
export type GraphicsPreferences = {
  readonly preset: Preset | 'Custom'
  readonly custom: WorldQuality
}
type SettingsStorage = Pick<Storage, 'getItem' | 'setItem'>

function ranged(value: unknown, minimum: number, maximum: number, fallback: number) {
  return typeof value === 'number' && Number.isFinite(value) ? Math.min(maximum, Math.max(minimum, value)) : fallback
}

export function readGraphics(storage: SettingsStorage): GraphicsPreferences {
  const defaults: GraphicsPreferences = { preset: 'Medium', custom: QUALITY.Medium }
  try {
    const input = JSON.parse(storage.getItem(GRAPHICS_KEY) ?? 'null')
    if (!input || typeof input !== 'object') return defaults
    const preset = input.preset === 'Low' || input.preset === 'High' || input.preset === 'Custom' ? input.preset : 'Medium'
    const custom = input.custom
    if (!custom || typeof custom !== 'object') return { ...defaults, preset }
    return { preset, custom: {
      scale: ranged(custom.scale, 50, 100, defaults.custom.scale),
      particles: ranged(custom.particles, 0, 100, defaults.custom.particles),
      environment: ranged(custom.environment, 0, 100, defaults.custom.environment),
      lighting: custom.lighting === 'reduced' || custom.lighting === 'full' ? custom.lighting : 'standard',
      shadows: custom.shadows === 'dynamic' ? 'dynamic' : 'contact',
      dprCap: ranged(custom.dprCap, 1, 2, defaults.custom.dprCap),
    } }
  } catch {
    return defaults
  }
}

export function storeGraphics(storage: SettingsStorage, preferences: GraphicsPreferences) {
  try { storage.setItem(GRAPHICS_KEY, JSON.stringify(preferences)) } catch { /* Keep settings in memory. */ }
}

export function renderDpr(deviceDpr: number, quality: WorldQuality) {
  return Math.min(quality.dprCap, deviceDpr) * quality.scale / 100
}
