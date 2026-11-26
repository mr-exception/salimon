import type { assessScannerEnvironment } from './scannerEnvironment.ts'

export function ScannerEnvironment({ assessment }: { assessment: ReturnType<typeof assessScannerEnvironment> }) {
  if (!assessment) return <p>Planetary environment unknown.</p>
  const { summary, local, rows } = assessment
  const warning = rows.some(row => row.status === 'Missing protection' || row.status === 'Insufficient protection')
  return <details className="scanner-environment">
    <summary>Planetary environment{warning ? ' · Gear warning' : ''}</summary>
    <section aria-label="Planetary environment and equipment">
      <p>Body-wide temperature range {summary.temperatureRangeC.join(' to ')} °C</p>
      <p>Body baselines · radiation {summary.radiationHpPerSecond} HP/s · toxicity {summary.toxicityHpPerSecond} HP/s</p>
      {local ? <p>Landing site · {local.temperatureC} °C · radiation {local.radiationHpPerSecond} HP/s · toxicity {local.toxicityHpPerSecond} HP/s</p>
        : <p>Landing site conditions unknown.</p>}
      <ul aria-label="Current equipped protection">
        {rows.map(({ label, incoming, available, status }) => <li key={label}>
          {label}: {available} HP/s equipped{incoming === undefined ? '' : ` / ${incoming} HP/s needed`} · {status}
        </li>)}
      </ul>
    </section>
  </details>
}
