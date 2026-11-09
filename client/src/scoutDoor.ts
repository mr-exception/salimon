import { takeoffDenial } from './scoutLocation.ts'
import { relativeM } from './coordinates.ts'
import { SCOUT_ANCHORS, SCOUT_ENTRY_HALF_WIDTH_M, SCOUT_ENTRY_REACH_M } from './scoutLayout.ts'
import { PLAYER_RADIUS_M, interactionInReach } from './topDownInteraction.ts'
import { convertAnchor } from './topDownFrames.ts'
import type { TopDownState } from './topDownRuntime.ts'

export const BOARDING_DOOR_ID = 'scout:entry'
export const BOARDING_DOOR_HALF_DEPTH_M = .15
export function doorDenial(world: TopDownState): string | null {
  const location = world.scoutLocation
  if (location.ship.mode !== 'grounded' || !location.binding || location.maneuver !== null || world.frames.ship.parent === null)
    return 'Airlock locked in flight. Land at a validated surface site before opening the aft boarding door.'
  const siteDenial = takeoffDenial({ ...location, boardingDoor: 'closed' })
  if (siteDenial) return siteDenial
  const player = world.entities.find(e => e.id === 'person:player')!, door = world.entities.find(e => e.id === BOARDING_DOOR_ID)
  if (world.player.mode === 'seated' || !door || !interactionInReach(player.anchor, door, world.frames)) return 'Approach the aft boarding door to operate it.'
  // Never close a collider through a person standing across the threshold.
  if (location.boardingDoor === 'open' && world.entities.some(e => {
    if (e.appearance !== 'player' && e.layer !== 'people') return false
    const [x, y] = relativeM(convertAnchor(e.anchor, world.frames.ship.id, world.frames).point, SCOUT_ANCHORS.entry!.point)
    return Math.abs(x) < SCOUT_ENTRY_HALF_WIDTH_M + PLAYER_RADIUS_M && Math.abs(y) < BOARDING_DOOR_HALF_DEPTH_M + PLAYER_RADIUS_M
  })) return 'Boarding doorway occupied. Step clear before closing the door.'
  return null
}
/** Collision and artwork are derived from the sole saved door state. */
export function synchronizeDoor(world: TopDownState): TopDownState {
  const closed = world.scoutLocation.boardingDoor === 'closed', appearance = closed ? 'door' : 'threshold'
  if (world.entities.find(e => e.id === BOARDING_DOOR_ID)?.appearance === appearance) return world
  return { ...world, entities: world.entities.map(e => e.id === BOARDING_DOOR_ID ? { ...e, appearance,
    footprint: closed ? { halfWidth: SCOUT_ENTRY_HALF_WIDTH_M, halfHeight: BOARDING_DOOR_HALF_DEPTH_M } : undefined,
    interaction: { label: `Aft boarding door · ${closed ? 'closed' : 'open'}`, action: closed ? 'Open door' : 'Close door', reachM: SCOUT_ENTRY_REACH_M },
  } : e) }
}
export function toggleDoor(world: TopDownState): TopDownState {
  const denial = doorDenial(world)
  if (denial) return { ...world, feedback: denial }
  const boardingDoor = world.scoutLocation.boardingDoor === 'open' ? 'closed' : 'open'
  return synchronizeDoor({ ...world, scoutLocation: { ...world.scoutLocation, boardingDoor },
    player: { ...world.player, mode: 'standing', targetId: null }, feedback: `Aft boarding door ${boardingDoor}.` })
}
