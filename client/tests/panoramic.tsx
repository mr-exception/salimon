import { createRoot } from 'react-dom/client'
import { PanoramicNavigation } from '../src/PanoramicNavigation.tsx'
import { ObservationCameraState } from '../src/observationCamera.ts'
import { NavigationRuntime } from '../src/navigationRuntime.ts'
import { SAMPLE_BODIES, type Body } from '../src/sampleUniverse.ts'
import { startingScoutLocation } from '../src/scoutLocation.ts'
import { ORIGIN, translate } from '../src/coordinates.ts'
import type { WebGLRenderer } from 'three'
const params = new URLSearchParams(location.search)
const grounded = params.has('grounded')
const bodies: readonly Body[] = grounded ? SAMPLE_BODIES : Array.from({ length: 40 }, (_, i) => ({ id: `contact-${i}`, name: `Contact ${String(i).padStart(2, '0')}`, kind: 'moon', radiusKm: 0.01, position: translate(ORIGIN, [i * 1000, 5000, -100000]) }))
const navigation = new NavigationRuntime(bodies, grounded ? startingScoutLocation() : undefined)
const view = new ObservationCameraState(navigation.clock.state)
let renderer: string | null = null, exits = 0
let graphics: WebGLRenderer | undefined
Object.assign(window, {
 hudView: (fov: number, yaw = 0, pitch = 0) => { view.reset({ yaw, pitch }); view.restore({ ...view.snapshot(), fov }) },
 // Exercise the domain clock without tying scanner acquisition to GPU frame time.
 hudAdvance: (seconds: number) => {
  // Stay below NavigationClock's per-call catch-up limit.
  for (let elapsed = 0; elapsed < seconds; elapsed++) navigation.advance(Math.min(1, seconds - elapsed))
 },
 hudSnapshot: () => ({ ship: structuredClone(navigation.clock.state), view: view.snapshot(), contacts: navigation.scanner.contacts.map(b => b.id), renderer, exits, calls: graphics?.info.render.calls, triangles: graphics?.info.render.triangles, geometries: graphics?.info.memory.geometries }),
 hudScenario: (kind: string) => {
  if (kind === 'lost-contact') navigation.clock.state = { ...navigation.clock.state, position: translate(ORIGIN, [1e20, 0, 0]) }
  if (kind === 'rear') navigation.clock.state = { ...navigation.clock.state, yaw: Math.PI, velocityMps: [0, 0, 10] }
  if (kind === 'overlap') navigation.clock.state = { ...navigation.clock.state, yaw: 0, velocityMps: [0, 0, -10] }
  if (kind === 'metric-speed') navigation.clock.state = { ...navigation.clock.state, velocityMps: [12400, 0, 0], throttle: 0 }
  if (kind === 'stationary') navigation.clock.state = { ...navigation.clock.state, velocityMps: [0, 0, 0] }
  if (kind === 'warning') navigation.clock.state = { ...navigation.clock.state, position: translate(ORIGIN, [0, 5000, -98000]), velocityMps: [0, 0, -200] }
  if (kind === 'proximity') navigation.clock.state = { ...navigation.clock.state, position: translate(ORIGIN, [0, 5000, -99500]), velocityMps: [0, 0, 0], throttle: 0 }
  if (kind === 'clear-warning') navigation.clock.state = { ...navigation.clock.state, position: ORIGIN, velocityMps: [0, 0, 0], throttle: 0 }
  if (kind === 'recovery') navigation.clock.state = { ...navigation.clock.state, mode:'grounded', throttle:0, contact:{ bodyId:bodies[0].id, outcome:'crash', impactSpeedMps:50, normal:[0,0,1], ageTicks:0 } }
  if (kind === 'destroyed') navigation.clock.state = { ...navigation.clock.state, mode: 'destroyed', throttle: 0, contact: { bodyId: bodies[0].id, outcome: 'destroyed', impactSpeedMps: 200, normal: [0, 0, 1], ageTicks: 0 } }
 }
})
createRoot(document.getElementById('root')!).render(<PanoramicNavigation navigation={navigation} view={view} conditionWarning={params.has('bright') ? 'Low health · critical condition.' : ''} opticalBodies={params.has('environment') ? [] : params.has('bright') ? [{ id: 'bright-fixture', name: 'Unscanned bright star', kind: 'star', position: translate(navigation.clock.state.position, [0, 0, -1e9]), radiusKm: 990000 }] : bodies.slice(0, 2)} onExit={() => exits++} onSceneReady={({ gl }) => {
 graphics = gl
 const ctx = gl.getContext(), debug = ctx.getExtension('WEBGL_debug_renderer_info')
 renderer = debug ? ctx.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null
}}/>)
