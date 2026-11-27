import { useState } from 'react'
import { QUALITY, readGraphics, storeGraphics, type GraphicsPreferences, type WorldQuality } from './graphicsPreferences.ts'

const storage = {
  getItem: (key: string) => localStorage.getItem(key),
  setItem: (key: string, value: string) => localStorage.setItem(key, value),
}
const presets = ['Low', 'Medium', 'High', 'Custom'] as const
const fields = [
  ['scale', 'World render scale', 50],
  ['particles', 'Particle density', 0],
  ['environment', 'Environmental detail', 0],
] as const

export function useWorldGraphics() {
  const [preferences, setPreferences] = useState(() => readGraphics(storage))
  const quality = preferences.preset === 'Custom' ? preferences.custom : QUALITY[preferences.preset]
  return {
    preferences, quality,
    change(next: GraphicsPreferences) { setPreferences(next); storeGraphics(storage, next) },
  }
}

export function WorldGraphicsSettings({ graphics }: { graphics: ReturnType<typeof useWorldGraphics> }) {
  const { preferences, change } = graphics
  const setCustom = (settings: Partial<WorldQuality>) =>
    change({ preset: 'Custom', custom: { ...preferences.custom, ...settings } })
  return <details className="world-graphics-settings">
    <summary>Graphics settings</summary>
    <label>World quality <select value={preferences.preset}
      onChange={event => {
        const preset = presets.find(value => value === event.currentTarget.value)
        if (preset) change({ ...preferences, preset })
      }}>
      {presets.map(preset => <option key={preset}>{preset}</option>)}
    </select></label>
    {preferences.preset === 'Custom' && <>
      {fields.map(([key, label, min]) => <label key={key} className="audio-volume">
        <span>{label}</span>
        <input type="range" min={min} max="100" value={preferences.custom[key]}
          onChange={event => setCustom({ [key]: event.currentTarget.valueAsNumber })} />
        <output>{preferences.custom[key]}%</output>
      </label>)}
      <label>World lighting <select value={preferences.custom.lighting}
        onChange={event => setCustom({ lighting: event.currentTarget.value as WorldQuality['lighting'] })}>
        <option value="reduced">Reduced</option><option value="standard">Standard</option><option value="full">Full</option>
      </select></label>
      <label>World shadows <select value={preferences.custom.shadows}
        onChange={event => setCustom({ shadows: event.currentTarget.value as WorldQuality['shadows'] })}>
        <option value="contact">Contact</option><option value="dynamic">Dynamic</option>
      </select></label>
      <label>Resolution cap <select value={preferences.custom.dprCap}
        onChange={event => setCustom({ dprCap: Number(event.currentTarget.value) })}>
        <option value={1}>1×</option><option value={1.5}>1.5×</option><option value={2}>2×</option>
      </select></label>
    </>}
  </details>
}
