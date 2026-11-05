import { App } from './App'
import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { isCockpitTarget, leaveCockpit, setCockpitPower } from './cockpit.ts'
import { useEffect, useRef, useState } from 'react'
import { acceptsFlightShortcut, FrameTimer, NavigationInput, panAxisForCode } from './input.ts'
import { cancelInteraction, createTopDownState, drawList, LAYERS, PLAYER_ID, requestInteraction, TopDownClock, type Appearance, type Entity2D, type TopDownState } from './topDownRuntime.ts'
import './topDown.css'

// The tiny on-crate capacity legend is vector ink, not subpixel-hinted text.
// At exterior zoom its old ~5px font could rasterize differently at identical
// transforms (CI PLANET/TRAVERSE failures). Keep the units in the HUD and label.
function StorageCapacityMark() {
  const glyphs: Record<string, string[]> = {
    '5': ['111','100','111','001','111'], '0': ['111','101','101','101','111'],
    'K': ['101','110','100','110','101'], 'G': ['111','100','101','101','111'],
    '/': ['001','001','010','100','100'], 'L': ['100','100','100','100','111'],
    ' ': ['000','000','000','000','000'],
  }
  const label = '50 KG/50 L'
  const path = [...label].flatMap((letter, index) => glyphs[letter].flatMap((row, y) =>
    [...row].flatMap((ink, x) => ink === '1' ? [`M${index * 4 + x} ${y}h1v1h-1Z`] : []))).join('')
  return <g role="img" aria-label="50 kg / 50 L" transform="translate(-.47 .15) scale(.024 .038)"><path d={path} fill="#283d43" stroke="none" /></g>
}

function Illustration({ appearance, footprint }: { appearance: Appearance; footprint?: Entity2D['footprint'] }) {
  switch (appearance) {
    case 'ground': return <g><rect x="-200" y="-200" width="400" height="400" fill="#b5a58b" />
      <path d="M-28 -12Q-14 -3 -11 12L-17 19 -33 8ZM-4 -18Q12 -8 22 -12L29 -5Q11 0 -4 -8Z" fill="#a69580" />
      <path d="M-27 -10Q-16 -1 -14 12M-26 -8Q-18 0 -16 10M-3 -16Q12 -6 23 -10M-4 -14Q11 -4 24 -8" fill="none" stroke="#c7b69b" strokeWidth=".18" />
      <path d="M-14 5Q-8 1 -2 5T12 7M-12 6Q-7 3 -2 6T11 8M-5 -10Q-4 -5 1 -4M9 -2Q15 0 17 4" fill="none" stroke="#978573" strokeWidth=".09" />
      <path d="M-16 4l2 -.3m8 -13l1 .2m20 6l1 -.5m-8 20l2 .3m-20 2l1 -.4m2 -8l.4 -.1m5 1l.6 .2m-1 6l.3 .2" stroke="#d7cab0" strokeWidth=".2" />
    </g>
    case 'mineral': return <g stroke="#6f5c58" strokeWidth=".07"><path d="M-1.2 -.3L-.7 -.9 .7 -.8 1.2 .3 .5 .9 -1 .7Z" fill="#957d76" /><path d="M-1 -.3L-.5 -.6 .7 -.5M-.9 0L.4 -.1 1 .3M-.8 .4L.4 .3 .8 .5" fill="none" stroke="#e0b37c" strokeWidth=".16" /><path d="M-.7 -.9L-.5 -.2 .7 -.8M-.5 -.2L.5 .9" fill="none" /></g>
    case 'flora': return <g stroke="#62546a" strokeWidth=".06"><ellipse cy=".15" rx=".8" ry=".6" fill="#89717e" /><path d="M0 .6Q-1.3 .1 -.65 -.65Q.05 -.75 0 .6M0 .6Q1.3 .1 .65 -.65Q-.05 -.75 0 .6" fill="#b49da9" /><path d="M0 .55Q-.7 -.35 0 -.85Q.7 -.35 0 .55" fill="#d0b7be" /><path d="M0 .6V-.65M0 .5L-.65 -.4M0 .5L.65 -.4" stroke="#705768" fill="none" /></g>
    case 'stones': return <g fill="#8b8274" stroke="#6b6961" strokeWidth=".06"><path d="M-.5 -.1L-.25 -.4 .15 -.3 .2 .1 -.25 .2Z" /><path d="M.1 .15L.3 -.05 .5 .15 .35 .4 .1 .35Z" /></g>
    case 'rock': return <g stroke="#6b6961" strokeWidth=".09"><path d="M-1.7 -.5L-.9 -1.4 .8 -1.2 1.6 .2 .7 1.1 -1.2 .9Z" fill="#827f73" /><path d="M-1.7 -.5L-.3 -.7 .8 -1.2 .5 .2 -1.2 .9M.5 .2L1.6 .2" fill="#aaa493" /></g>
    case 'hull': return <g stroke="#343f43" strokeWidth=".14" strokeLinejoin="round"><path d="M-3 -4.6L-1.8 -7H1.8L3 -4.6V5.5L2.1 6.5H1.5V5.5H-1.5V6.5H-2.1L-3 5.5Z" fill="#536064" /><path d="M-3 2.5H-3.7V5.8H-3M3 2.5H3.7V5.8H3" fill="#65777b" /><path d="M-1.6 -6.7H1.6" stroke="#78c6cc" strokeWidth=".2" /></g>
    case 'floor': return <g><path d="M-2.7 -4.5L-1.6 -6.5H1.6L2.7 -4.5V5.4L1.9 6.1H-1.9L-2.7 5.4Z" fill="#dfd9c6" /><path d="M-2.7 -2.5H2.7V-4.5L1.6 -6.5H-1.6L-2.7 -4.5ZM-2.7 2.8H2.7V5.4L1.9 6.1H-1.9L-2.7 5.4Z" fill="#747f7d" /><path d="M-2.5 -2.3H-.8M.8 -2.3H2.5M-2.5 2.6H-.8M.8 2.6H2.5" stroke="#b48261" strokeWidth=".16" /><path d="M-2.3 -1.7V1.8M2.3 -1.7V1.8M-2.3 4.8H1" stroke="#c6c0ae" strokeWidth=".06" /><text x="0" y="-5.75" textAnchor="middle" fontSize=".22" fill="#e9e3cf">COCKPIT</text><text x="0" y="1.4" textAnchor="middle" fontSize=".22" fill="#665a49">MAIN CABIN</text><text x="0" y="4.6" textAnchor="middle" fontSize=".22" fill="#efe3cc">CARGO</text><rect x="-1.2" y=".8" width="2.4" height="2" rx=".12" fill="#b88a6e" opacity=".35" /></g>
    case 'player': return <g stroke="#344950" strokeWidth=".07"><ellipse cy=".08" rx=".43" ry=".24" fill="#425962" /><path d="M-.29 .18L-.25 .36M.29 .18L.25 .36" strokeWidth=".16" /><ellipse cy="-.06" rx=".28" ry=".3" fill="#e2d8b9" /><path d="M-.2 -.2Q0 -.35 .2 -.2" stroke="#86d7d7" strokeWidth=".1" /></g>
    case 'desk': return <g stroke="#46575a" strokeWidth=".08"><path d="M-1.5 -.5Q0 -1.2 1.5 -.5V.4H-1.5Z" fill="#414e54" /><rect x="-.85" y="-.55" width="1.7" height=".5" rx=".08" fill="#77bcc2" /></g>
    case 'chair': return <g stroke="#46575a" strokeWidth=".07"><rect x="-.35" y="-.35" width=".7" height=".7" rx=".12" fill="#ba8864" /><path d="M-.27 .2H.27M0 -.25V.15" stroke="#e1c8a2" /></g>
    case 'fabricator': return <g stroke="#394b50" strokeWidth=".08"><rect x="-.6" y="-.65" width="1.2" height="1.3" rx=".08" fill="#607277" /><rect x="-.4" y="-.42" width=".8" height=".55" fill="#283d43" /><path d="M-.35 .35H.35" stroke="#d6a06b" strokeWidth=".14" /><circle cx=".4" cy="-.5" r=".06" fill="#91d1cd" /></g>
    case 'storage': return <g stroke="#6b6558" strokeWidth=".07"><rect x="-.55" y="-.7" width="1.1" height="1.4" rx=".08" fill="#b6ac8c" /><path d="M-.4 -.5V.5M.4 -.5V.5M-.45 0H.45" stroke="#536064" /><StorageCapacityMark /></g>
    case 'threshold': return <g stroke="#d6b183" strokeWidth=".08"><path d="M-1.4 -.1H1.4M-1.4 .3H1.4M-1 .65H1" /><path d="M-.25 .45L0 .15 .25 .45" fill="none" /></g>
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
  return <svg className="top-down-scene" viewBox="-600 -320 1200 640" shapeRendering="geometricPrecision" textRendering="geometricPrecision" role="img" aria-label="Strict overhead Scout and planetary test site" data-tick={state.tick}>
    {LAYERS.map(layer => <g key={layer} data-layer={layer}>
      {items.filter(item => item.entity.layer === layer).map(({ entity, positionM: [x, y], angle }) =>
        <g key={entity.id} data-entity={entity.id} data-source-kind={entity.source?.kind} data-source-site={entity.source?.siteId} data-frame={entity.anchor.frame} data-position={entity.anchor.point.offsetM.slice(0, 2).join(',')} data-posture={entity.id === PLAYER_ID ? state.player.mode : undefined} data-facing={entity.id === PLAYER_ID ? state.player.facing : undefined} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`}>
          <Illustration appearance={entity.appearance} footprint={entity.footprint} />
        </g>)}
      {layer === 'cues' && items.filter(item => item.entity.id === state.selectedId).map(({ entity, positionM: [x, y], angle }) => <g key={entity.id} data-selection={entity.id} transform={`translate(${x * state.camera.pixelsPerM} ${y * state.camera.pixelsPerM}) rotate(${angle * 180 / Math.PI}) scale(${state.camera.pixelsPerM})`} fill="none" stroke="#bce9df" strokeWidth=".08"><rect x={-(entity.footprint?.halfWidth ?? .4) - .15} y={-(entity.footprint?.halfHeight ?? .4) - .15} width={2 * (entity.footprint?.halfWidth ?? .4) + .3} height={2 * (entity.footprint?.halfHeight ?? .4) + .3} rx=".1" /></g>)}
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
    <main hidden={state.cockpit.view === 'radar'} data-cockpit-view={state.cockpit.view} data-powered={String(state.cockpit.powered)} data-occupant={state.cockpit.occupantId ?? ''} data-ship={gameplay ? JSON.stringify(navigation.clock.state, (_, v) => typeof v === 'bigint' ? v.toString() : v) : undefined} className="top-down-app" data-player-mode={state.player.mode} data-target={state.player.targetId ?? ''}>
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
