import { relativeM } from './coordinates.ts'
import type { ObservationCameraState } from './observationCamera.ts'
import { currentContacts } from './perspectiveContacts.ts'
import type { Body } from './sampleUniverse.ts'
import type { Scanner } from './scanner.ts'
import { alignTo, type ShipState } from './simulation.ts'

export function contactCommandAvailability(ship: ShipState, body: Body, canPilot = true) {
  return {
    currentPlanet: ship.mode === 'grounded' && ship.contact?.bodyId === body.id,
    lookAt: !(ship.mode === 'grounded' && ship.contact?.bodyId === body.id),
    setDirection: canPilot && ship.mode === 'flying',
  }
}

/** Revalidate detection and physical state at activation, including stale UI events.
 * Optical visibility is deliberately not an action prerequisite.
 */
export function lookAtContact(scanner: Scanner, ship: ShipState, view: ObservationCameraState, id: string): boolean {
  const body = currentContacts(scanner, ship.position).find(body => body.id === id)
  return !!body && contactCommandAvailability(ship, body).lookAt && view.lookAt(relativeM(body.position, ship.position))
}
export function setContactDirection(scanner: Scanner, ship: ShipState, id: string, canPilot = true): ShipState {
  const body = currentContacts(scanner, ship.position).find(body => body.id === id)
  return body && contactCommandAvailability(ship, body, canPilot).setDirection
    ? alignTo(ship, relativeM(body.position, ship.position)) : ship
}
