import type { SpaceshipDocument } from '@models';
import { RepositoryService } from '../repository.service';
import { tickingState } from './state';

export async function startSpaceshipTargetSpeedFeature(
  spaceship: SpaceshipDocument,
  params: {
    targetSpeedMetersPerSecond: number;
    maximumThrustPercent: number;
    targetDirection?: number;
  },
) {
  if (spaceship.motionState === 'crashed') return undefined;

  const simulatedAt = new Date();
  const result = tickingState.sandbox?.startSpaceshipTargetSpeed(
    spaceship.securityCode,
    params,
    simulatedAt.getTime(),
  );
  if (!result?.snapshot) return undefined;

  return RepositoryService.updateSpaceshipBySecurityCode(
    spaceship.securityCode,
    {
      activeFeature: {
        type: 'target-speed',
        ...result.plan,
      },
      motionState: 'flying',
      position: result.snapshot.position,
      velocity: result.snapshot.velocity,
      speed: result.snapshot.speed,
      direction: result.snapshot.direction,
      simulatedAt,
      updatedAt: simulatedAt,
    },
  );
}
