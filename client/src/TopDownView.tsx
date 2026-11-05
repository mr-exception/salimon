import { App } from './App'
import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { isCockpitTarget, leaveCockpit, setCockpitPower } from './cockpit.ts'
import { useEffect, useRef, useState } from 'react'
import { acceptsFlightShortcut, FrameTimer, NavigationInput, panAxisForCode } from './input.ts'
import { cancelInteraction, createTopDownState, drawList, LAYERS, PLAYER_ID, requestInteraction, TopDownClock, type TopDownState } from './topDownRuntime.ts'
import { useWorldSprites, WorldSprite } from './topDownSprites.tsx'
import './topDown.css'

// Presentation receives immutable scene state. It has no radar or Three imports.
export function TopDownScene({ state }: { state: TopDownState }) {
  const items = drawList(state)
  const sprites = useWorldSprites(state.entities)
  return <svg className="top-down-scene" viewBox="-600 -320 1200 640" shapeRendering="geometricPrecision" textRendering="geometricPrecision" role="img" aria-label="Strict overhead Scout and planetary test site" data-render-ready={String(sprites.ready)} aria-busy={!sprites.ready}>
    {LAYERS.map(layer => <g key={layer} data-layer={layer}>
      {items.filter(item => item.entity.layer === layer).map(({ entity, positionM: [x, y], angle }) =>
        <g key={entity.id} data-entity={entity.id} data-source-kind={entity.source?.kind} data-source-site={entity.source?.siteId} data-frame={entity.anchor.frame} data-position={entity.anchor.point.offsetM.slice(0, 2).join(',')} data-posture={entity.id === PLAYER_ID ? state.player.mode : undefined} data-facing={entity.id === PLAYER_ID ? state.player.facing : undefined} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`}>
          <WorldSprite entity={entity} />
        </g>)}
      {layer === 'cues' && items.filter(item => item.entity.id === state.selectedId).map(({ entity, positionM: [x, y], angle }) => <g key={entity.id} data-selection={entity.id} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`} fill="none" stroke="#bce9df" strokeWidth=".08"><WorldSprite entity={entity} selected /></g>)}
    </g>)}
  </svg>
}

export function TopDownView({ gameplay = false }: { gameplay?: boolean }) {
  const [navigation] = useState(() => new NavigationRuntime(SAMPLE_BODIES))
  const powerFixture = import.meta.env.DEV && gameplay && new URLSearchParams(location.search).has('powerFixture')
  const [state, setState] = useState(() => createTopDownState(import.meta.env.DEV && new URLSearchParams(location.search).get('view') === 'interior' ? 'interior' : 'exterior', gameplay))
  const current = useRef(state)
  useEffect(() => {
    const input = new NavigationInput(), clock = new TopDownClock(), timer = new FrameTimer(performance.now())
    let raf = 0
    const release = () => { input.clear(); clock.reset(); timer.resume(performance.now()) }
    const clear = () => { release(); current.current = cancelInteraction(current.current); setState(current.current) }
    const keydown = (event: KeyboardEvent) => {
      if (current.current.cockpit.view === 'radar') return
      if (!event.repeat && (event.code === 'Escape' || (event.code === 'KeyE' && acceptsFlightShortcut(event.target)))) {
        event.preventDefault(); input.clear()
        current.current = event.code === 'KeyE' ? requestInteraction(current.current) : current.current.player.mode === 'seated' ? leaveCockpit(current.current) : cancelInteraction(current.current)
        setState(current.current)
      }
      const axis = panAxisForCode(event.code)
      if (axis && acceptsFlightShortcut(event.target)) { event.preventDefault(); input.press(event.code, axis) }
    }
    const keyup = (event: KeyboardEvent) => input.release(event.code)
    const frame = (now: number) => {
      const dt = timer.sample(now, document.hidden)
      const inWorld = current.current.cockpit.view === 'world'
      if (gameplay && inWorld) navigation.advance(dt)
      if (!inWorld) input.clear()
      const controls = input.controls
      current.current = clock.advance(current.current, dt, [Number(controls.right) - Number(controls.left), Number(controls.down) - Number(controls.up)])
      setState(current.current)
      raf = requestAnimationFrame(frame)
    }
    window.addEventListener('keydown', keydown); window.addEventListener('keyup', keyup)
    window.addEventListener('blur', clear); window.addEventListener('resize', clear)
    document.addEventListener('visibilitychange', clear); window.addEventListener('focusin', release)
    raf = requestAnimationFrame(frame)
    return () => {
      cancelAnimationFrame(raf)
      window.removeEventListener('keydown', keydown); window.removeEventListener('keyup', keyup)
      window.removeEventListener('blur', clear); window.removeEventListener('resize', clear)
      document.removeEventListener('visibilitychange', clear); window.removeEventListener('focusin', release)
    }
  }, [])
  const selected = state.entities.find(e => e.id === state.selectedId)
  const update = (next: TopDownState) => { current.current = next; setState(next) }
  const cockpitTarget = gameplay && isCockpitTarget(selected?.id ?? null)
  const interact = () => { current.current = current.current.player.mode === 'interacting' ? cancelInteraction(current.current) : requestInteraction(current.current); setState(current.current) }
  return <>
    {state.cockpit.view === 'radar' && <><App navigation={navigation} /><button className="cockpit-exit" onClick={() => update(leaveCockpit(current.current))}>Exit navigation · Stand up</button></>}
    {powerFixture && <div className="power-fixture"><button onClick={event => { update(setCockpitPower(current.current, !current.current.cockpit.powered)); event.currentTarget.blur() }}>{state.cockpit.powered ? 'Test cockpit power loss' : 'Restore cockpit power'}</button></div>}
    <main data-tick={state.tick} hidden={state.cockpit.view === 'radar'} data-cockpit-view={state.cockpit.view} data-powered={String(state.cockpit.powered)} data-occupant={state.cockpit.occupantId ?? ''} data-ship={gameplay ? JSON.stringify(navigation.clock.state, (_, v) => typeof v === 'bigint' ? v.toString() : v) : undefined} className="top-down-app" data-player-mode={state.player.mode} data-target={state.player.targetId ?? ''}>
    <header><div><p className="top-down-eyebrow">SALIMON / WORLD FOUNDATION</p><h1>Scout · Surface study</h1></div><span className="top-down-tag">STRICT OVERHEAD / 2D</span></header>
    <section className="top-down-stage"><TopDownScene state={state} /><div className="top-down-caption"><span>01 / QUIET LANDING SITE</span><span>SCOUT — INTERIOR CUTAWAY</span></div></section>
    <aside className="top-down-interaction" aria-label="Nearby interaction">
      <div><p data-prompt>{selected?.interaction ? `${selected.interaction.label} · ${cockpitTarget ? 'Navigate' : selected.interaction.action} [E]` : 'No fixture in reach · Move closer'}</p><p role="status" aria-live="polite">{state.feedback}</p></div>
      <button onClick={event => { interact(); event.currentTarget.blur() }} disabled={!selected && state.player.mode !== 'interacting'}>{state.player.mode === 'interacting' ? 'Cancel inspection' : cockpitTarget ? 'Use cockpit' : 'Inspect fixture'}</button>
    </aside>
    {state.player.mode === 'seated' && <button onClick={() => update(leaveCockpit(current.current))}>Stand up</button>}
    <footer><p><strong>W A S D</strong> Walk · <strong>E</strong> Interact · <strong>Esc</strong> Cancel</p><p>{gameplay ? 'Cockpit navigation active · Other systems are placeholders' : 'Console inspections are a test · Systems inactive'}</p></footer>
  </main></>
}
