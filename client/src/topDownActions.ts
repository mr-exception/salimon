import { isCockpitTarget, LOCAL_PLAYER } from './cockpit.ts'
import { BOARDING_DOOR_ID, doorDenial } from './scoutDoor.ts'
import { RESOURCE_CATALOG, type ResourceSession } from './resourceSession.ts'
import { resourceInventoryUsage } from './resourceInventory.ts'
import { survivalPolicy } from './survival.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { drawList, PLAYER_ID, type TopDownState } from './topDownRuntime.ts'
import type { Anchor } from './topDownFrames.ts'

export type ObjectActionId = 'inspect' | 'gather' | 'container' | 'core' | 'fabricator' | 'cockpit' | 'door'
export interface ActionContext { world: TopDownState; session?: ResourceSession | null; paused: boolean }
export interface ObjectAction {
  id: ObjectActionId
  label: string
  targetId: string
  targetFrame: Anchor['frame']
  rangeM: number | null
  available: boolean
  reason: string | null
  preconditions: readonly string[]
  interrupt: 'replace-command'
  execute: (handlers: ActionHandlers) => void
}
/** Adapters retain domain mutation and explicit equipment confirmation ownership. */
export interface ActionHandlers { inspect: (id: string) => void; interact: (id: string, action: ObjectActionId) => void }
export function objectActions(context: ActionContext, id: string): ObjectAction[] {
  const { world, session, paused } = context
  const target = drawList(world).find(item => item.entity.id === id)?.entity
  if (!target?.interaction || world.cockpit.view !== 'world') return []
  const common = paused ? 'Resume the expedition first.' : world.scoutLocation.ship.mode === 'destroyed' ? 'Journey ended.' : session?.error || (session?.survival.health === 0 ? 'Run ended.' : null)
  const player = world.entities.find(e => e.id === PLAYER_ID)!
  const reach = interactionInReach(player.anchor, target, world.frames)
  const nearby = !reach ? 'Move closer using right-click ground or WASD.' : world.player.mode === 'seated' ? 'Stand up first.' : null
  const action = (actionId: ObjectActionId, label: string, reason: string | null, physical = true, preconditions: readonly string[] = []): ObjectAction => ({
    id: actionId, label, targetId: id, targetFrame: target.anchor.frame,
    rangeM: physical ? target.interaction!.reachM : null,
    available: !(common || reason), reason: common || reason,
    preconditions: ['visible target', 'active expedition', ...preconditions], interrupt: 'replace-command',
    execute: handlers => actionId === 'inspect' ? handlers.inspect(id) : handlers.interact(id, actionId),
  })
  // Remote inspection is presentation-only: never query geological truth,
  // register a source, or call the nearby discovery/notebook APIs.
  const actions = [action('inspect', 'Inspect', null, false)]
  const work = session && !survivalPolicy(session.survival).canWork ? 'Too exhausted to work.' : null
  if (id === BOARDING_DOOR_ID) actions.push(action('door', world.scoutLocation.boardingDoor === 'open' ? 'Close door' : 'Open door', nearby || doorDenial(world), true, ['door unlocked', 'doorway clear']))
  else if (isCockpitTarget(id) && world.cockpit.enabled) actions.push(action('cockpit', 'Use cockpit', nearby || (!world.cockpit.powered ? 'Cockpit has no power.' : world.cockpit.ownerId !== LOCAL_PLAYER || world.cockpit.occupantId !== null && world.cockpit.occupantId !== LOCAL_PLAYER ? 'Cockpit requires owner control and a free chair.' : player.anchor.frame !== world.frames.ship.id ? 'Enter the Scout first.' : work), true, ['powered', 'owner', 'free chair']))
  else if (session && ['scout:storage', 'scout:core', 'scout:fabricator'].includes(id)) {
    const kind = id === 'scout:storage' ? 'container' : id === 'scout:core' ? 'core' : 'fabricator'
    actions.push(action(kind, kind === 'container' ? 'Use Resource Container' : kind === 'core' ? 'Use Energy Core' : 'Use Fabricator', nearby, true, ['proximity', 'explicit panel confirmation']))
  } else if (session && target.source) {
    const source = session.source(id), inventory = session.snapshot.inventory
    const usage = resourceInventoryUsage(inventory, RESOURCE_CATALOG, session.itemCatalog)
    const reason = nearby || work || (!session.equipped ? 'Equip the gathering tool first.' : !source || source.remainingGrams === 0 ? 'Source depleted.' : usage.grams >= inventory.capacityGrams ? 'Inventory full: weight limit.' : !inventory.quantities[source.resourceId] && usage.slots >= inventory.capacitySlots ? 'Inventory full: slot limit.' : null)
    actions.push(action('gather', 'Gather / Mine', reason, true, ['equipped tool', 'remaining source', 'inventory capacity', 'work policy']))
  }
  return actions
}
/** Rebuild on invocation; a retained menu row never grants stale authority. */
export function executeObjectAction(context: ActionContext, targetId: string, actionId: ObjectActionId, handlers: ActionHandlers): boolean {
  const action = objectActions(context, targetId).find(action => action.id === actionId)
  if (!action?.available) return false
  action.execute(handlers)
  return true
}
