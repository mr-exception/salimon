import { physicalRadiusM, surfaceVelocity } from './contact.ts'
import { relativeM } from './coordinates.ts'
import { length, normal, scale, sub } from './math.ts'
import type { NavigationRuntime } from './navigationRuntime.ts'

/** Scanner selection, never camera Look at, owns the landing target. Without a
 * selection, offer the nearest detected solid contact. Unknown names stay hidden. */
export function landingReadout(navigation: NavigationRuntime) {
  const ship = navigation.clock.state, scanner = navigation.scanner
  const selected = navigation.location?.landing?.surface.bodyId ?? scanner.selected
  const body = scanner.contacts.find(b => b.id === selected)
    ?? (selected === null ? scanner.contacts.filter(b => b.kind === 'planet' || b.kind === 'moon')
      .sort((a, b) => length(relativeM(ship.position, a.position)) - length(relativeM(ship.position, b.position)))[0] : undefined)
  if (!body) return { body: undefined, altitudeM: null, relativeSpeedMps: null }
  const radial = relativeM(ship.position, body.position), radius = physicalRadiusM(body)
  return { body, altitudeM: length(radial) - radius,
    relativeSpeedMps: length(sub(ship.velocityMps, surfaceVelocity(body, scale(normal(radial), radius)))) }
}

export function landingProgress(navigation: NavigationRuntime): string | null {
  const plan = navigation.location?.landing
  if (!plan) return null
  const progress = Math.max(0, Math.min(1, (navigation.clock.state.tick - plan.startTick) / plan.durationTicks))
  // Labels describe trajectory progress; they do not drive the domain clock.
  return `${progress >= .9 ? 'Touchdown approach' : progress >= .7 ? 'Flare' : 'Descent'} · ${Math.floor(progress * 100)}% · automatic landing · maneuver already running · unlimited prototype energy`
}
