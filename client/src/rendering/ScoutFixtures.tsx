import { SCOUT_RESOURCE_CONTAINER_ID } from '../scoutResourceContainer.ts'
import type { Entity2D } from '../topDownRuntime.ts'

const fixtures = {
  [SCOUT_RESOURCE_CONTAINER_ID]: { kind: 'storage', rotation: 270 },
  'scout:desk': { kind: 'desk', rotation: 180 },
  'scout:chair': { kind: 'seat', rotation: 0 },
} as const

export type ScoutFixture = typeof fixtures[keyof typeof fixtures]

export function scoutFixture(entity: Entity2D): ScoutFixture | undefined {
  return fixtures[entity.id as keyof typeof fixtures]
}

function mounting(entity: Entity2D, { rotation }: ScoutFixture) {
  const { halfWidth, halfHeight } = entity.footprint!
  const [width, depth] = rotation % 180 ? [halfHeight, halfWidth] : [halfWidth, halfHeight]
  return `rotate(${rotation}) scale(${width} ${depth})`
}

export function ScoutFixtureBody({ entity, fixture }: { entity: Entity2D; fixture: ScoutFixture }) {
  return <g transform={mounting(entity, fixture)} stroke="#35464a" strokeWidth=".07" strokeLinejoin="round">
    {fixture.kind === 'storage' && <>
      <path d="M-.92 -.81 -.8 -.94H.8L.92 -.81V.81L.78 .94H-.78L-.92 .81Z" fill="#6e7b79" />
      <path d="M-.78 -.78H.05V.79H-.78Z" fill="#9c9e86" />
      <path d="M-.68 -.66H-.07V.61H-.68Z" fill="#aeb09c" />
      <path d="M-.69 -.3H-.04m-.65 .55H-.04" stroke="#586b68" strokeWidth=".12" />
      <path d="M-.55 -.67H-.21" stroke="#c8a474" strokeWidth=".12" />
      <path d="M.14 -.78H.79V.79H.14Z" fill="#414f54" />
      <path d="M.24 -.68H.71V-.05H.24Z" fill="#c6c3aa" />
      <path d="M.24 .04H.71V.65H.24Z" fill="#8a9b97" />
      <path d="M.29 -.53H.66M.29 .18H.66M.34 .45H.61" stroke="#485a5d" strokeWidth=".10" />
    </>}
    {fixture.kind === 'desk' && <>
      <path d="M-.97 -.58 -.68 -.90H.68L.97 -.58V.82H-.97Z" fill="#54646e" />
      <path d="M-.83 -.46 -.57 -.71H.57L.83 -.46V.60H-.83Z" fill="#293d49" />
      <path d="M-.42 -.40H.42V.13H-.42Z" fill="#516c78" />
      <path d="M-.76 -.32H-.53V.08H-.76Zm1.29 0H.76V.08H.53Z" fill="#829694" />
      <path d="M-.86 .43H.86M-.41 -.55H.41" stroke="#839591" strokeWidth=".12" />
      <path d="M-.78 .65H-.61M.61 .65H.78" stroke="#b08f66" strokeWidth=".12" />
    </>}
    {fixture.kind === 'seat' && <>
      <path d="M-.73 -.71H.73L.87 .52 .47 .88H-.47L-.87 .52Z" fill="#344753" />
      <path d="M-.50 -.63H.50L.58 .39 .35 .57H-.35L-.58 .39Z" fill="#8f7d68" />
      <path d="M-.56 .42H.56V.74H-.56Z" fill="#b6a087" />
      <path d="M-.73 -.50V.37M.73 -.50V.37" stroke="#859795" strokeWidth=".16" />
      <path d="M-.33 -.42H.33M-.34 -.16H.34M-.35 .10H.35" stroke="#a4947d" strokeWidth=".08" />
    </>}
  </g>
}

export function ScoutFixtureSignal({ entity, fixture, rawAccess, equipmentAccess, stocked, gearStocked, occupied, powered }: {
  entity: Entity2D; fixture: ScoutFixture; rawAccess: boolean; equipmentAccess: boolean; stocked: boolean; gearStocked: boolean;
  occupied: boolean; powered: boolean
}) {
  const state = fixture.kind === 'storage' ? rawAccess ? 'raw-access' : equipmentAccess ? 'equipment-access' : stocked ? 'loaded' : 'empty'
    : fixture.kind === 'seat' ? occupied ? 'occupied' : 'vacant'
    : powered ? 'powered' : 'unpowered'
  return <g transform={mounting(entity, fixture)} data-scout-fixture={fixture.kind}
    data-fixture-state={state} data-fixture-rotation={fixture.rotation} pointerEvents="none">
    {fixture.kind === 'storage' && <>
      <path d="M-.64 -.51H-.14" stroke={rawAccess ? '#8bd6d2' : stocked ? '#ceb992' : '#828b7c'} strokeWidth=".10" />
      <path d="M.34 -.67H.60" stroke={equipmentAccess ? '#8bd6d2' : gearStocked ? '#ceb992' : '#a4ad9e'} strokeWidth=".08" />
    </>}
    {fixture.kind === 'desk' && powered && <path d="M-.36 -.28H.36M-.27 -.04H.27" stroke="#80b7bb" strokeWidth=".11" />}
    {fixture.kind === 'seat' && <path d="M-.22 .71H.22" stroke={occupied ? '#c7a878' : '#7b8e8c'} strokeWidth=".10" />}
  </g>
}
