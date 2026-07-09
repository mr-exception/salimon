import { type SerializedPosition, type WorldBodyDocument } from '@models';
import { RepositoryService } from './repository.service';

const FULL_ROTATION_RADIANS = Math.PI * 2;
const SIMULATION_INTERVAL_MS = 1_000;

type Timer = ReturnType<typeof setInterval>;

function advancePosition(
  body: WorldBodyDocument,
  elapsedSeconds: number,
): SerializedPosition {
  const x = BigInt(body.position.x);
  const y = BigInt(body.position.y);
  const orbitalRadius = Math.hypot(Number(x), Number(y));
  const speed = Number(BigInt(body.speed));

  if (
    !body.orbitalCenter ||
    orbitalRadius === 0 ||
    speed === 0 ||
    elapsedSeconds <= 0
  ) {
    return body.position;
  }

  const direction = body.clockwise ? 1 : -1;
  const angle =
    ((direction * speed * elapsedSeconds) / orbitalRadius) %
    FULL_ROTATION_RADIANS;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);

  return {
    x: BigInt(Math.round(Number(x) * cos - Number(y) * sin)).toString(),
    y: BigInt(Math.round(Number(x) * sin + Number(y) * cos)).toString(),
    ...(body.position.relativeTo
      ? { relativeTo: body.position.relativeTo }
      : {}),
  };
}

export class OrbitalUpdaterService {
  private static simulationTimer: Timer | undefined;
  private static startPromise: Promise<void> | undefined;

  static async start() {
    OrbitalUpdaterService.startPromise ??=
      OrbitalUpdaterService.startSimulation();
    return OrbitalUpdaterService.startPromise;
  }

  private static async startSimulation() {
    await RepositoryService.start();
    await OrbitalUpdaterService.updatePositions(new Date());

    OrbitalUpdaterService.simulationTimer ??= setInterval(() => {
      void OrbitalUpdaterService.updatePositions(new Date()).catch(
        (error: unknown) => {
          console.error('Failed to update orbital positions', error);
        },
      );
    }, SIMULATION_INTERVAL_MS);
  }

  static stop() {
    if (OrbitalUpdaterService.simulationTimer) {
      clearInterval(OrbitalUpdaterService.simulationTimer);
      OrbitalUpdaterService.simulationTimer = undefined;
    }

    OrbitalUpdaterService.startPromise = undefined;
  }

  static async getWorldData() {
    await OrbitalUpdaterService.start();
    return RepositoryService.getWorldData();
  }

  static async getWorldSystemsBodies() {
    await OrbitalUpdaterService.start();
    return RepositoryService.getWorldSystemsBodies();
  }

  static async updateOrbitalBodies(time: string | Date) {
    await OrbitalUpdaterService.start();
    const invocationTime = new Date(time);
    if (Number.isNaN(invocationTime.getTime())) {
      throw new Error('Invocation time is invalid');
    }

    const selected =
      await OrbitalUpdaterService.updatePositions(invocationTime);

    return {
      selected,
      updated: selected,
    };
  }

  static async flushToDatabase() {
    return RepositoryService.flushToDatabase();
  }

  private static async updatePositions(invocationTime: Date) {
    let updated = 0;

    await RepositoryService.updateWorldBodies((worldData) => {
      for (const body of [
        ...worldData.stars,
        ...worldData.planets,
        ...worldData.moons,
      ]) {
        const elapsedSeconds =
          (invocationTime.getTime() - body.updatedAt.getTime()) / 1_000;
        body.position = advancePosition(body, elapsedSeconds);
        body.updatedAt = invocationTime;
        updated += 1;
      }

      return updated;
    });

    return updated;
  }
}
