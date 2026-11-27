export const DEFAULT_VOLUMES = {
  master: 100,
  music: 35,
  environment: 75,
  machinery: 65,
  character: 80,
  interface: 50,
  warnings: 85,
} as const

export type AudioChannel = keyof typeof DEFAULT_VOLUMES
type Channel = Exclude<AudioChannel, 'master'>
export type AudioPreferences = {
  readonly volumes: Readonly<Record<AudioChannel, number>>
  readonly mono: boolean
}
export const AUDIO_PREFERENCES_KEY = 'salimon:audio:v1'
type AudioStorage = Pick<Storage, 'getItem' | 'setItem'>

export function readAudioPreferences(storage: AudioStorage): AudioPreferences {
  const defaults = { volumes: { ...DEFAULT_VOLUMES }, mono: false }
  try {
    const json = JSON.parse(storage.getItem(AUDIO_PREFERENCES_KEY) ?? 'null')
    if (!json || typeof json !== 'object') return defaults
    const volumes = { ...DEFAULT_VOLUMES } as Record<AudioChannel, number>
    for (const key of Object.keys(volumes) as AudioChannel[]) {
      const value = json.volumes?.[key]
      if (typeof value === 'number' && Number.isFinite(value)) volumes[key] = Math.round(Math.max(0, Math.min(100, value)))
    }
    return { volumes, mono: json.mono === true }
  } catch {
    return defaults
  }
}

export function storeAudioPreferences(storage: AudioStorage, preferences: AudioPreferences) {
  try {
    storage.setItem(AUDIO_PREFERENCES_KEY, JSON.stringify(preferences))
  } catch {
    // Presentation settings still apply for this document if storage is denied.
  }
}

export const CUES = {
  conditionWarning: { channel: 'warnings', instances: 1, cooldownMs: 1500, loop: false },
  boardingDoor: { channel: 'machinery', instances: 1, cooldownMs: 150, loop: false },
} as const
export type CueId = keyof typeof CUES
export interface SoundPosition {
  readonly rightM: number
  readonly distanceM: number
  /** 0 is unobstructed and 1 is maximally obstructed. */
  readonly obstruction?: number
}

export function spatialMix(position?: SoundPosition) {
  if (!position) return { gain: 1, pan: 0, lowpassHz: 20_000 }
  const distance = Math.max(0, position.distanceM)
  const obstruction = Math.min(1, Math.max(0, position.obstruction ?? 0))
  return {
    gain: (1 - obstruction * .7) / (1 + (distance / 4) ** 2),
    pan: Math.min(1, Math.max(-1, position.rightM / Math.max(2, distance))),
    lowpassHz: 20_000 * (1 - obstruction) + 750 * obstruction,
  }
}

type Mix = ReturnType<typeof spatialMix>
export interface Voice {
  stop(fadeSeconds: number): void
}
export interface AudioBackend<Buffer> {
  unlock(): Promise<void>
  setHidden(hidden: boolean): void
  setPreferences(preferences: AudioPreferences): void
  load(id: CueId): Promise<Buffer>
  start(buffer: Buffer, channel: Channel, mix: Mix, loop: boolean, ended: () => void): Voice
  dispose(): void
}

type Scope = { active: boolean; hidden: boolean; view: string }
type Instance = { id: CueId; voice?: Voice }
export class AudioManager<Buffer> {
  private scope: Scope = { active: false, hidden: false, view: '' }
  private readonly instances = new Set<Instance>()
  private readonly lastPlayed = new Map<CueId, number>()
  private unlocked = false
  private unlocking?: Promise<void>
  private disposed = false

  private readonly backend: AudioBackend<Buffer>
  private preferences: AudioPreferences
  private readonly now: () => number

  constructor(backend: AudioBackend<Buffer>, preferences: AudioPreferences, now: () => number) {
    this.backend = backend
    this.preferences = preferences
    this.now = now
    backend.setPreferences(preferences)
  }

  activeCueIds(): readonly CueId[] {
    return [...new Set([...this.instances].filter(instance => instance.voice).map(instance => instance.id))]
  }

  setPreferences(preferences: AudioPreferences) {
    if (this.disposed) return
    this.preferences = preferences
    this.backend.setPreferences(preferences)
  }

  async unlock() {
    if (this.disposed || this.scope.hidden || this.unlocked) return
    this.unlocking ??= this.backend.unlock().then(() => {
      if (this.disposed) return
      this.unlocked = true
      this.backend.setHidden(this.scope.hidden)
    }).catch(() => {
      // Another trusted interaction can retry a failed browser unlock.
    }).finally(() => { this.unlocking = undefined })
    await this.unlocking
  }

  setScope(scope: Scope) {
    if (this.disposed) return
    const before = this.scope
    this.scope = scope
    if (before.active !== scope.active || before.hidden !== scope.hidden || before.view !== scope.view) {
      this.stopAll(scope.hidden || scope.view === 'ended' ? 0 : .12)
    }
    if (before.hidden !== scope.hidden && this.unlocked) this.backend.setHidden(scope.hidden)
  }

  private audible(id: CueId) {
    return !this.disposed && this.unlocked && this.scope.active && !this.scope.hidden &&
      this.preferences.volumes.master > 0 && this.preferences.volumes[CUES[id].channel] > 0
  }

  async play(id: CueId, position?: SoundPosition): Promise<boolean> {
    if (!this.audible(id)) return false
    const cue = CUES[id]
    if (this.instances.size >= 8 || [...this.instances].filter(instance => instance.id === id).length >= cue.instances ||
      this.now() - (this.lastPlayed.get(id) ?? -Infinity) < cue.cooldownMs) return false
    const instance: Instance = { id }
    this.instances.add(instance)
    try {
      const buffer = await this.backend.load(id)
      if (!this.instances.has(instance) || !this.audible(id)) {
        this.instances.delete(instance)
        return false
      }
      instance.voice = this.backend.start(buffer, cue.channel, spatialMix(position), cue.loop, () => this.instances.delete(instance))
      this.lastPlayed.set(id, this.now())
      return true
    } catch {
      this.instances.delete(instance)
      return false
    }
  }

  private stopAll(fadeSeconds: number) {
    for (const instance of this.instances) instance.voice?.stop(fadeSeconds)
    this.instances.clear()
    this.lastPlayed.clear()
  }

  dispose() {
    if (this.disposed) return
    this.disposed = true
    this.stopAll(0)
    this.backend.dispose()
  }
}
