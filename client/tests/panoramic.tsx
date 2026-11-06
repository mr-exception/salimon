import { createRoot } from 'react-dom/client'
import { PanoramicNavigation } from '../src/PanoramicNavigation.tsx'
import { ObservationCameraState } from '../src/observationCamera.ts'
import { NavigationRuntime } from '../src/navigationRuntime.ts'
import { SAMPLE_BODIES, type Body } from '../src/sampleUniverse.ts'
import { startingScoutLocation } from '../src/scoutLocation.ts'
import { ORIGIN, translate } from '../src/coordinates.ts'
const grounded = new URLSearchParams(location.search).has('grounded')
const bodies: readonly Body[] = grounded ? SAMPLE_BODIES : Array.from({ length: 40 }, (_, i) => ({ id: `contact-${i}`, name: `Contact ${String(i).padStart(2, '0')}`, kind: 'moon', radiusKm: 0.01, position: translate(ORIGIN, [i * 1000, 5000, -100000]) }))
const navigation = new NavigationRuntime(bodies, grounded ? startingScoutLocation() : undefined)
const view = new ObservationCameraState(navigation.clock.state)
let renderer: string | null = null, exits = 0
Object.assign(window, {
 // Exercise the domain clock without tying scanner acquisition to GPU frame time.
 hudAdvance: (seconds: number) => {
  // Stay below NavigationClock's per-call catch-up limit.
  for (let elapsed = 0; elapsed < seconds; elapsed++) navigation.advance(Math.min(1, seconds - elapsed))
 },
 hudSnapshot: () => ({ ship: structuredClone(navigation.clock.state), view: view.snapshot(), contacts: navigation.scanner.contacts.map(b => b.id), renderer, exits }),
 hudScenario: (kind: string) => {
  if (kind === 'rear') navigation.clock.state = { ...navigation.clock.state, yaw: Math.PI, velocityMps: [0, 0, 10] }
  if (kind === 'overlap') navigation.clock.state = { ...navigation.clock.state, yaw: 0, velocityMps: [0, 0, -10] }
  if (kind === 'stationary') navigation.clock.state = { ...navigation.clock.state, velocityMps: [0, 0, 0] }
  if (kind === 'warning') navigation.clock.state = { ...navigation.clock.state, position: translate(ORIGIN, [0, 5000, -98000]), velocityMps: [0, 0, -200] }
  if (kind === 'destroyed') navigation.clock.state = { ...navigation.clock.state, mode: 'destroyed', throttle: 0, contact: { bodyId: bodies[0].id, outcome: 'destroyed', impactSpeedMps: 200, normal: [0, 0, 1], ageTicks: 0 } }
 }
})
createRoot(document.getElementById('root')!).render(<PanoramicNavigation navigation={navigation} view={view} opticalBodies={bodies.slice(0, 2)} onExit={() => exits++} onSceneReady={({ gl }) => {
 const ctx = gl.getContext(), debug = ctx.getExtension('WEBGL_debug_renderer_info')
 renderer = debug ? ctx.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null
}}/>)
