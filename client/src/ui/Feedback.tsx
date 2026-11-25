import { createContext, useContext, type ReactNode } from 'react'
import { notificationCue, orderedFeedback, type FeedbackMessage, type FeedbackPriority, type NotificationTone } from './feedback.ts'

const CriticalFeedback = createContext<readonly FeedbackMessage[]>([])
export function FeedbackScope({ messages, children }: { messages: readonly FeedbackMessage[]; children: ReactNode }) {
  const parent = useContext(CriticalFeedback)
  return <CriticalFeedback.Provider value={orderedFeedback([...parent, ...messages])}>{children}</CriticalFeedback.Provider>
}
function NotificationCue({ tone }: { tone: NotificationTone }) {
  const [icon, label] = notificationCue[tone]
  return <><span className="ui-notification-icon" aria-hidden="true">{icon}</span>{label}: </>
}
export function StatusMessage({ children, priority = 'status', tone, label, announce = true }: { children: string; priority?: FeedbackPriority; tone?: NotificationTone; label?: string; announce?: boolean }) {
  return <p className={`ui-status ui-status--${priority}${tone ? ` ui-notification ui-notification--${tone}` : ''}`} role={announce ? 'status' : undefined} aria-label={label} aria-live={announce ? priority === 'critical' || priority === 'error' ? 'assertive' : 'polite' : 'off'} aria-atomic="true">
    {children && <>{tone ? <NotificationCue tone={tone}/> : priority === 'critical' ? 'Warning: ' : priority === 'error' ? 'Error: ' : ''}{children}</>}
  </p>
}
/** Mirror persistent conditions inside native modal top layers (background is inert).
 * The announcement is a stable string, so frame updates do not mutate live text. */
export function WarningSummary({ id }: { id?: string }) {
  const messages = useContext(CriticalFeedback)
  return <div id={id} className="ui-status ui-status--critical ui-warning-summary" role="status" aria-label="Persistent warnings" aria-live="assertive" aria-atomic="true">
    {orderedFeedback(messages).map(message => <p key={message.text} className={message.tone ? `ui-notification ui-notification--${message.tone}` : undefined}><span>{message.tone ? <NotificationCue tone={message.tone}/> : message.priority === 'error' ? 'Error: ' : 'Warning: '}</span><span>{message.text}</span></p>)}
  </div>
}
