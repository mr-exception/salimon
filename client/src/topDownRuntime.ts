import { planetaryEntities, STARTING_SITE, type SourceIdentity } from './planetarySite.ts'
import { scoutEntities } from './scoutLayout.ts'
import { translate } from './coordinates.ts'
import { allowedMove, interactionInReach, nearbyObject, type WorldObject } from './topDownInteraction.ts'
import { point2, relativeAnchor, rotate2, type Anchor, type Point2, type WorldFrames } from './topDownFrames.ts'

export const LAYERS = ['terrain', 'hull', 'interior', 'people', 'props', 'cues'] as const
export type Layer = typeof LAYERS[number]
export type Appearance = 'ground' | 'rock' | 'hull' | 'floor' | 'player' | 'desk' | 'bed' | 'core' | 'crate' | 'wall' | 'door' | 'cue' | 'chair' | 'fabricator' | 'storage' | 'threshold' | 'mineral' | 'flora' | 'stones'
export interface Entity2D extends WorldObject { readonly layer: Layer; readonly appearance: Appearance; readonly source?: SourceIdentity }
export interface TopDownState {
  readonly tick: number
  readonly player: { readonly facing: number; readonly mode: 'standing' | 'walking' | 'interacting'; readonly targetId: string | null }
  readonly selectedId: string | null
  readonly feedback: string
  readonly frames: WorldFrames
  readonly entities: readonly Entity2D[]
  readonly camera: { readonly anchor: Anchor; readonly pixelsPerM: number }
}
export const STEP_2D = 1 / 60
export const PLAYER_SPEED_MPS = 2.4
export const PLAYER_ID = 'person:player'
const entity = (id: string, frame: Anchor['frame'], x: number, y: number, layer: Layer, appearance: Appearance): Entity2D =>
  ({ id, anchor: { frame, point: point2(x, y) }, layer, appearance })

export function createTopDownState(view: 'interior' | 'exterior' = 'exterior'): TopDownState {
  const entities = [
    ...planetaryEntities(),
    entity('ship:scout', 'ship:scout', 0, 0, 'hull', 'hull'),
    entity('scout:floor', 'ship:scout', 0, 0, 'interior', 'floor'),
    view === 'exterior' ? { ...entity(PLAYER_ID, 'planet:fixture', 0, 0, 'people', 'player'), anchor: STARTING_SITE.spawn } : entity(PLAYER_ID, 'ship:scout', 0, 2, 'people', 'player'),
    ...scoutEntities(),
  ]
  return {
    tick: 0,
    player: { facing: 0, mode: 'standing', targetId: null }, selectedId: null, feedback: 'Approach a fixture to inspect it. Systems are inactive.',
    frames: {
      planet: { id: 'planet:fixture', universeOrigin: { cell: [10n ** 24n, -(10n ** 24n), 0n], offsetM: [0, 0, 0] }, angle: .2 },
      ship: { id: 'ship:scout', parent: 'planet:fixture', origin: STARTING_SITE.shipOrigin, angle: STARTING_SITE.shipAngle },
    },
    entities,
    camera: { anchor: entities.find(e => e.id === PLAYER_ID)!.anchor, pixelsPerM: view === 'exterior' ? 26 : 32 },
  }
}
// One continuous scene. Collision queries use each object's frame/footprint;
// crossing ownership and cockpit seating remain downstream work.
export function stepTopDown(state: TopDownState, direction: Point2): TopDownState {
  const [x, y] = direction
  if (![x, y].every(Number.isFinite)) throw new RangeError('Expected finite movement')
  const length = Math.hypot(x, y), scale = length > 0 ? PLAYER_SPEED_MPS * STEP_2D / Math.max(1, length) : 0
  const player = state.entities.find(e => e.id === PLAYER_ID)!
  let anchor = player.anchor
  const angle = anchor.frame === state.frames.ship.id ? state.frames.ship.angle : 0
  const local = rotate2([x * scale, y * scale], -angle)
  // Independent local axes provide sliding along walls without tunnelling.
  for (const delta of [[local[0], 0], [0, local[1]]] as const) {
    const fraction = allowedMove(anchor, rotate2(delta, angle), state.entities, state.frames)
    anchor = { ...anchor, point: translate(anchor.point, [delta[0] * fraction, delta[1] * fraction, 0]) }
  }
  const entities = state.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e)
  const selectedId = nearbyObject(anchor, entities, state.frames)?.id ?? null
  let activity = state.player
  let feedback = state.feedback
  if (length > 0) {
    activity = { facing: Math.atan2(x, -y), mode: 'walking', targetId: null }
    if (state.player.mode === 'interacting') feedback = 'Inspection cancelled by movement.'
  } else if (activity.mode === 'walking') activity = { ...activity, mode: 'standing' }
  if (activity.targetId) {
    const target = entities.find(e => e.id === activity.targetId)
    if (!target || !interactionInReach(anchor, target, state.frames)) {
      activity = { ...activity, mode: 'standing', targetId: null }; feedback = 'Inspection cancelled: target unavailable.'
    }
  }
  // Fixed-step exponential follow: same result at any render cadence. The camera
  // remains in the player's frame and never flattens astronomical positions.
  const [cx, cy] = relativeAnchor(anchor, state.camera.anchor, state.frames)
  const cameraAngle = state.camera.anchor.frame === state.frames.ship.id ? state.frames.ship.angle : 0
  const deltaCamera = rotate2([cx * .16, cy * .16], -cameraAngle)
  const cameraAnchor = { ...state.camera.anchor, point: translate(state.camera.anchor.point, [deltaCamera[0], deltaCamera[1], 0]) }
  return { ...state, tick: state.tick + 1, entities, player: activity, selectedId, feedback,
    camera: { ...state.camera, anchor: cameraAnchor } }
}
export function requestInteraction(state: TopDownState, id: string | null = state.selectedId): TopDownState {
  const player = state.entities.find(e => e.id === PLAYER_ID)!, target = state.entities.find(e => e.id === id)
  if (!target || !interactionInReach(player.anchor, target, state.frames)) {
    return { ...state, feedback: 'No available object in reach. Move closer to a console.' }
  }
  return { ...state, selectedId: target.id, player: { ...state.player, mode: 'interacting', targetId: target.id },
    feedback: `${target.interaction!.label}: test inspection active. No systems changed.` }
}
export function cancelInteraction(state: TopDownState): TopDownState {
  return state.player.mode === 'interacting' ? { ...state, player: { ...state.player, mode: 'standing', targetId: null }, feedback: 'Inspection cancelled.' } : state
}
export interface DrawItem { readonly entity: Entity2D; readonly positionM: Point2; readonly angle: number }
export function drawList(state: TopDownState): DrawItem[] {
  return [...state.entities].sort((a, b) => LAYERS.indexOf(a.layer) - LAYERS.indexOf(b.layer)
    || Number(b.appearance === 'ground') - Number(a.appearance === 'ground')
    || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))
    .map(entity => ({ entity, positionM: relativeAnchor(entity.anchor, state.camera.anchor, state.frames), angle: entity.id === PLAYER_ID ? state.player.facing : entity.anchor.frame === state.frames.ship.id ? state.frames.ship.angle : 0 }))
}
export class TopDownClock {
  private remainder = 0
  advance(state: TopDownState, seconds: number, direction: Point2): TopDownState {
    if (!Number.isFinite(seconds) || seconds < 0) throw new RangeError('Expected nonnegative finite time')
    this.remainder += Math.min(seconds, .25)
    while (this.remainder + 1e-12 >= STEP_2D) {
      state = stepTopDown(state, direction)
      this.remainder = Math.max(0, this.remainder - STEP_2D)
    }
    return state
  }
  reset() { this.remainder = 0 }
}
