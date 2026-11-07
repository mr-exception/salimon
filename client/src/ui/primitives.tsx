import { useEffect, useId, useRef, type ButtonHTMLAttributes, type ReactNode } from 'react'
import { meterValue } from './values.ts'
import './tokens.css'
import './primitives.css'

type Feedback = 'neutral' | 'pending' | 'error' | 'confirmation' | 'warning'
type ButtonProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'disabled'> & {
  disabledReason?: string
  pending?: boolean
  variant?: 'primary' | 'secondary' | 'danger'
  active?: boolean
}

// aria-disabled keeps the explanation reachable by keyboard. Activation is blocked
// here rather than relying on CSS; domain owners must still validate commands.
export function Button({ disabledReason, pending = false, variant = 'primary', active, children, onClick, className = '', type = 'button', ...props }: ButtonProps) {
  const reasonId = useId()
  const blocked = Boolean(disabledReason) || pending
  return <span className="ui-control">
    <button {...props} type={type} className={`ui-button ui-button--${variant} ${className}`}
      aria-disabled={blocked || undefined} aria-busy={pending || undefined} aria-pressed={active}
      aria-describedby={[props['aria-describedby'], disabledReason ? reasonId : null].filter(Boolean).join(' ') || undefined}
      onClick={event => { if (blocked) { event.preventDefault(); return }; onClick?.(event) }}>
      {pending && <span aria-hidden="true">… </span>}{children}
    </button>
    {disabledReason && <small className="ui-reason" id={reasonId}>{disabledReason}</small>}
  </span>
}

export function IconButton({ label, icon, ...props }: Omit<ButtonProps, 'children' | 'aria-label'> & { label: string; icon: ReactNode }) {
  return <Button {...props} aria-label={label} className={`ui-icon-button ${props.className ?? ''}`}><span aria-hidden="true">{icon}</span></Button>
}

function Panel({ title, eyebrow, children, kind }: { title: string; eyebrow?: string; children: ReactNode; kind: 'equipment' | 'utility' }) {
  const titleId = useId()
  return <section className={`ui-panel ui-panel--${kind}`} aria-labelledby={titleId}>
    {eyebrow && <p className="ui-eyebrow">{eyebrow}</p>}<h2 id={titleId}>{title}</h2>{children}
  </section>
}
type PanelProps = Omit<Parameters<typeof Panel>[0], 'kind'>
export const EquipmentPanel = (props: PanelProps) => <Panel {...props} kind="equipment" />
export const UtilityPanel = (props: PanelProps) => <Panel {...props} kind="utility" />

// Native modal supplies inert background, focus containment and Escape semantics.
// Render within .expedition-ui so the top-layer dialog inherits the same tokens.
type DialogProps = { open: boolean; title: string; children: ReactNode; onDismiss: () => void; eyebrow?: string; status?: ReactNode; feedback?: ReactNode; actions?: ReactNode; className?: string }
export function Dialog({ open, title, children, onDismiss, eyebrow, status, feedback, actions, className = '' }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null)
  const titleId = useId()
  const dismiss = useRef(onDismiss)
  dismiss.current = onDismiss
  useEffect(() => {
    const dialog = ref.current!
    if (!open) return
    const previous = document.activeElement
    dialog.showModal()
    dialog.focus()
    return () => { dialog.close(); if (previous instanceof HTMLElement && previous.isConnected) previous.focus() }
  }, [open])
  return <dialog ref={ref} className={`ui-dialog ${className}`} tabIndex={-1} aria-labelledby={titleId}
    onCancel={event => { event.preventDefault(); dismiss.current() }}
    onKeyDown={event => {
      event.stopPropagation()
      if (event.key !== 'Tab') return
      const controls = Array.from(event.currentTarget.querySelectorAll<HTMLElement>(
        'button:not(:disabled),input:not(:disabled),select:not(:disabled),textarea:not(:disabled),a[href],[tabindex]:not([tabindex="-1"])',
      )).filter(element => element.getClientRects().length > 0 && !element.closest('[inert]'))
      const first = controls[0], last = controls.at(-1)
      if (!first) { event.preventDefault(); event.currentTarget.focus(); return }
      if (event.shiftKey && (document.activeElement === first || document.activeElement === event.currentTarget)) {
        event.preventDefault(); last?.focus()
      } else if (!event.shiftKey && (document.activeElement === last || document.activeElement === event.currentTarget)) {
        event.preventDefault(); first.focus()
      }
    }} onKeyUp={event => event.stopPropagation()}>
    <header className="ui-dialog-identity">{eyebrow && <p className="ui-eyebrow">{eyebrow}</p>}
      <h2 id={titleId}>{title}</h2>{status}</header>
    <div className="ui-dialog-content">{children}</div>
    {feedback && <div className="ui-dialog-feedback">{feedback}</div>}
    <div className="ui-dialog-actions">{actions}<Button variant="secondary" onClick={onDismiss}>Close {title}</Button></div>
  </dialog>
}

/** Presentation regions only: machine owners retain all draft/commit authority. */
export function EquipmentDialog(props: Omit<DialogProps, 'className'>) {
  return <Dialog {...props} className="ui-dialog--equipment" />
}

export function Meter({ label, value, max, unit }: { label: string; value: number; max: number; unit: string }) {
  const reading = meterValue(value, max)
  return <div className="ui-meter"><span>{label}: {reading.value} / {reading.max} {unit}</span>
    <meter aria-label={label} min={0} max={reading.max} value={reading.value} aria-valuetext={`${reading.value} / ${reading.max} ${unit}`} />
  </div>
}
export function Progress({ label, value, max, unit }: Parameters<typeof Meter>[0]) {
  const reading = meterValue(value, max)
  return <div className="ui-meter"><span>{label}: {reading.value} / {reading.max} {unit}</span>
    <progress aria-label={label} max={reading.max} value={reading.value} aria-valuetext={`${reading.value} / ${reading.max} ${unit}`} />
  </div>
}
export const KeyHint = ({ children }: { children: ReactNode }) => <kbd className="ui-key">{children}</kbd>
export function ContextPrompt({ binding, children }: { binding: string; children: ReactNode }) {
  return <p className="ui-prompt"><KeyHint>{binding}</KeyHint> {children}</p>
}
// Callers change messages only on meaningful events, never every simulation tick.
export function StatusLabel({ tone = 'neutral', children }: { tone?: Feedback; children: ReactNode }) {
  return <p className={`ui-status ui-status--${tone}`} role={tone === 'error' || tone === 'warning' ? 'alert' : 'status'}>{children}</p>
}
