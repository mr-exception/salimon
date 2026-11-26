import warning from '../../assets/audio/condition-warning.mp3?no-inline'
import door from '../../assets/audio/boarding-door.mp3?no-inline'
import { DEFAULT_VOLUMES, type AudioBackend, type AudioChannel, type AudioPreferences, type CueId } from './audioManager.ts'

const urls: Record<CueId, string> = { conditionWarning: warning, boardingDoor: door }

export function createWebAudio(): AudioBackend<AudioBuffer> {
  let context: AudioContext | undefined
  const buses = new Map<AudioChannel, GainNode>()
  const panners = new Map<StereoPannerNode, number>()
  const buffers = new Map<CueId, Promise<AudioBuffer>>()
  const fetches = new AbortController()
  let preferences: AudioPreferences = { volumes: DEFAULT_VOLUMES, mono: false }

  function applyPreferences() {
    if (!context) return
    for (const [key, gain] of buses) gain.gain.setTargetAtTime(preferences.volumes[key] / 100, context.currentTime, .025)
    for (const [panner, pan] of panners) panner.pan.setTargetAtTime(preferences.mono ? 0 : pan, context.currentTime, .025)
  }

  return {
    async unlock() {
      if (!context) {
        context = new AudioContext()
        const master = context.createGain()
        master.gain.value = preferences.volumes.master / 100
        master.connect(context.destination)
        buses.set('master', master)
        for (const key of Object.keys(DEFAULT_VOLUMES) as AudioChannel[]) {
          if (key === 'master') continue
          const gain = context.createGain()
          gain.gain.value = preferences.volumes[key] / 100
          gain.connect(master)
          buses.set(key, gain)
        }
      }
      await context.resume()
    },
    setHidden(hidden) {
      if (!context || context.state === 'closed') return
      void (hidden ? context.suspend() : context.resume()).catch(() => {})
    },
    setPreferences(next) {
      preferences = next
      applyPreferences()
    },
    load(id) {
      const current = context!
      const existing = buffers.get(id)
      if (existing) return existing
      const pending = fetch(urls[id], { signal: fetches.signal }).then(response => {
        if (!response.ok) throw new Error(`Audio cue ${id} could not be loaded`)
        return response.arrayBuffer()
      }).then(data => current.decodeAudioData(data)).catch(error => {
        buffers.delete(id)
        throw error
      })
      buffers.set(id, pending)
      return pending
    },
    start(buffer, channel, mix, loop, ended) {
      const current = context!
      const voice = current.createBufferSource()
      voice.buffer = buffer
      voice.loop = loop
      const gain = current.createGain()
      const filter = current.createBiquadFilter()
      filter.type = 'lowpass'
      filter.frequency.value = mix.lowpassHz
      const panner = current.createStereoPanner()
      panners.set(panner, mix.pan)
      panner.pan.value = preferences.mono ? 0 : mix.pan
      voice.connect(filter).connect(panner).connect(gain).connect(buses.get(channel)!)
      const now = current.currentTime
      gain.gain.setValueAtTime(0, now)
      gain.gain.linearRampToValueAtTime(mix.gain, now + .018)
      voice.onended = () => {
        panners.delete(panner)
        voice.disconnect()
        filter.disconnect()
        panner.disconnect()
        gain.disconnect()
        ended()
      }
      voice.start()
      return {
        stop(seconds) {
          const at = current.currentTime
          if (typeof gain.gain.cancelAndHoldAtTime === 'function') gain.gain.cancelAndHoldAtTime(at)
          else gain.gain.cancelScheduledValues(at)
          gain.gain.linearRampToValueAtTime(0, at + seconds)
          voice.stop(at + seconds)
        },
      }
    },
    dispose() {
      fetches.abort()
      buffers.clear()
      panners.clear()
      for (const gain of buses.values()) gain.disconnect()
      buses.clear()
      if (context && context.state !== 'closed') void context.close().catch(() => {})
    },
  }
}
