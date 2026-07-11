import type { WorldBodyDocument } from '@models';

export function groupByOrbitalCenter(bodies: WorldBodyDocument[]) {
  const bodiesByOrbitalCenter = new Map<string | null, WorldBodyDocument[]>();

  for (const body of bodies) {
    const siblings = bodiesByOrbitalCenter.get(body.orbitalCenter) ?? [];
    siblings.push(body);
    bodiesByOrbitalCenter.set(body.orbitalCenter, siblings);
  }

  return bodiesByOrbitalCenter;
}

