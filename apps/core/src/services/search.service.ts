import { RepositoryService } from './repository.service';

const DEFAULT_LIMIT = 20;
const PLANET_NAVIGATION_ZOOM = 0.0000001 * 10;
const STAR_NAVIGATION_ZOOM = 0.00000000001 * 10;

type SearchKind = 'planet' | 'moon' | 'star';

export type SearchResult = {
  name: string;
  kind: SearchKind;
  navigationZoom: number;
};

export class SearchService {
  static async searchByName(query: unknown) {
    const normalizedQuery = SearchService.parseQuery(query);
    if (!normalizedQuery) return [];

    const world = await RepositoryService.getWorldSystemsBodies();
    return [
      ...world.planets.map((body) => ({
        name: body.name,
        kind: 'planet' as const,
        navigationZoom: PLANET_NAVIGATION_ZOOM,
      })),
      ...world.moons.map((body) => ({
        name: body.name,
        kind: 'moon' as const,
        navigationZoom: PLANET_NAVIGATION_ZOOM,
      })),
      ...world.stars.map((body) => ({
        name: body.name,
        kind: 'star' as const,
        navigationZoom: STAR_NAVIGATION_ZOOM,
      })),
    ]
      .filter((result) =>
        result.name.toLocaleLowerCase().includes(normalizedQuery),
      )
      .sort((left, right) => {
        const leftName = left.name.toLocaleLowerCase();
        const rightName = right.name.toLocaleLowerCase();
        const leftStartsWith = leftName.startsWith(normalizedQuery);
        const rightStartsWith = rightName.startsWith(normalizedQuery);

        return (
          Number(rightStartsWith) - Number(leftStartsWith) ||
          left.name.localeCompare(right.name)
        );
      })
      .slice(0, DEFAULT_LIMIT) satisfies SearchResult[];
  }

  private static parseQuery(query: unknown) {
    if (typeof query !== 'string') return '';
    return query.trim().toLocaleLowerCase();
  }
}
