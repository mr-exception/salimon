import { translate, relativeM, ORIGIN } from './coordinates.ts'
import { objectActions, type ActionContext, type ObjectActionId } from './topDownActions.ts'
import { relativeAnchor, type Anchor } from './topDownFrames.ts'
import { BOARDING_DOOR_ID, doorDenial } from './scoutDoor.ts'
import { cancelGroundMovement, groundPositionClear, planGroundRoute, requestGroundMovement } from './topDownNavigation.ts'
import { PLAYER_ID, requestInteraction, type TopDownState } from './topDownRuntime.ts'

export type ObjectCommandStatus = 'selected' | 'routing' | 'approaching' | 'revalidating' | 'executing' | 'completed' | 'cancelled' | 'failed'
/** Session-local receipt. Save serializers deliberately do not include commands. */
export interface ObjectCommand {
  readonly sequence: number
  readonly targetId: string
  readonly actionId: ObjectActionId
  readonly targetAnchor: Anchor
  readonly siteId: string
  readonly shipMode: TopDownState['scoutLocation']['ship']['mode']
  readonly doorState: TopDownState['scoutLocation']['boardingDoor']
  readonly status: ObjectCommandStatus
  readonly executed: boolean
  readonly message: string
}
const terminal = (command: ObjectCommand) => ['completed', 'cancelled', 'failed'].includes(command.status)
const transition = (world: TopDownState, status: ObjectCommandStatus, message = world.objectCommand!.message): TopDownState => ({
  ...world, objectCommand: { ...world.objectCommand!, status, message }, feedback: message,
})
function fail(world: TopDownState, message: string): TopDownState {
  return transition(cancelGroundMovement(world), 'failed', message)
}

/** A menu publication gets one increasing receipt; replay cannot restart work,
 * toggle a door twice, or re-open a completed confirmation panel. */
export function selectObjectCommand(context: ActionContext, targetId: string, actionId: ObjectActionId, sequence: number): TopDownState {
  let { world } = context
  if (!Number.isSafeInteger(sequence) || sequence <= (world.objectCommand?.sequence ?? 0)) return world
  context.session?.cancel()
  world = cancelGroundMovement(world)
  const target = world.entities.find(e => e.id === targetId)
  const command: ObjectCommand = { sequence, targetId, actionId, targetAnchor: target?.anchor ?? world.entities.find(e => e.id === PLAYER_ID)!.anchor,
    siteId: world.scoutLocation.siteId, shipMode: world.scoutLocation.ship.mode, doorState: world.scoutLocation.boardingDoor, status: 'selected', executed: false, message: 'Command selected.' }
  world = { ...world, groundCommand: undefined, objectCommand: command }
  const action = objectActions({ ...context, world, phase: 'selection' }, targetId).find(a => a.id === actionId)
  return action?.available ? world : fail(world, action?.reason ?? 'Target or action unavailable.')
}

/** Bounded deterministic candidates within centre-based reach, outside all
 * colliders. Every candidate route uses #890's authoritative swept planner. */
export function approachRoute(world: TopDownState, targetId: string): readonly Anchor[] | null {
  const target = world.entities.find(e => e.id === targetId)!
  const player = world.entities.find(e => e.id === PLAYER_ID)!
  // Frame ownership changes only through the aft door, even if another
  // outside point lies within an interior fixture's reach radius.
  if (targetId !== BOARDING_DOOR_ID && target.anchor.frame !== player.anchor.frame && world.scoutLocation.boardingDoor === 'closed') return null
  const centre = target.anchor
  const radius = target.interaction!.reachM - .08
  const candidates = Array.from({ length: 32 }, (_, i): Anchor => {
    const angle = i * Math.PI / 16
    return { ...centre, point: translate(centre.point, [radius * Math.cos(angle), radius * Math.sin(angle), 0]) }
  }).filter(anchor => {
    if (targetId !== BOARDING_DOOR_ID && anchor.frame === 'ship:scout') {
      const [x, y] = relativeM(anchor.point, ORIGIN)
      if (Math.abs(x) >= 2.5 || y >= 5.5 || y <= -6) return false
    }
    return groundPositionClear(world, anchor, .025) && (targetId !== BOARDING_DOOR_ID || !doorDenial({ ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e) }))
  })
  candidates.sort((a, b) => Math.hypot(...relativeAnchor(a, player.anchor, world.frames)) - Math.hypot(...relativeAnchor(b, player.anchor, world.frames)))
  for (const candidate of candidates) {
    const route = planGroundRoute(world, candidate)
    if (route) return route
  }
  return null
}

/** Called on the sole fixed tick after the session updates survival/authority.
 * Domain owners still validate their own side effects and atomic revisions. */
export function advanceObjectCommand(context: ActionContext, openEquipment: (world: TopDownState) => boolean = () => false): TopDownState {
  let { world } = context
  const command = world.objectCommand
  if (!command || terminal(command)) return world
  const target = world.entities.find(e => e.id === command.targetId)
  if (context.paused || world.cockpit.view !== 'world') return cancelGroundMovement(world)
  if (!target || target.anchor.frame !== command.targetAnchor.frame || world.scoutLocation.siteId !== command.siteId || world.scoutLocation.ship.mode !== command.shipMode
    || world.frames.ship.parent === null && target.anchor.frame !== 'ship:scout') return fail(world, 'Command failed: target or scene changed.')
  if (Math.hypot(...relativeAnchor(target.anchor, command.targetAnchor, world.frames)) > .01) return fail(world, 'Command failed: target moved.')
  if (command.actionId === 'door' && world.scoutLocation.boardingDoor !== command.doorState) return fail(world, 'Door state changed. Choose the action again.')
  const action = objectActions({ ...context, world, phase: command.status === 'revalidating' || command.status === 'executing' ? 'execution' : 'approach' }, command.targetId).find(a => a.id === command.actionId)
  if (!action?.available) return fail(world, action?.reason ?? 'Command failed: target unavailable.')
  switch (command.status) {
    case 'selected': return transition(world, action.requiresApproach ? 'routing' : 'revalidating')
    case 'routing': {
      const route = approachRoute(world, target.id)
      if (!route?.length) return fail(world, 'No reachable interaction position. Route blocked.')
      // requestGroundMovement cancels any older command; restore this receipt
      // only after a successful new route has been admitted.
      world = requestGroundMovement(world, route[route.length - 1]!)
      if (world.groundCommand?.status !== 'moving') return fail(world, 'Approach unavailable.')
      world = { ...world, groundCommand: { ...world.groundCommand, waypoints: route, objectSequence: command.sequence }, objectCommand: command }
      return transition(world, 'approaching', `Approaching ${target.interaction!.label}. Escape to cancel.`)
    }
    case 'approaching':
      if (world.groundCommand?.status === 'cancelled' || !world.groundCommand) return transition(world, 'cancelled', 'Command cancelled.')
      if (world.groundCommand.status === 'failed') return fail(world, 'Approach blocked. Choose another command.')
      return world.groundCommand.status === 'arrived' ? transition(world, 'revalidating') : world
    case 'revalidating': return transition(world, 'executing')
    case 'executing': {
      if (command.executed) return transition(world, 'completed')
      // Consume before calling any adapter, including synchronous/replayed UI.
      world = { ...world, objectCommand: { ...command, executed: true }, selectedId: target.id }
      const session = context.session
      if (session) { session.world = world; session.feedback = '' }
      let succeeded = true
      if (command.actionId === 'inspect') world = { ...world, feedback: `${target.interaction!.label}. Physical actions require proximity.` }
      else if (['container', 'core', 'fabricator'].includes(command.actionId)) {
        succeeded = !!session && openEquipment(world)
        if (session) world = session.world
      } else if (command.actionId === 'gather') {
        session?.start(target.id)
        succeeded = session?.activeSourceId === target.id
        if (session) world = session.world
      } else if (command.actionId === 'door' && session) {
        succeeded = session.toggleBoardingDoor(session.snapshot.revision)
        world = session.world
      } else {
        world = requestInteraction(world, target.id)
        succeeded = command.actionId === 'cockpit' ? world.cockpit.view === 'radar' : world.scoutLocation.boardingDoor !== context.world.scoutLocation.boardingDoor
      }
      world = { ...world, objectCommand: { ...command, executed: true } }
      return transition(world, succeeded && !session?.error ? 'completed' : 'failed', session?.error || session?.feedback || world.feedback)
    }
  }
  return world
}
