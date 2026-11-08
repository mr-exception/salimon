import { useState } from 'react'
import { createRoot } from 'react-dom/client'
import { Button, IconButton, EquipmentPanel, UtilityPanel, Dialog, Meter, Progress, ContextPrompt, StatusLabel } from '../src/ui/primitives.tsx'
import { acceptsFlightShortcut } from '../src/input.ts'
import './ui-fixture.css'

// A bubbling game shortcut sentinel verifies that modal/input events never escape.
Object.assign(window, { shortcutCount: 0 })
window.addEventListener('keydown', event => {
  if (acceptsFlightShortcut(event.target) && event.code === 'KeyW') {
    const state = window as unknown as { shortcutCount: number }; state.shortcutCount++
  }
})
function Gallery({ context }: { context: 'planet' | 'space' }) {
  const [open, setOpen] = useState(false)
  const [nested, setNested] = useState(false)
  const [clicks, setClicks] = useState(0)
  return <main className={`expedition-ui fixture fixture--${context}`} aria-label={`${context} context`}>
    <h1>Expedition Interface / {context}</h1>
    <p>Component fixture · illustrative readings · no gameplay commands</p>
    <div className="fixture-grid">
      <EquipmentPanel eyebrow="Scout / equipment" title="Shared controls">
        <div className="fixture-actions">
          <Button onClick={() => setOpen(true)}>Inspect equipment</Button>
          <Button variant="secondary">Secondary action</Button>
          <Button active>Instrument on</Button>
          <IconButton label="Inspect settings" icon="⚙" />
          <Button disabledReason="Grounded: takeoff unavailable" onClick={() => setClicks(clicks + 1)}>Take off</Button>
          <Button pending onClick={() => setClicks(clicks + 1)}>Saving</Button>
          <Button variant="danger" onClick={() => setOpen(true)}>Review removal</Button>
        </div>
        <p>Blocked activations: <output>{clicks}</output></p>
        <StatusLabel tone="pending">Pending — waiting for committed result</StatusLabel>
        <StatusLabel tone="error">Rejected — capacity exceeded</StatusLabel>
        <StatusLabel tone="confirmation">Confirmed — transfer committed</StatusLabel>
        <StatusLabel tone="warning">Warning — low reserve</StatusLabel>
      </EquipmentPanel>
      <UtilityPanel title="Instrument readings">
        <Meter label="Health" value={160} max={200} unit="HP" />
        <Meter label="Carried" value={4.25} max={10} unit="kg" />
        <Progress label="Scan" value={3} max={5} unit="s" />
        <ContextPrompt binding="E">Inspect nearby equipment</ContextPrompt>
        <label>Instrument label <input aria-label="Instrument label" defaultValue="Scout" /></label>
        <StatusLabel>Empty — no scanned contacts</StatusLabel>
        <p className="fixture-axes">Reserved compass: <span>Heading</span> / <span>Velocity</span></p>
      </UtilityPanel>
    </div>
    <Dialog open={open} title="Equipment review" onDismiss={() => setOpen(false)}>
      <Dialog open={nested} title="Nested details" onDismiss={() => setNested(false)}><p>Nested utility details</p></Dialog>
      <p>Review the authoritative result before confirming an operation.</p>
      <label>Quantity (g) <input aria-label="Quantity (g)" defaultValue="250" /></label>
      <Button onClick={() => setNested(true)}>Inspect nested details</Button>
      <StatusLabel tone="error">Rejected — example validation error</StatusLabel>
      <StatusLabel tone="confirmation">Confirmed — example committed result</StatusLabel>
      <Button pending>Waiting for result</Button>
      <Button disabledReason="No available capacity">Confirm transfer</Button>
    </Dialog>
  </main>
}
createRoot(document.getElementById('root')!).render(<><Gallery context="planet" /><Gallery context="space" /></>)
