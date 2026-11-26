import { cancelGroundMovement, groundPositionClear, terrainMovementFraction, type GroundCommand } from './topDownNavigation.ts'
import type { ObjectCommand } from './topDownCommands.ts'
import { followWorldCamera, type WorldCamera } from './topDownCamera.ts'
import { surfaceRegion, surfaceToLocal, localToSurface } from './surfaceCoordinates.ts'
import { BOARDING_DOOR_ID, BOARDING_DOOR_HALF_DEPTH_M, synchronizeDoor, toggleDoor } from './scoutDoor.ts'
import { startingScoutLocation, type ScoutLocation } from './scoutLocation.ts'
import { initialCockpit, enterCockpit, isCockpitTarget, type CockpitState } from './cockpit.ts'
import { siteEntities, resolveSurface } from './surfaceSite.ts'
import { planetaryEntities, STARTING_SITE, type SourceIdentity } from './planetarySite.ts'
import { scoutEntities } from './scoutLayout.ts'
import { copyPlayerLocation, crossScoutEntry, type PlayerLocation } from './topDownTraversal.ts'
import { relativeM, ORIGIN, translate } from './coordinates.ts'
import { allowedMove, objectAngle, PLAYER_RADIUS_M, interactionInReach, nearbyObject, type WorldObject } from './topDownInteraction.ts'
import { walkingFrames, convertAnchor, point2, relativeAnchor, rotate2, worldToView, type Anchor, type Point2, type WorldFrames } from './topDownFrames.ts'

export const LAYERS = ['terrain', 'hull', 'interior', 'people', 'props', 'cues'] as const
export type Layer = typeof LAYERS[number]
export type Appearance = 'ground' | 'surface' | 'rock' | 'hull' | 'floor' | 'player' | 'desk' | 'bed' | 'core' | 'crate' | 'wall' | 'door' | 'cue' | 'chair' | 'fabricator' | 'storage' | 'threshold' | 'mineral' | 'flora' | 'stones'
export interface Entity2D extends WorldObject { readonly layer: Layer; readonly appearance: Appearance; readonly source?: SourceIdentity }
export interface TopDownState {
  readonly objectCommand?: ObjectCommand
  readonly groundCommand?: GroundCommand
  readonly scoutLocation: ScoutLocation
  readonly cockpit: CockpitState
  readonly tick: number
  readonly player: { readonly facing: number; readonly mode: 'standing' | 'walking' | 'interacting' | 'seated'; readonly targetId: string | null }
  readonly selectedId: string | null
  readonly feedback: string
  readonly frames: WorldFrames
  readonly entities: readonly Entity2D[]
  readonly camera: WorldCamera
}
export const STEP_2D = 1 / 60
export const PLAYER_SPEED_MPS = 2.4
export const PLAYER_ID = 'person:player'
const entity = (id: string, frame: Anchor['frame'], x: number, y: number, layer: Layer, appearance: Appearance): Entity2D =>
  ({ id, anchor: { frame, point: point2(x, y) }, layer, appearance })

export function createTopDownState(view: 'interior' | 'exterior' = 'exterior', cockpitEnabled = false): TopDownState {
  const scoutLocation = startingScoutLocation()
  const entities = [
    ...planetaryEntities(),
    entity('ship:scout', 'ship:scout', 0, 0, 'hull', 'hull'),
    entity('scout:floor', 'ship:scout', 0, 0, 'interior', 'floor'),
    view === 'exterior' ? { ...entity(PLAYER_ID, 'planet:fixture', 0, 0, 'people', 'player'), anchor: STARTING_SITE.spawn } : entity(PLAYER_ID, 'ship:scout', 0, 2, 'people', 'player'),
    ...scoutEntities(),
  ]
  return {
    scoutLocation,
    cockpit: initialCockpit(cockpitEnabled), tick: 0,
    player: { facing: 0, mode: 'standing', targetId: null }, selectedId: null, feedback: 'Approach a fixture to inspect it. Systems are inactive.',
    frames: scoutLocation.frames,
    entities,
    camera: { anchor: entities.find(e => e.id === PLAYER_ID)!.anchor, pixelsPerM: view === 'exterior' ? 26 : 32 },
  }
}
// One continuous scene. Crossing changes ownership, never world position.
export function stepTopDown(state: TopDownState, direction: Point2, speedMultiplier = 1): TopDownState {
  const previousAngle = state.frames.ship.angle
  state = rechartWalking(state)
  direction = rotate2(direction, state.frames.ship.angle - previousAngle)
  if (Math.hypot(...direction) > 0) state = followWorldCamera(state)
  if (Math.hypot(...direction) > 0 || state.player.mode === 'seated' || state.cockpit.view !== 'world' || state.scoutLocation.ship.mode === 'destroyed') state = cancelGroundMovement(state)
  let command = state.groundCommand
  if (command?.status === 'moving' && !groundPositionClear(state, state.entities.find(e => e.id === PLAYER_ID)!.anchor)) {
    command = { ...command, status: 'failed', waypoints: [] }
    state = { ...state, groundCommand: command, feedback: 'Route blocked. Choose another destination.' }
  }
  if (command?.status === 'moving') {
    const anchor = state.entities.find(e => e.id === PLAYER_ID)!.anchor
    let waypoints = command.waypoints
    while (waypoints.length && Math.hypot(...relativeAnchor(waypoints[0]!, anchor, state.frames)) < 1e-7) waypoints = waypoints.slice(1)
    command = { ...command, waypoints, status: waypoints.length ? 'moving' : 'arrived' }
    state = { ...state, groundCommand: command, feedback: waypoints.length ? state.feedback : 'Arrived at destination.' }
    if (waypoints.length) {
      const delta = relativeAnchor(waypoints[0]!, anchor, state.frames), distance = Math.hypot(...delta)
      const speed = PLAYER_SPEED_MPS * speedMultiplier * STEP_2D
      const factor = Math.min(1, distance / speed) / distance
      direction = [delta[0] * factor, delta[1] * factor]
    }
  }
  const [x, y] = state.player.mode === 'seated' || state.scoutLocation.ship.mode === 'destroyed' ? [0, 0] : direction
  if (![x, y].every(Number.isFinite)) throw new RangeError('Expected finite movement')
  const length = Math.hypot(x, y), scale = length > 0 ? PLAYER_SPEED_MPS * speedMultiplier * STEP_2D / Math.max(1, length) : 0
  const player = state.entities.find(e => e.id === PLAYER_ID)!
  let anchor = player.anchor
  let terrainBlocked = false
  const angle = anchor.frame === state.frames.ship.id ? state.frames.ship.angle : 0
  const local = rotate2([x * scale, y * scale], -angle)
  // Routes must sweep the same diagonal edge validated by the planner. Axis
  // splitting can clip a nearby corner that the diagonal legally clears.
  // Direct keyboard movement retains its existing wall sliding.
  const deltas: readonly Point2[] = command?.status === 'moving' ? [local] : [[local[0], 0], [0, local[1]]]
  for (const delta of deltas) {
    const terrainFraction = terrainMovementFraction(state, anchor, rotate2(delta, angle))
    const fraction = Math.min(allowedMove(anchor, rotate2(delta, angle), state.entities, state.frames), terrainFraction)
    terrainBlocked ||= terrainFraction < 1
    anchor = { ...anchor, point: translate(anchor.point, [delta[0] * fraction, delta[1] * fraction, 0]) }
  }
  const nearShip = Math.hypot(...relativeAnchor(anchor, {frame: 'ship:scout', point: point2(0,0)}, state.frames)) < 64
  const crossed = nearShip ? crossScoutEntry(player.anchor, anchor, state.frames, state.scoutLocation.boardingDoor === 'open') : anchor
  const doorBlocked = state.frames.ship.parent === null && (anchor.point.offsetM[1] > 5.5 || (state.scoutLocation.boardingDoor === 'closed' && length > 0 && player.anchor.point.offsetM[1] >= 5.5 - BOARDING_DOOR_HALF_DEPTH_M - PLAYER_RADIUS_M - 1e-6 && local[1] > 0))
  anchor = doorBlocked ? player.anchor : crossed
  if (command?.status === 'moving') {
    const actual = relativeAnchor(anchor, player.anchor, state.frames)
    if (Math.hypot(actual[0] - x * scale, actual[1] - y * scale) > 1e-6) {
      state = { ...state, groundCommand: { ...command, status: 'failed', waypoints: [] }, feedback: 'Route blocked. Choose another destination.' }
    }
  }
  const entities = state.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e)
  const selectedId = nearbyObject(anchor, entities, state.frames)?.id ?? null
  let activity = state.player
  let feedback = doorBlocked ? 'Airlock locked in flight. Land at a validated surface site before leaving the Scout.' : terrainBlocked ? 'Ground too steep for walking. Follow the gentler terrain.' : state.feedback
  if (length > 0) {
    activity = { facing: Math.atan2(x, -y), mode: 'walking', targetId: null }
    if (state.player.mode === 'interacting') feedback = 'Inspection cancelled by movement.'
  } else if (activity.mode === 'walking') activity = { ...activity, mode: 'standing' }
  if (activity.targetId && activity.mode !== 'seated') {
    const target = entities.find(e => e.id === activity.targetId)
    if (!target || !interactionInReach(anchor, target, state.frames)) {
      activity = { ...activity, mode: 'standing', targetId: null }; feedback = 'Inspection cancelled: target unavailable.'
    }
  }
  // Fixed-step exponential follow: same result at any render cadence. The camera
  // follows through frame handoff without resetting its lag or zoom.
  const [cx, cy] = relativeAnchor(anchor, state.camera.anchor, state.frames)
  const cameraAngle = state.camera.anchor.frame === state.frames.ship.id ? state.frames.ship.angle : 0
  const deltaCamera = rotate2([cx * .16, cy * .16], -cameraAngle)
  // Finish a settled follow exactly; residual 1e-14 m transforms can change
  // subpixel SVG rasterization after view changes. Player/world points stay exact.
  const cameraAnchor = { ...state.camera.anchor, point: translate(state.camera.anchor.point, [deltaCamera[0], deltaCamera[1], 0]) }
  return { ...state, tick: state.tick + 1, entities, player: activity, selectedId, feedback,
    camera: state.camera.panned ? state.camera : { ...state.camera, anchor: Math.hypot(cx, cy) < 1e-9 ? anchor : convertAnchor(cameraAnchor, anchor.frame, state.frames) } }
}
export function capturePlayerLocation(state: TopDownState): PlayerLocation {
  return copyPlayerLocation({ anchor: state.entities.find(e => e.id === PLAYER_ID)!.anchor, facing: state.player.facing })
}
export function requestInteraction(state: TopDownState, id: string | null = state.selectedId): TopDownState {
  state = cancelGroundMovement(state)
  if (state.scoutLocation.ship.mode === 'destroyed') return { ...state, feedback: 'Journey ended — whole ship destroyed.' }
  if (id === BOARDING_DOOR_ID) return toggleDoor(state)
  if (state.cockpit.enabled && isCockpitTarget(id)) return enterCockpit({ ...state, selectedId: id })
  const player = state.entities.find(e => e.id === PLAYER_ID)!, target = state.entities.find(e => e.id === id)
  if (!target || !interactionInReach(player.anchor, target, state.frames)) {
    return { ...state, feedback: 'No available object in reach. Move closer to a console.' }
  }
  return { ...state, selectedId: target.id, player: { ...state.player, mode: 'interacting', targetId: target.id },
    feedback: `${target.interaction!.label}: test inspection active. No systems changed.` }
}
export function cancelInteraction(state: TopDownState): TopDownState {
  state = cancelGroundMovement(state)
  return state.player.mode === 'interacting' ? { ...state, player: { ...state.player, mode: 'standing', targetId: null }, feedback: 'Inspection cancelled.' } : state
}
export interface DrawItem { readonly entity: Entity2D; readonly positionM: Point2; readonly angle: number }
type DrawListState = Pick<TopDownState, 'entities' | 'frames'> & {
  readonly player: Pick<TopDownState['player'], 'mode' | 'facing'>
  readonly camera: Pick<TopDownState['camera'], 'anchor'>
}
export function drawList(state: DrawListState): DrawItem[] {
  return state.entities.filter(e => state.frames.ship.parent !== null || e.anchor.frame === state.frames.ship.id).map(e => e.id === PLAYER_ID && state.player.mode === 'seated' ? { ...e, layer: 'cues' as const } : e).sort((a, b) => LAYERS.indexOf(a.layer) - LAYERS.indexOf(b.layer)
    || Number(b.appearance === 'ground') - Number(a.appearance === 'ground')
    || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))
    .map(entity => ({ entity, positionM: worldToView(relativeAnchor(entity.anchor, state.camera.anchor, state.frames), state.frames), angle: ( entity.id === PLAYER_ID ? state.player.facing : objectAngle(entity, state.frames)) - state.frames.ship.angle }))
    .filter(item => item.entity.id === PLAYER_ID || Math.hypot(...item.positionM) < (item.entity.appearance === 'ground' || item.entity.appearance === 'surface' ? 300 : 100))
}
export class TopDownClock {
  private remainder = 0
  advance(state: TopDownState, seconds: number, direction: Point2, step = stepTopDown): TopDownState {
    if (!Number.isFinite(seconds) || seconds < 0) throw new RangeError('Expected nonnegative finite time')
    this.remainder += Math.min(seconds, .25)
    while (this.remainder + 1e-12 >= STEP_2D) {
      state = step(state, direction)
      this.remainder = Math.max(0, this.remainder - STEP_2D)
    }
    return state
  }
  reset() { this.remainder = 0 }
}

// Called when the shared lifecycle changes. Preserve camera lag by translating
// it to ship-local coordinates before invalidating the planetary transform.
export function synchronizeScout(state: TopDownState): TopDownState {
  state = synchronizeDoor(state)
  if ((state.frames.surface?.source ?? state.frames) === state.scoutLocation.frames) return state
  const camera = state.camera.anchor.frame === 'planet:fixture' && state.scoutLocation.binding === null
    ? { ...state.camera, anchor: convertAnchor(state.camera.anchor, 'ship:scout', state.frames) } : state.camera
  const entities = [
    ...state.entities.filter(e => e.anchor.frame === 'ship:scout' || e.layer !== 'terrain'),
    ...(state.scoutLocation.binding ? siteEntities(resolveSurface(state.scoutLocation.surface)) : []),
  ]
  return { ...cancelGroundMovement(state), entities, frames: state.scoutLocation.frames, camera, selectedId: null }
}

/** Change charts before precision/horizon limits, preserving physical positions.
 * Keep the original landing chart nearby for exact authored collision/door art. */
export function rechartWalking(state: TopDownState): TopDownState {
  const player = state.entities.find(e=>e.id===PLAYER_ID)!
  if (!state.scoutLocation.binding || player.anchor.frame !== 'planet:fixture') return state
  const address = state.scoutLocation.surface, landing = surfaceRegion(address,address.definition)
  const old = player.anchor.chart ?? landing
  const position = localToSurface(old, relativeM(player.anchor.point,ORIGIN).slice(0,2) as [number,number])
  const current = state.frames.surface?.chart ?? landing
  const local = surfaceToLocal(position,current)
  const homeDistance = Math.hypot(...position.radialM.map((n,i)=>n-address.radialM[i]!))
  const home = homeDistance < 128
  // Align the remote transported view gradually before the Scout is visible.
  // This removes loop holonomy without a camera snap on returning home.
  if (state.frames.surface && homeDistance < 512) {
    const desired = walkingFrames(state.scoutLocation.frames,landing,current).ship.angle
    const difference = Math.atan2(Math.sin(desired-state.frames.ship.angle),Math.cos(desired-state.frames.ship.angle))
    if (Math.abs(difference)>1e-12) state={...state,frames:{...state.frames,ship:{...state.frames.ship,angle:state.frames.ship.angle+Math.max(-.005,Math.min(.005,difference))}}}
  }
  if (Math.hypot(...local) < 256 && (!home || current.id === landing.id)) return state
  const chart = home ? landing : surfaceRegion(position,address.definition)
  if (chart.id === current.id) return state
  const frames = walkingFrames(state.scoutLocation.frames,landing,chart,state.frames)
  const project = (anchor: Anchor): Anchor => {
    if (anchor.frame !== 'planet:fixture') return anchor
    const physical = localToSurface(anchor.chart ?? landing, relativeM(anchor.point,ORIGIN).slice(0,2) as [number,number])
    return {frame:anchor.frame,point:point2(...surfaceToLocal(physical,chart)),chart}
  }
  return {...state,frames,entities:state.entities.map(e=>e.id===PLAYER_ID ? {...e,anchor:project(e.anchor)} : e),camera:{...state.camera,anchor:project(state.camera.anchor)}}
}
