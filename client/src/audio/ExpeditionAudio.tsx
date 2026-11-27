import { useEffect, useRef, useState } from 'react'
import { AudioManager, DEFAULT_VOLUMES, readAudioPreferences, storeAudioPreferences, type AudioChannel, type SoundPosition } from './audioManager.ts'
import { createWebAudio } from './webAudio.ts'

const storage = {
  getItem: (key: string) => localStorage.getItem(key),
  setItem: (key: string, value: string) => localStorage.setItem(key, value),
}
const channelLabels: Record<AudioChannel, string> = {
  master: 'Master', music: 'Music', environment: 'Environment', machinery: 'Machinery',
  character: 'Character and tools', interface: 'Interface', warnings: 'Warnings',
}

interface ExpeditionAudioState {
  enabled: boolean
  active: boolean
  view: 'world' | 'radar' | 'ended'
  warning: string
  door: 'open' | 'closed'
  doorPosition: () => SoundPosition | undefined
}

export function useExpeditionAudio(state: ExpeditionAudioState) {
  const [preferences, setPreferences] = useState(() => readAudioPreferences(storage))
  const manager = useRef<AudioManager<AudioBuffer> | null>(null)
  const current = useRef(state)
  const previous = useRef(state)
  const pageHidden = useRef(false)
  current.current = state

  useEffect(() => {
    if (!state.enabled) return
    const audio = new AudioManager(createWebAudio(), readAudioPreferences(storage), () => performance.now())
    manager.current = audio
    pageHidden.current = false
    const scope = () => audio.setScope({
      active: current.current.active, hidden: document.hidden || pageHidden.current, view: current.current.view,
    })
    const hidePage = () => { pageHidden.current = true; scope() }
    const showPage = () => { pageHidden.current = false; scope() }
    const unlock = (event: Event) => {
      if (event.isTrusted) void audio.unlock()
    }
    scope()
    window.addEventListener('pointerdown', unlock, true)
    window.addEventListener('keydown', unlock, true)
    document.addEventListener('visibilitychange', scope)
    window.addEventListener('pagehide', hidePage)
    window.addEventListener('pageshow', showPage)
    return () => {
      window.removeEventListener('pointerdown', unlock, true)
      window.removeEventListener('keydown', unlock, true)
      document.removeEventListener('visibilitychange', scope)
      window.removeEventListener('pagehide', hidePage)
      window.removeEventListener('pageshow', showPage)
      audio.dispose()
      manager.current = null
    }
  }, [state.enabled])

  useEffect(() => {
    manager.current?.setPreferences(preferences)
  }, [state.enabled, preferences])

  useEffect(() => {
    const before = previous.current
    const next = current.current
    previous.current = next
    const audio = manager.current
    if (!audio) return
    audio.setScope({ active: next.active, hidden: document.hidden || pageHidden.current, view: next.view })
    if (!before.active || !next.active || before.view !== next.view) return
    if (next.warning && next.warning !== before.warning) void audio.play('conditionWarning')
    if (next.view === 'world' && next.door !== before.door) void audio.play('boardingDoor', next.doorPosition())
  }, [state.enabled, state.active, state.view, state.warning, state.door])

  return {
    preferences,
    activeCueIds: () => manager.current?.activeCueIds() ?? [],
    change(next: typeof preferences) {
      setPreferences(next)
      storeAudioPreferences(storage, next)
    },
  }
}

export function ExpeditionAudioSettings({ audio }: { audio: ReturnType<typeof useExpeditionAudio> }) {
  const { preferences, change } = audio
  return <details>
    <summary>Audio settings</summary>
    {(Object.keys(DEFAULT_VOLUMES) as AudioChannel[]).map(key =>
      <label key={key} className="audio-volume">
        <span>{channelLabels[key]} volume</span>
        <input type="range" min="0" max="100" value={preferences.volumes[key]}
          onChange={event => change({ ...preferences, volumes: { ...preferences.volumes, [key]: event.currentTarget.valueAsNumber } })} />
        <output>{preferences.volumes[key]}%</output>
      </label>)}
    <label><input type="checkbox" checked={preferences.mono}
      onChange={event => change({ ...preferences, mono: event.currentTarget.checked })} /> Mono positional audio</label>
  </details>
}
