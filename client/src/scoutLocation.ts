import { physicalRadiusM } from './contact.ts'
import { ORIGIN, relativeM, translate } from './coordinates.ts'
import { normal } from './math.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { initialShip, type ShipState } from './simulation.ts'
import type { WorldFrames } from './topDownFrames.ts'

// The authored site's tangent origin is the +Y surface of sample Vesper.
// Its ship origin (local metres) selects a radial contact on that sphere.
// This is a deterministic illustrative binding, not canonical planet generation.
// Both presentations consume this domain value. Until world relocation exists,
// no command may transition this location to flight or another surface anchor.
export function startingScoutLocation() {
  const body = SAMPLE_BODIES.find(body => body.id === 'vesper')!
  const [x, z] = relativeM(STARTING_SITE.shipOrigin, ORIGIN)
  const radius = physicalRadiusM(body)
  const radial = [x, Math.sqrt(radius * radius - x * x - z * z), z] as const
  const planetOrigin = translate(body.position, [0, radius, 0])
  const frames: WorldFrames = {
    planet: { id: 'planet:fixture', universeOrigin: planetOrigin, angle: 0 },
    ship: { id: 'ship:scout', parent: 'planet:fixture', origin: STARTING_SITE.shipOrigin, angle: STARTING_SITE.shipAngle },
  }
  const ship: ShipState = {
    ...initialShip(), position: translate(body.position, radial),
    yaw: STARTING_SITE.shipAngle, mode: 'grounded',
    contact: { bodyId: body.id, outcome: 'safe-contact', impactSpeedMps: 0, normal: normal(radial), ageTicks: 60 },
  }
  return { siteId: STARTING_SITE.id, body, frames, ship } as const
}
export type ScoutLocation = ReturnType<typeof startingScoutLocation>

