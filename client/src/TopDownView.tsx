import { useEffect, useRef, useState } from 'react'
import { acceptsFlightShortcut, FrameTimer, NavigationInput, panAxisForCode } from './input.ts'
import { cancelInteraction, createTopDownState, drawList, LAYERS, PLAYER_ID, requestInteraction, TopDownClock, type Appearance, type Entity2D, type TopDownState } from './topDownRuntime.ts'
import './topDown.css'

function Illustration({ appearance, footprint }: { appearance: Appearance; footprint?: Entity2D['footprint'] }) {
  switch (appearance) {
    case 'ground': return <g><rect x="-200" y="-200" width="400" height="400" fill="#b5a58b" /><path d="M-30 15 Q-10 -10 20 -18 M-35 17 Q-15 -5 22 -16 M-18 -26 Q8 -3 28 4 M-20 25 Q-8 12 16 20" fill="none" stroke="#9f927c" strokeWidth=".15" /><path d="M-16 4l2 -.3m8 -13l1 .2m20 6l1 -.5m-8 20l2 .3m-20 2l1 -.4" stroke="#d7cab0" strokeWidth=".25" /></g>
    case 'rock': return <g stroke="#6b6961" strokeWidth=".09"><path d="M-1.7 -.5L-.9 -1.4 .8 -1.2 1.6 .2 .7 1.1 -1.2 .9Z" fill="#827f73" /><path d="M-1.7 -.5L-.3 -.7 .8 -1.2 .5 .2 -1.2 .9M.5 .2L1.6 .2" fill="#aaa493" /></g>
    case 'hull': return <g stroke="#343f43" strokeWidth=".14" strokeLinejoin="round"><path d="M-3 -4.6L-1.8 -7H1.8L3 -4.6V5.5L2.1 6.5H-2.1L-3 5.5Z" fill="#536064" /><path d="M-3 2.5H-3.7V5.8H-3M3 2.5H3.7V5.8H3" fill="#65777b" /><path d="M-1.6 -6.7H1.6" stroke="#78c6cc" strokeWidth=".2" /></g>
    case 'floor': return <g><path d="M-2.7 -4.5L-1.6 -6.5H1.6L2.7 -4.5V5.4L1.9 6.1H-1.9L-2.7 5.4Z" fill="#dfd9c6" /><path d="M-2.7 -2.9H2.7V-4.5L1.6 -6.5H-1.6L-2.7 -4.5ZM-2.7 2.8H2.7V5.4L1.9 6.1H-1.9L-2.7 5.4Z" fill="#747f7d" /><path d="M-2.5 -2.7H-.8M.8 -2.7H2.5M-2.5 2.6H-.8M.8 2.6H2.5" stroke="#b48261" strokeWidth=".16" /><path d="M-2.3 -1.7V1.8M2.3 -1.7V1.8M-2.3 4.8H1" stroke="#c6c0ae" strokeWidth=".06" /><rect x="-1.2" y=".8" width="2.4" height="2" rx=".12" fill="#b88a6e" opacity=".35" /></g>
    case 'player': return <g stroke="#344950" strokeWidth=".07"><ellipse cy=".08" rx=".43" ry=".24" fill="#425962" /><path d="M-.29 .18L-.25 .36M.29 .18L.25 .36" strokeWidth=".16" /><ellipse cy="-.06" rx=".28" ry=".3" fill="#e2d8b9" /><path d="M-.2 -.2Q0 -.35 .2 -.2" stroke="#86d7d7" strokeWidth=".1" /></g>
    case 'desk': return <g stroke="#46575a" strokeWidth=".08"><path d="M-1.5 -.5Q0 -1.2 1.5 -.5V.4H-1.5Z" fill="#414e54" /><rect x="-.85" y="-.55" width="1.7" height=".5" rx=".08" fill="#77bcc2" /><rect x="-.35" y=".6" width=".7" height=".65" rx=".15" fill="#ba8864" /></g>
    case 'bed': return <g stroke="#a59e8d" strokeWidth=".07"><rect x="-.4" y="-1.2" width=".85" height="2.4" rx=".08" fill="#eee8d8" /><rect x="-.3" y="-.6" width=".65" height="1.7" rx=".06" fill="#ac7860" /><rect x="-.3" y="-1.1" width=".65" height=".4" rx=".08" fill="#faf1da" /></g>
    case 'core': return <g stroke="#394b50" strokeWidth=".08"><rect x="-.4" y="-.7" width=".8" height="1.4" rx=".1" fill="#50636a" /><circle r=".25" fill="#77c5c9" /><path d="M-.25 .42H.25" stroke="#c59365" /></g>
    case 'wall': case 'door': return <g><rect x={-footprint!.halfWidth} y={-footprint!.halfHeight} width={2 * footprint!.halfWidth} height={2 * footprint!.halfHeight} fill={appearance === 'door' ? '#ba8864' : '#526466'} stroke="#2c4146" strokeWidth=".04" />{appearance === 'door' && <path d="M-.55 0H.55" stroke="#f2d9ae" strokeWidth=".06" />}</g>
    case 'crate': return <g stroke="#785e4b" strokeWidth=".06"><rect x={-footprint!.halfWidth} y={-footprint!.halfHeight} width={2 * footprint!.halfWidth} height={2 * footprint!.halfHeight} rx=".06" fill="#b58a65" /><path d="M-.5 -.4L.5 .4M-.5 .4L.5 -.4" /></g>
    case 'cue': return <g fill="none" stroke="#bce9df" strokeWidth=".06"><circle r=".28" /><path d="M-.1 0H.1M0 -.1V.1" /></g>
  }
}

// Presentation receives immutable scene state. It has no radar or Three imports.
export function TopDownScene({ state }: { state: TopDownState }) {
  const items = drawList(state)
  return <svg className="top-down-scene" viewBox="-600 -320 1200 640" role="img" aria-label="Strict overhead Scout and planetary test site" data-tick={state.tick}>
    {LAYERS.map(layer => <g key={layer} data-layer={layer}>
      {items.filter(item => item.entity.layer === layer).map(({ entity, positionM: [x, y], angle }) =>
        <g key={entity.id} data-entity={entity.id} data-frame={entity.anchor.frame} data-position={entity.anchor.point.offsetM.slice(0, 2).join(',')} data-facing={entity.id === PLAYER_ID ? state.player.facing : undefined} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`}>
          <Illustration appearance={entity.appearance} footprint={entity.footprint} />
        </g>)}
      {layer === 'cues' && items.filter(item => item.entity.id === state.selectedId).map(({ entity, positionM: [x, y], angle }) => <g key={entity.id} data-selection={entity.id} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`} fill="none" stroke="#bce9df" strokeWidth=".08"><rect x={-(entity.footprint?.halfWidth ?? .4) - .15} y={-(entity.footprint?.halfHeight ?? .4) - .15} width={2 * (entity.footprint?.halfWidth ?? .4) + .3} height={2 * (entity.footprint?.halfHeight ?? .4) + .3} rx=".1" /></g>)}
    </g>)}
  </svg>
}

export function TopDownView() {
  const [state, setState] = useState(createTopDownState)
  const current = useRef(state)
  useEffect(() => {
    const input = new NavigationInput(), clock = new TopDownClock(), timer = new FrameTimer(performance.now())
    let raf = 0
    const release = () => { input.clear(); clock.reset(); timer.resume(performance.now()) }
    const clear = () => { release(); current.current = cancelInteraction(current.current); setState(current.current) }
    const keydown = (event: KeyboardEvent) => {
      if (!event.repeat && (event.code === 'Escape' || (event.code === 'KeyE' && acceptsFlightShortcut(event.target)))) {
        event.preventDefault(); input.clear()
        current.current = event.code === 'KeyE' ? requestInteraction(current.current) : cancelInteraction(current.current)
        setState(current.current)
      }
      const axis = panAxisForCode(event.code)
      if (axis && acceptsFlightShortcut(event.target)) { event.preventDefault(); input.press(event.code, axis) }
    }
    const keyup = (event: KeyboardEvent) => input.release(event.code)
    const frame = (now: number) => {
      const controls = input.controls
      current.current = clock.advance(current.current, timer.sample(now, document.hidden), [Number(controls.right) - Number(controls.left), Number(controls.down) - Number(controls.up)])
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
  const interact = () => { current.current = current.current.player.mode === 'interacting' ? cancelInteraction(current.current) : requestInteraction(current.current); setState(current.current) }
  return <main className="top-down-app" data-player-mode={state.player.mode} data-target={state.player.targetId ?? ''}>
    <header><div><p className="top-down-eyebrow">SALIMON / WORLD FOUNDATION</p><h1>Scout · Surface study</h1></div><span className="top-down-tag">STRICT OVERHEAD / 2D</span></header>
    <section className="top-down-stage"><TopDownScene state={state} /><div className="top-down-caption"><span>01 / QUIET LANDING SITE</span><span>SCOUT — INTERIOR CUTAWAY</span></div></section>
    <aside className="top-down-interaction" aria-label="Nearby interaction">
      <div><p data-prompt>{selected?.interaction ? `${selected.interaction.label} · ${selected.interaction.action} [E]` : 'No console in reach · Move closer'}</p><p role="status" aria-live="polite">{state.feedback}</p></div>
      <button onClick={event => { interact(); event.currentTarget.blur() }} disabled={!selected && state.player.mode !== 'interacting'}>{state.player.mode === 'interacting' ? 'Cancel inspection' : 'Inspect console'}</button>
    </aside>
    <footer><p><strong>W A S D</strong> Walk · <strong>E</strong> Inspect · <strong>Esc</strong> Cancel</p><p>Console inspections are a test · Systems inactive</p></footer>
  </main>
}
