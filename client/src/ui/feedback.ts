export type FeedbackPriority = 'critical' | 'error' | 'pending' | 'status' | 'confirmation'
export interface FeedbackMessage { id: string; text: string; priority: FeedbackPriority }
const rank: Record<FeedbackPriority, number> = { critical: 0, error: 1, pending: 2, status: 3, confirmation: 4 }
/** Stable owner identity and text, never frame counters or progress readings. */
export function orderedFeedback(messages: readonly FeedbackMessage[]): FeedbackMessage[] {
  const texts = new Set<string>()
  return [...messages].sort((a, b) => rank[a.priority] - rank[b.priority]).filter(message => {
    if (!message.text.trim() || texts.has(message.text)) return false
    texts.add(message.text); return true
  })
}
export function announcement(messages: readonly FeedbackMessage[]): string {
  return orderedFeedback(messages).map(message => `${message.priority === 'critical' ? 'Warning: ' : message.priority === 'error' ? 'Error: ' : ''}${message.text}`).join(' ')
}
