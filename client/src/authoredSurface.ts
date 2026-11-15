/** Physical baseline only. Artwork consumes these anchors/boxes; saves contain
 * local anchor/removal deltas and resourceSession owns source depletion. */
export const AUTHORED_SURFACE = Object.freeze({
  siteId: 'site:illustrated-landing:v1', bodyId: 'vesper',
  spawn: [-6, 3], ship: [2, -1, -.12], bounds: [18, 14],
  objects: [
    ['rock:west', -10, -3, 1.7, 1.4], ['rock:east', 11, 5, 1.7, 1.4],
    ['rock:north', -3, -10, 1.7, 1.4], ['rock:south', -12, 10, 1.7, 1.4], ['rock:rim', 13, -7, 1.7, 1.4],
    ['stones:west', -14, 4, .5, .4], ['stones:north', -7, -6, .5, .4], ['stones:east', 8, -8, .5, .4], ['stones:south', 7, 10, .5, .4],
    ['source:landing:mineral:01', -9, 0, 1.2, .9], ['source:landing:biology:01', -5, 6, .9, .85],
    ['flora:west', -14, -1, .9, .85], ['flora:south', -9, 9, .9, .85], ['flora:east', 10, -3, .9, .85],
  ] as const,
} as const)
// Deep immutability also protects callers using plain JavaScript.
for (const value of [AUTHORED_SURFACE.spawn, AUTHORED_SURFACE.ship, AUTHORED_SURFACE.bounds, ...AUTHORED_SURFACE.objects, AUTHORED_SURFACE.objects]) Object.freeze(value)
