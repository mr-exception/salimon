import type { SerializedWorldSystems } from '@repo/types';
import { TickingService } from '../ticking.service';
import { bodyIntersectsCircle } from './body-intersects-circle';
import { getSearchArea } from './get-search-area';
import { groupByOrbitalCenter } from './group-by-orbital-center';
import { resolvePositions } from './resolve-positions';
import { resolveVelocities } from './resolve-velocities';
import type {
  StarSystem,
  VisiblePlanetSystem,
  WorldViewportOptions,
  WorldViewportRequest,
} from './types';
import { withVelocity } from './with-velocity';

export async function getWorldSystems(
  request: WorldViewportRequest,
  options: WorldViewportOptions = {},
): Promise<SerializedWorldSystems> {
  const searchArea = getSearchArea(request);
  const { planets, moons, stars } = await TickingService.getWorldSystemsBodies();
  const orbitingBodies = [...planets, ...moons];
  const positions = resolvePositions([...orbitingBodies, ...stars]);
  const center = { x: searchArea.x, y: searchArea.y };
  const planetsByOrbitalCenter = groupByOrbitalCenter(orbitingBodies);
  const velocities = resolveVelocities([...orbitingBodies, ...stars], positions);
  const requiredBodyNames = new Set(options.requiredBodyNames ?? []);
  const systems: StarSystem[] = stars
    .map((star) => {
      const isStarRequired = requiredBodyNames.has(star.name);
      const isStarVisible = bodyIntersectsCircle(
        star,
        positions.get(star.name)!,
        center,
        searchArea.radius,
      );
      const planetSystems = (planetsByOrbitalCenter.get(star.name) ?? [])
        .map((planet) => {
          const visibleMoons = (
            planetsByOrbitalCenter.get(planet.name) ?? []
          ).filter(
            (moon) =>
              requiredBodyNames.has(moon.name) ||
              bodyIntersectsCircle(
                moon,
                positions.get(moon.name)!,
                center,
                searchArea.radius,
              ),
          );
          const isPlanetRequired = requiredBodyNames.has(planet.name);
          const isPlanetVisible = bodyIntersectsCircle(
            planet,
            positions.get(planet.name)!,
            center,
            searchArea.radius,
          );

          return isPlanetRequired || isPlanetVisible || visibleMoons.length > 0
            ? {
                planet,
                moons: visibleMoons,
              }
            : undefined;
        })
        .filter(
          (system): system is VisiblePlanetSystem => system !== undefined,
        );
      const hasVisibleBody =
        isStarRequired || isStarVisible || planetSystems.length > 0;

      return hasVisibleBody
        ? {
            star: withVelocity(star, velocities),
            planets: planetSystems.map(({ planet, moons: planetMoons }) => ({
              planet: withVelocity(planet, velocities),
              moons: planetMoons.map((moon) => withVelocity(moon, velocities)),
            })),
          }
        : undefined;
    })
    .filter((system): system is StarSystem => system !== undefined);

  return { systems } as unknown as SerializedWorldSystems;
}

