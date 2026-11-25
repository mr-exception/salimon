import type { TopDownState } from './topDownRuntime.ts'

/** Presentation only: receipts and domain work keep their existing owners. */
export function commandFeedback(world: TopDownState, activeSourceId?: string | null, workFeedback = '') {
  const command = world.objectCommand, route = world.groundCommand
  const label = (id: string) => world.entities.find(e => e.id === id)?.interaction?.label ?? 'target'
  if (activeSourceId) return { text: `Mining ${label(activeSourceId)}`, cancellable: true, failed: false }
  if (command && !['completed', 'cancelled', 'failed'].includes(command.status)) {
    const verb = command.status === 'selected' ? 'Queued' : command.status === 'routing' ? 'Planning route to' : command.status === 'approaching' ? 'Moving to' : command.status === 'revalidating' ? 'Checking' : 'Executing action on'
    return { text: `${verb} ${label(command.targetId)}`, cancellable: true, failed: false }
  }
  if (route?.status === 'moving') return { text: 'Moving to destination', cancellable: true, failed: false }
  if (command && (!route || route.objectSequence === command.sequence)) {
    if (command.status === 'completed' && command.actionId === 'gather') return { text: workFeedback || 'Mining stopped.', cancellable: false, failed: /full|depleted|unavailable|blocked/i.test(workFeedback) }
    return { text: `${command.status === 'completed' ? 'Completed · ' : ''}${command.message}`, cancellable: false, failed: command.status === 'failed' }
  }
  if (route) return { text: route.status === 'failed' ? 'Destination unreachable or route blocked. Choose another destination.' : route.status === 'arrived' ? 'Arrived at destination.' : 'Movement cancelled.', cancellable: false, failed: route.status === 'failed' }
  return null
}
