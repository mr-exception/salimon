import type { ResourceSession } from '../resourceSession.ts'
import type { Entity2D } from '../topDownRuntime.ts'

const machines = {
  'scout:core': { kind: 'core', quarterTurns: 3 },
  'scout:fabricator': { kind: 'fabricator', quarterTurns: 1 },
  'scout:gear-bench': { kind: 'bench', quarterTurns: 3 },
} as const
type Machine = typeof machines[keyof typeof machines]
type MachineState = 'idle' | 'active' | 'paused' | 'unpowered' | 'output'

export function scoutMachine(entity: Entity2D): Machine | undefined {
  return machines[entity.id as keyof typeof machines]
}

export function machineState(machine: Machine, session: ResourceSession | null | undefined, paused: boolean): MachineState {
  if (!session) return 'idle'
  if (machine.kind === 'core') {
    const { generatedJ, drawnJ60, status } = session.corePower
    if (!session.snapshot.core.reserveJ60 && (status === 'empty' || session.corePowerTick === null)) return 'unpowered'
    if (generatedJ || drawnJ60) return paused || session.error ? 'paused' : 'active'
    return 'idle'
  }
  const { tray, job } = session.snapshot[machine.kind]
  if (tray) return 'output'
  if (!job) return session.snapshot.core.reserveJ60 ? 'idle' : 'unpowered'
  return paused || session.error || job.powerPaused ? 'paused' : 'active'
}

function mounting(entity: Entity2D, { quarterTurns }: Machine) {
  const { halfWidth, halfHeight } = entity.footprint!
  const sideMounted = quarterTurns % 2 === 1
  const [width, depth] = sideMounted ? [halfHeight, halfWidth] : [halfWidth, halfHeight]
  return `rotate(${quarterTurns * 90}) scale(${width} ${depth})`
}

/** Forward points toward the service edge at local −Y. */
export function ScoutMachineBody({ entity, machine }: { entity: Entity2D; machine: Machine }) {
  return <g transform={mounting(entity, machine)} stroke="#283b43" strokeWidth=".07" strokeLinejoin="round">
    {machine.kind === 'core' && <>
      <path d="M-.93 -.73 -.68 -.95H.68L.93 -.73V.78L.69 .94H-.69L-.93 .78Z" fill="#53676d" />
      <ellipse cy=".13" rx=".65" ry=".65" fill="#a1aaa2" />
      <ellipse cy=".13" rx=".48" ry=".51" fill="#384f56" />
      <ellipse cy=".13" rx=".26" ry=".3" fill="#738f91" />
      <path d="M-.84 -.52H-.55M.55 -.52H.84M-.8 .67h.21m1.18 0h.21" stroke="#b6a074" strokeWidth=".11" />
      <path d="M-.32 -.7H.32" stroke="#22363d" strokeWidth=".25" />
    </>}
    {machine.kind === 'fabricator' && <>
      <path d="M-.95 -.79 -.82 -.92H.82L.95 -.79V.84L.82 .95H-.82L-.95 .84Z" fill="#71848a" />
      <rect x="-.74" y="-.24" width="1.48" height=".99" rx=".12" fill="#273e46" />
      <path d="M-.64 -.12H.63v.67H-.64Z" fill="#536b71" />
      <path d="M-.65 .55H.68M-.54 -.57H.54" stroke="#a9b6af" strokeWidth=".11" />
      <path d="M-.59 -.79H.59" stroke="#b79665" strokeWidth=".13" />
      <path d="M-.82 .74h.18m1.28 0h.18" stroke="#333f43" strokeWidth=".09" />
    </>}
    {machine.kind === 'bench' && <>
      <path d="M-.96 -.88H.96V.8L.82 .93H-.82L-.96 .8Z" fill="#b7b7a1" />
      <path d="M-.81 .24H.79V.78H-.81Z" fill="#506269" />
      <path d="M-.74 .47h.28m.22 0h.22m.22 0h.42" stroke="#bdc9bf" strokeWidth=".2" />
      <path d="M-.82 -.59h.3v.45h-.3Zm1.34 .08h.29v.44H.52Z" fill="#8a6f51" />
      <path d="M-.33 -.57H.33v.38h-.66Z" fill="#788780" />
    </>}
  </g>
}

const lamp: Record<MachineState, string> = {
  active: '#7bced0', idle: '#587075', paused: '#d1a160', unpowered: '#273b41', output: '#e2c18b',
}

export function ScoutMachineSignal({ entity, machine, state }: {
  entity: Entity2D; machine: Machine; state: MachineState
}) {
  return <g transform={mounting(entity, machine)} data-scout-machine={machine.kind}
    data-machine-state={state} data-machine-rotation={machine.quarterTurns * 90} pointerEvents="none">
    <path d={machine.kind === 'core' ? 'M-.24 -.7H.24' : 'M-.20 -.70H.20'}
      fill="none" stroke={lamp[state]} strokeWidth=".105" strokeLinecap="round" />
    {state === 'output' && <path d="M-.30 -.47h.60" stroke="#e2c18b" strokeWidth=".18" />}
  </g>
}
