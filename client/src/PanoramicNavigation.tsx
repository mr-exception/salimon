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
import { landingReadout, landingProgress } from './landingHud.ts'
import { SCANNER_RADIUS_LS } from './sampleUniverse.ts'
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
export function PanoramicNavigation({ navigation, view, opticalBodies = navigation.bodies, canPilot = () => true, isPaused = () => false, onExit, onSceneReady, advanceSimulation = true, conditionWarning = '', takeoff, landing }: {
  landing?: { denial: (bodyId: string) => string | null; activate: (bodyId: string) => void }
  takeoff?: { denial: () => string | null; activate: () => void }; conditionWarning?: string; advanceSimulation?: boolean; navigation: NavigationRuntime; view: ObservationCameraState; canPilot?: () => boolean
  opticalBodies?: readonly Body[]
  isPaused?: () => boolean; onExit?: () => void; onSceneReady?: (state: RootState) => void
}) {
  const [, redraw] = useState(0)
  const [orientationGrid, setOrientationGrid] = useState(false)
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
  const approach = landingReadout(navigation)
  const landingDenial = !canPilot() ? 'An authorized owner must pilot from the cockpit seat.'
    : isPaused() ? 'Resume the expedition before landing.'
    : approach.body ? landing?.denial(approach.body.id) ?? null : 'Scan a target planet or moon before requesting landing.'
  const progress = landingProgress(navigation)
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
    ...warnings.map(w => ({ id: w.id, text: w.message, priority: 'critical' as const, tone: w.severity === 'danger' ? 'danger' as const : 'caution' as const })),
    { id: 'terminal', text: ship.mode === 'destroyed' ? 'Journey ended — ship destroyed' : '', priority: 'critical' },
    { id: 'condition', text: conditionWarning, priority: 'critical' },
  ]}><div ref={root} className="panoramic expedition-ui" data-observation-surface role="region" aria-label="Panoramic navigation" data-observation={JSON.stringify(snapshot)} data-mode={ship.mode} onKeyDown={dismiss}>
    <Canvas className="panoramic-scene" onCreated={state => { state.gl.domElement.setAttribute('aria-label', 'Panoramic universe'); state.gl.domElement.tabIndex = 0; state.gl.domElement.focus({ preventScroll: true }); onSceneReady?.(state) }} fallback={<p role="alert">WebGL 2 is required.</p>}>
      <ObservationCamera view={view} isPaused={() => options.current.isPaused()}/><OpticalSky bodies={opticalBodies} getOrigin={() => navigation.clock.state.position} definitionForBody={navigation.surfaceDefinitionForBody} scannerEnvironment orientationGrid={orientationGrid}/>
    </Canvas>
    <div className="observation-reticle" aria-hidden="true"><svg viewBox="0 0 32 32"><path d="M3 16H10 M22 16H29 M16 3V10 M16 22V29"/><circle cx="16" cy="16" r="1"/></svg></div>
    <header className="panoramic-status ui-panel ui-panel--utility">
      <strong className="ui-eyebrow">SALIMON · OBSERVATION</strong>
      <StatusMessage>{navigation.scanner.pulse === 0 ? 'Scanner awaiting first 5-second pulse' : `Scanner active · ${navigation.scanner.contacts.length} contacts`}</StatusMessage>
      <span className="scanner-pulse" aria-live="off">Pulse {navigation.scanner.pulse} · </span><span>{ship.mode}{grounded ? ` · ${grounded.body.name}` : ''} · sample inertial frame</span>
      <span className="scanner-readings">Range {SCANNER_RADIUS_LS.toLocaleString()} ls · FOV {snapshot.fov.toFixed(1)}°</span>
    </header>
    <nav className="panoramic-utilities" aria-label="Cockpit utilities">
      <button className="ui-button ui-button--secondary help-toggle" onClick={() => setHelp(true)}>Help</button>
      {onExit && <button className="ui-button ui-button--secondary" onClick={onExit}>Exit cockpit</button>}
    </nav>
    <div className="panoramic-priority" aria-label="Navigation warnings">
      {warnings.map(w => <StatusMessage key={w.id} priority={w.severity === 'danger' ? 'critical' : 'status'} tone={w.severity === 'danger' ? 'danger' : 'caution'}>{w.message}</StatusMessage>)}
      {ship.contact && <StatusMessage priority={ship.mode === 'destroyed' ? 'critical' : 'status'} tone={ship.mode === 'destroyed' ? 'danger' : 'info'}>{`${ship.mode === 'destroyed' ? 'Journey ended — ship destroyed' : ship.contact.outcome} · impact ${ship.contact.impactSpeedMps.toFixed(1)} m/s`}</StatusMessage>}
      {!navigation.location && !grounded && (ship.mode === 'destroyed' || ship.mode === 'grounded') && <button className="ui-button ui-button--secondary" onClick={() => { navigation.reset(); view.reset(navigation.clock.state); redraw(n => n + 1) }}>Restart navigation</button>}
    </div>
    <PerspectiveContacts destinationId={navigation.location?.landing?.surface.bodyId} compact browserOpen={browser} onBrowserChange={setBrowser} scanner={navigation.scanner} geometry={navigation.bodies} view={view}
      getOrigin={() => navigation.clock.state.position} getShip={() => navigation.clock.state} canPilot={canPilot}
      onSetDirection={body => { navigation.clock.state = setContactDirection(navigation.scanner, navigation.clock.state, body.id, canPilot() && !isPaused() && !document.querySelector('dialog[open]')) }}/>
    <div className="panoramic-instruments"><OrientationCompass ship={ship} view={snapshot}/></div>
    <footer className="panoramic-controls">
      <section className="panoramic-propulsion ui-panel ui-panel--utility" aria-label="Velocity and thrust">
        <strong className="ui-eyebrow">Velocity / thrust</strong>
        <output className="velocity-readout" aria-live="off" aria-label="Inertial speed">{speed.toFixed(1)} <small>m/s</small></output>
        <label>Throttle {ship.throttle}%<input aria-label="Throttle" type="range" min="-100" max="100" value={ship.throttle} disabled={ship.mode !== 'flying' || !canPilot()} onChange={e => throttle(Number(e.target.value))}/></label>
        <button className="ui-button ui-button--secondary" disabled={ship.mode !== 'flying' || !canPilot()} onClick={() => throttle(0)}>Zero throttle (Space)</button>
      </section>
      <section className="panoramic-maneuvers" aria-label="Ship maneuvers">
      {takeoff && (!landing || ship.mode === 'grounded' || ship.mode === 'taking-off') && <div className="launch-control">
        <button className="ui-button" disabled={!!takeoff.denial()} aria-describedby="takeoff-status" onClick={() => { if (!isPaused() && !document.querySelector('dialog[open]') && !takeoff.denial()) { input.current.clear(); takeoff.activate(); redraw(n => n + 1) } }}>Take off</button>
        <span id="takeoff-status" role="status">{navigation.location?.launch ? `Taking off · ${Math.min(100, Math.floor((ship.tick - navigation.location.launch.startTick) / TAKEOFF_TICKS * 100))}% · automatic launch` : ship.mode === 'flying' ? 'Flying · manual controls available' : takeoff.denial() ?? 'Ready for takeoff'}</span>
      </div>}
      {landing && ship.mode !== 'grounded' && ship.mode !== 'taking-off' && <div className="launch-control landing-control">
        <button className="ui-button" disabled={!!landingDenial} aria-describedby="landing-status landing-readings" onClick={() => {
          const target = landingReadout(navigation).body
          if (target && canPilot() && !isPaused() && !document.querySelector('dialog[open]') && !landing.denial(target.id)) {
            input.current.clear(); landing.activate(target.id); redraw(n => n + 1)
          }
        }}>Land</button>
        <div><span id="landing-readings">{approach.body ? `${approach.body.name} · altitude ${approach.altitudeM!.toFixed(1)} m · full relative speed ${approach.relativeSpeedMps!.toFixed(3)} m/s` : 'No scanned landing target'}</span>
          <span id="landing-status" role="status">{ship.mode === 'destroyed' ? 'Journey ended — whole ship destroyed. No surface access.' : progress ?? landingDenial ?? 'Ready to land · no heading requirement'}</span></div>
      </div>}
      </section>
      <section className="panoramic-systems ui-panel ui-panel--utility" aria-label="Available ship systems">
      <strong className="ui-eyebrow">Observation systems</strong>
      <button className="ui-button ui-button--secondary" onClick={() => { view.reset(navigation.clock.state); redraw(n => n + 1) }}>Reset view</button>
      <button className="ui-button ui-button--secondary telemetry-toggle" aria-expanded={telemetry} aria-controls="detailed-telemetry" onClick={() => setTelemetry(value => !value)}>Telemetry</button>
      <button className="ui-button ui-button--secondary" aria-pressed={orientationGrid} onClick={() => setOrientationGrid(value => !value)}>Orientation grid</button>
      </section>
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
