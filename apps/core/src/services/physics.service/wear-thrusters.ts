import { SpaceshipService } from '../spaceship.service';
import { THRUSTER_DURABILITY_DRAIN_RATE } from '../ticking.service/constants';

export function wearThrusters(
  stats: ReturnType<typeof SpaceshipService.normalizeSpaceshipStats>,
  thrustByIndex: readonly number[],
  elapsedSeconds: number,
) {
  if (elapsedSeconds <= 0) return stats;

  return {
    ...stats,
    thrusterDurability: stats.thrusterDurability.map((durability, index) =>
      Math.max(
        0,
        durability -
          ((thrustByIndex[index] ?? 0) / 100) *
            THRUSTER_DURABILITY_DRAIN_RATE *
            elapsedSeconds,
      ),
    ),
  };
}

