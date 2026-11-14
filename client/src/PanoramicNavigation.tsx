import { Canvas, type RootState } from '@react-three/fiber'
import { useEffect, useRef, useState } from 'react'
import { ObservationCamera } from './ObservationCamera.tsx'
import { ObservationCameraState } from './observationCamera.ts'
import { OpticalSky } from './OpticalSky.tsx'
import { PerspectiveContacts } from './PerspectiveContacts.tsx'
import { OrientationCompass } from './OrientationCompass.tsx'
import { HelpDialog } from './HelpDialog.tsx'
import { FrameTimer, acceptsFlightShortcut, axisForCode } from './input.ts'
import { setContactDirection } from './contactCommands.ts'
import { TAKEOFF_TICKS } from './takeoff.ts'
import { setThrottle } from './simulation.ts'
import { navigationWarnings } from './contact.ts'
import type { NavigationRuntime } from './navigationRuntime.ts'
import type { Body } from './sampleUniverse.ts'
import { useDismissLayer } from './ui/interaction.ts'
import { FeedbackScope, StatusMessage } from './ui/Feedback.tsx'
import './ui/tokens.css'
import './ui/primitives.css'
import './panoramic.css'

/** Reusable presentation. The cockpit lifecycle owner supplies the runtime and
 * local view; no simulation, discovery or view lifetime belongs to the camera.
 */
export function PanoramicNavigation({ navigation, view, opticalBodies = navigation.bodies, canPilot = () => true, isPaused = () => false, onExit, onSceneReady, advanceSimulation = true, conditionWarning = '', takeoff }: {
  takeoff?: { denial: () => string | null; activate: () => void }; conditionWarning?: string; advanceSimulation?: boolean; navigation: NavigationRuntime; view: ObservationCameraState; canPilot?: () => boolean
  opticalBodies?: readonly Body[]
  isPaused?: () => boolean; onExit?: () => void; onSceneReady?: (state: RootState) => void
}) {
  const [, redraw] = useState(0)
  const [browser, setBrowser] = useState(false), [telemetry, setTelemetry] = useState(false), [help, setHelp] = useState(false)
  const root = useRef<HTMLDivElement>(null), contactsButton = useRef<HTMLButtonElement | null>(null)
  const input = useRef(navigation.pilotInput)
  const escape = useRef<(event: KeyboardEvent) => void>(() => {})
  // Read current callbacks without restarting the simulation clock on a render.
  const options = useRef({ canPilot, isPaused })
  options.current = { canPilot, isPaused }
  useEffect(() => {
    let frame = 0, publication = 0
    const timer = new FrameTimer(performance.now())
    const animate = (now: number) => {
      const paused = document.hidden || options.current.isPaused()
      const dt = timer.sample(now, paused)
      if (paused) input.current.clear()
      if (!options.current.canPilot() || navigation.clock.state.mode !== 'flying' || document.querySelector('dialog[open]')) input.current.clear()
      if (advanceSimulation) navigation.advance(dt, input.current.controls)
      publication += dt
      if (publication >= 0.05) { redraw(n => n + 1); publication = 0 }
      frame = requestAnimationFrame(animate)
    }
    frame = requestAnimationFrame(animate)
    const clear = () => input.current.clear()
    const interrupt = () => { clear(); view.cancelLookAt() }
    const down = (e: KeyboardEvent) => {
      if (!acceptsFlightShortcut(e.target) || e.altKey || e.ctrlKey || e.metaKey || document.querySelector('dialog[open]')) return
      const axis = axisForCode(e.code)
      if (axis && options.current.canPilot() && !options.current.isPaused() && navigation.clock.state.mode === 'flying') {
        input.current.press(e.code, axis)
        navigation.clock.state = { ...navigation.clock.state, alignment: null }
        e.preventDefault()
      }
      if (e.code === 'Space' && options.current.canPilot() && !options.current.isPaused() && navigation.clock.state.mode === 'flying') { navigation.clock.state = setThrottle(navigation.clock.state, 0); e.preventDefault() }
      if (e.code === 'KeyC' && !e.repeat) { setBrowser(value => !value); e.preventDefault() }
    }
    const up = (e: KeyboardEvent) => input.current.release(e.code)
    const outsideEscape = (e: KeyboardEvent) => {
      // Pause/resume can leave focus on body or the separate survival HUD.
      if (!e.defaultPrevented && !options.current.isPaused() && !root.current?.contains(e.target as Node)) escape.current(e)
    }
    window.addEventListener('keydown', outsideEscape); window.addEventListener('keydown', down); window.addEventListener('keyup', up)
    window.addEventListener('blur', interrupt); window.addEventListener('resize', interrupt)
    document.addEventListener('focusin', clear); document.addEventListener('visibilitychange', interrupt)
    return () => {
      cancelAnimationFrame(frame); interrupt()
      window.removeEventListener('keydown', outsideEscape); window.removeEventListener('keydown', down); window.removeEventListener('keyup', up)
      window.removeEventListener('blur', interrupt); window.removeEventListener('resize', interrupt)
      document.removeEventListener('focusin', clear); document.removeEventListener('visibilitychange', interrupt)
    }
  }, [navigation, view, advanceSimulation])
  useEffect(() => {
    contactsButton.current = root.current?.querySelector('.contacts-toggle') ?? null
    if (browser) root.current?.querySelector<HTMLInputElement>('#scanned-browser input')?.focus()
  }, [browser])
  const closeBrowser = () => { setBrowser(false); contactsButton.current?.focus() }
  useDismissLayer(browser, closeBrowser)
  useDismissLayer(telemetry, () => { setTelemetry(false); root.current?.querySelector<HTMLButtonElement>('.telemetry-toggle')?.focus() })
  const ship = navigation.clock.state, snapshot = view.snapshot()
  const warnings = navigationWarnings(ship, navigation.scanner.contacts)
  const grounded = navigation.surfaceLocation
  const speed = Math.hypot(...ship.velocityMps)
  const throttle = (value: number) => { if (canPilot() && !isPaused() && ship.mode === 'flying' && !document.querySelector('dialog[open]')) navigation.clock.state = setThrottle(navigation.clock.state, value); redraw(n => n + 1) }
  const dismiss = (e: { key: string; preventDefault(): void; stopPropagation(): void }) => {
    if (e.key !== 'Escape') return
    input.current.clear()
    if ('repeat' in e && e.repeat) { e.preventDefault(); e.stopPropagation(); return }
    if (help) { setHelp(false); root.current?.querySelector<HTMLButtonElement>('.help-toggle')?.focus(); e.preventDefault(); e.stopPropagation() }
    else if (browser) { closeBrowser(); e.preventDefault(); e.stopPropagation() }
    else if (telemetry) { setTelemetry(false); root.current?.querySelector<HTMLButtonElement>('.telemetry-toggle')?.focus(); e.preventDefault(); e.stopPropagation() }
    else if (onExit) { e.preventDefault(); e.stopPropagation(); onExit() }
  }
  escape.current = dismiss
  return <FeedbackScope messages={[
    ...warnings.map(w => ({ id: w.id, text: w.message, priority: 'critical' as const })),
    { id: 'terminal', text: ship.mode === 'destroyed' ? 'Journey ended — ship destroyed' : '', priority: 'critical' },
    { id: 'condition', text: conditionWarning, priority: 'critical' },
  ]}><div ref={root} className="panoramic expedition-ui" data-observation-surface role="region" aria-label="Panoramic navigation" data-observation={JSON.stringify(snapshot)} data-mode={ship.mode} onKeyDown={dismiss}>
    <Canvas onCreated={state => { state.gl.domElement.setAttribute('aria-label', 'Panoramic universe'); state.gl.domElement.tabIndex = 0; state.gl.domElement.focus({ preventScroll: true }); onSceneReady?.(state) }} fallback={<p role="alert">WebGL 2 is required.</p>}>
      <ObservationCamera view={view} isPaused={() => options.current.isPaused()}/><OpticalSky bodies={opticalBodies} getOrigin={() => navigation.clock.state.position}/>
    </Canvas>
    <div className="observation-reticle" aria-hidden="true">+</div>
    <header className="panoramic-status ui-panel ui-panel--utility">
      <strong className="ui-eyebrow">SALIMON · OBSERVATION</strong>
      <StatusMessage>{navigation.scanner.pulse === 0 ? 'Scanner awaiting first 5-second pulse' : `Scanner active · ${navigation.scanner.contacts.length} contacts`}</StatusMessage>
      <span className="scanner-pulse" aria-live="off">Pulse {navigation.scanner.pulse} · </span><span>{ship.mode}{grounded ? ` · ${grounded.body.name}` : ''} · {speed.toFixed(1)} m/s · sample inertial frame</span>
    </header>
    <div className="panoramic-priority" aria-label="Navigation warnings">
      {warnings.map(w => <StatusMessage key={w.id} priority="critical">{w.message}</StatusMessage>)}
      {ship.contact && <StatusMessage priority={ship.mode === 'destroyed' ? 'critical' : 'status'}>{`${ship.mode === 'destroyed' ? 'Journey ended — ship destroyed' : ship.contact.outcome} · impact ${ship.contact.impactSpeedMps.toFixed(1)} m/s`}</StatusMessage>}
      {!grounded && (ship.mode === 'destroyed' || ship.mode === 'grounded') && <button className="ui-button ui-button--secondary" onClick={() => { navigation.reset(); view.reset(navigation.clock.state); redraw(n => n + 1) }}>Restart navigation</button>}
    </div>
    <PerspectiveContacts compact browserOpen={browser} onBrowserChange={setBrowser} scanner={navigation.scanner} geometry={navigation.bodies} view={view}
      getOrigin={() => navigation.clock.state.position} getShip={() => navigation.clock.state} canPilot={canPilot}
      onSetDirection={body => { navigation.clock.state = setContactDirection(navigation.scanner, navigation.clock.state, body.id, canPilot() && !isPaused() && !document.querySelector('dialog[open]')) }}/>
    <div className="panoramic-instruments"><OrientationCompass ship={ship} view={snapshot}/></div>
    <footer className="panoramic-controls">
      {takeoff && <div className="launch-control">
        <button className="ui-button" disabled={!!takeoff.denial()} aria-describedby="takeoff-status" onClick={() => { if (!isPaused() && !document.querySelector('dialog[open]') && !takeoff.denial()) { input.current.clear(); takeoff.activate(); redraw(n => n + 1) } }}>Take off</button>
        <span id="takeoff-status" role="status">{navigation.location?.launch ? `Taking off · ${Math.min(100, Math.floor((ship.tick - navigation.location.launch.startTick) / TAKEOFF_TICKS * 100))}% · automatic launch` : ship.mode === 'flying' ? 'Flying · manual controls available' : takeoff.denial() ?? 'Ready for takeoff'}</span>
      </div>}
      <label>Throttle {ship.throttle}%<input aria-label="Throttle" type="range" min="-100" max="100" value={ship.throttle} disabled={ship.mode !== 'flying' || !canPilot()} onChange={e => throttle(Number(e.target.value))}/></label>
      <button className="ui-button ui-button--secondary" disabled={ship.mode !== 'flying' || !canPilot()} onClick={() => throttle(0)}>Zero throttle (Space)</button>
      <button className="ui-button ui-button--secondary" onClick={() => { view.reset(navigation.clock.state); redraw(n => n + 1) }}>Reset view</button>
      <button className="ui-button ui-button--secondary telemetry-toggle" aria-expanded={telemetry} aria-controls="detailed-telemetry" onClick={() => setTelemetry(value => !value)}>Telemetry</button>
      <button className="ui-button ui-button--secondary help-toggle" onClick={() => setHelp(true)}>Help</button>{onExit && <button className="ui-button ui-button--secondary" onClick={onExit}>Exit cockpit</button>}
    </footer>
    {telemetry && <section data-ui-utility id="detailed-telemetry" className="panoramic-telemetry ui-panel ui-panel--utility" aria-label="Detailed telemetry">
      <p>Mass {ship.massKg.toLocaleString()} kg · FOV {snapshot.fov.toFixed(2)}°</p>
      <p>Yaw {(ship.yaw * 180 / Math.PI).toFixed(1)}° · pitch {(ship.pitch * 180 / Math.PI).toFixed(1)}° · {ship.alignment ? 'Turning' : 'Manual'}</p>
      <p>Cell {ship.position.cell.map(String).join(' / ')} · local {ship.position.offsetM.map(v => v.toFixed(1)).join(' / ')} m</p>
      {grounded && <p>Scout grounded. Close the aft door and use Take off when authorized; observation and scanner remain available.</p>}
      <p>Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run.</p>
    </section>}
    {help && <HelpDialog panoramic onClose={() => setHelp(false)} />}
  </div></FeedbackScope>
}
