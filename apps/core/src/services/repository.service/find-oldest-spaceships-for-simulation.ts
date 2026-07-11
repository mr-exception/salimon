import { cloneSpaceship } from './clone-spaceship';
import { requireSpaceshipsBySecurityCode } from './state';
import { start } from './start';

export async function findOldestSpaceshipsForSimulation(
  invocationTime: Date,
  batchSize: number,
) {
  await start();
  return [...requireSpaceshipsBySecurityCode().values()]
    .filter((spaceship) => {
      const simulatedAt = spaceship.simulatedAt;
      return simulatedAt
        ? simulatedAt < invocationTime
        : spaceship.updatedAt < invocationTime;
    })
    .sort(
      (left, right) =>
        (left.simulatedAt?.getTime() ?? left.updatedAt.getTime()) -
        (right.simulatedAt?.getTime() ?? right.updatedAt.getTime()),
    )
    .slice(0, batchSize)
    .map(cloneSpaceship);
}

