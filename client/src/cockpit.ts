import { SCOUT_ANCHORS } from './scoutLayout.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { point2, type Anchor } from './topDownFrames.ts'
import { PLAYER_ID, type TopDownState } from './topDownRuntime.ts'

// Local reservation/control boundary, ready to replace with atomic server authority.
// A player identity, chair reservation and local screen are separate concepts.
export interface CockpitState {
  readonly enabled: boolean
  readonly powered: boolean
  readonly occupantId: string | null
  readonly ownerId: string
  readonly view: 'world' | 'radar'
}
export const LOCAL_PLAYER = 'player:local'
export const initialCockpit = (enabled = false): CockpitState => ({ enabled, powered: true, occupantId: null, ownerId: LOCAL_PLAYER, view: 'world' })
export const controlsCockpit = (seat: CockpitState, actorId: string) => seat.enabled && seat.powered && seat.occupantId === actorId && seat.ownerId === actorId && seat.view === 'radar'
export const isCockpitTarget = (id: string | null) => id === 'scout:desk' || id === 'scout:chair'
function place(state: TopDownState, anchor: Anchor, seated: boolean): TopDownState {
  return { ...state, entities: state.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e),
    player: { facing: state.frames.ship.angle, mode: seated ? 'seated' : 'standing', targetId: seated ? 'scout:chair' : null } }
}
export function enterCockpit(state: TopDownState, actorId = LOCAL_PLAYER): TopDownState {
  const seat = state.cockpit, player = state.entities.find(e => e.id === PLAYER_ID)!
  const target = state.entities.find(e => e.id === state.selectedId)
  if (!seat.enabled || actorId !== seat.ownerId || (seat.occupantId !== null && seat.occupantId !== actorId)) return { ...state, feedback: 'Cockpit unavailable: owner control and a free chair are required.' }
  if (!seat.powered) return { ...state, feedback: 'Cockpit has no power. You can stand up without power.' }
  if (player.anchor.frame !== state.frames.ship.id || !target || !isCockpitTarget(target.id) || !interactionInReach(player.anchor, target, state.frames)) return { ...state, feedback: 'Approach the cockpit desk to navigate.' }
  return { ...place(state, SCOUT_ANCHORS.chair!, true), cockpit: { ...seat, occupantId: actorId, view: 'radar' }, feedback: 'Seated at cockpit · navigation active.' }
}
export function leaveCockpit(state: TopDownState): TopDownState {
  if (state.cockpit.occupantId !== LOCAL_PLAYER) return state
  const anchor: Anchor = { frame: state.frames.ship.id, point: point2(0, -3.1) }
  return { ...place(state, anchor, false), cockpit: { ...state.cockpit, occupantId: null, view: 'world' }, feedback: 'Standing beside cockpit · chair released.' }
}
export function setCockpitPower(state: TopDownState, powered: boolean): TopDownState {
  return { ...state, cockpit: { ...state.cockpit, powered, view: powered ? state.cockpit.view : 'world' },
    feedback: powered ? 'Cockpit power restored. Interact to reopen navigation.' : 'Cockpit power lost. Remain seated or stand up.' }
}
