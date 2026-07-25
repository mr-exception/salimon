import fs from 'node:fs/promises';
import path from 'node:path';
import type { SerializedWorldBody, SerializedWorldSystems } from '@repo/types';

type WorldAsset = SerializedWorldSystems & {
  schemaVersion?: number;
  generatedAt?: string;
};

const MIN_RENDER_SHAPE_SCREEN_WIDTH = 16;

const GALACTIC_CENTER: SerializedWorldBody = {
  id: '9661cfb2-d951-581a-a129-f7b64d392e7a',
  type: 'blackhole',
  name: 'Sagittarius A*',
  isReal: true,
  position: { x: '0', y: '0' },
  orbitalCenter: null,
  clockwise: false,
  speed: '0',
  mass: '8544584500000000000000000000000000000',
  radius: '0',
  minZoomRenderShape: 0,
  rotationPeriodSeconds: 0,
  cTime: 1784419200000,
} as SerializedWorldBody;
const SOLAR_SYSTEM = [
  {
    id: 'fa9c74d5-f63f-5bbc-aba5-0eaafa960920',
    type: 'star',
    name: 'Sun',
    isReal: true,
    position: {
      x: '245978992287100800000',
      y: '0',
      relativeToId: GALACTIC_CENTER.id,
    },
    orbitalCenter: GALACTIC_CENTER.name,
    clockwise: false,
    speed: '220000',
    mass: '1988500000000000000000000000000',
    radius: '695700000',
    minZoomRenderShape: getMinZoomRenderShape('695700000'),
    rotationPeriodSeconds: 2192832,
    cTime: 1784419200000,
  },
  {
    id: '0d2a0b86-2208-510f-a638-10b6c4556a55',
    type: 'planet',
    name: 'Earth',
    isReal: true,
    position: {
      x: '149598023000',
      y: '0',
      relativeToId: 'fa9c74d5-f63f-5bbc-aba5-0eaafa960920',
    },
    orbitalCenter: 'Sun',
    clockwise: false,
    speed: '29780',
    mass: '5972370000000000000000000',
    radius: '6371000',
    minZoomRenderShape: getMinZoomRenderShape('6371000'),
    rotationPeriodSeconds: 86164.1,
    cTime: 1784419200000,
  },
  {
    id: '79ebb54e-b054-5917-bcbe-7d8b55e2af3e',
    type: 'moon',
    name: 'Moon',
    isReal: true,
    position: {
      x: '333189479',
      y: '192199500',
      relativeToId: '0d2a0b86-2208-510f-a638-10b6c4556a55',
    },
    orbitalCenter: 'Earth',
    clockwise: false,
    speed: '1022',
    mass: '73420000000000000000000',
    radius: '1737400',
    minZoomRenderShape: getMinZoomRenderShape('1737400'),
    rotationPeriodSeconds: 2360591.51,
    cTime: 1784419200000,
  },
] as SerializedWorldBody[];
const SOLAR_SYSTEM_BODY_NAMES = new Set(SOLAR_SYSTEM.map((body) => body.name));

export class WorldAssetService {
  private static worldAsset: WorldAsset | undefined;
  private static worldAssetPromise: Promise<WorldAsset> | undefined;

  static getAssetsDirectory() {
    return path.resolve(__dirname, '../../assets');
  }

  static async getWorldSystems(): Promise<SerializedWorldSystems> {
    const asset = await WorldAssetService.getWorldAsset();
    return {
      systems: asset.systems,
    };
  }

  static async start() {
    await WorldAssetService.getWorldAsset();
  }

  private static async getWorldAsset() {
    if (WorldAssetService.worldAsset) {
      return WorldAssetService.worldAsset;
    }

    WorldAssetService.worldAssetPromise ??= (async () => {
      const worldDirectory = path.join(
        WorldAssetService.getAssetsDirectory(),
        'world',
      );
      const assetFileNames = (await fs.readdir(worldDirectory))
        .filter((fileName) => /^data-\d{4}\.json$/.test(fileName))
        .sort();

      if (assetFileNames.length === 0) {
        throw new Error('World asset directory is missing data chunk files.');
      }

      const chunks = await Promise.all(
        assetFileNames.map(async (fileName) => {
          const content = await fs.readFile(
            path.join(worldDirectory, fileName),
            'utf8',
          );
          const chunk = JSON.parse(content) as WorldAsset;

          if (!Array.isArray(chunk.systems)) {
            throw new Error(
              `World asset chunk ${fileName} is missing systems.`,
            );
          }

          return chunk;
        }),
      );
      const [firstChunk] = chunks;
      let systems = normalizeWorldSystems(
        chunks.flatMap((chunk) => chunk.systems),
      );
      const galacticCenter =
        systems.flat().find((body) => body.name === GALACTIC_CENTER.name) ??
        GALACTIC_CENTER;

      systems = systems
        .map((system) =>
          system.filter((body) => body.name !== GALACTIC_CENTER.name),
        )
        .filter((system) => system.length > 0);
      const bodyNames = new Set(systems.flat().map((body) => body.name));

      if (!bodyNames.has('Sun') || !bodyNames.has('Earth')) {
        systems = [
          SOLAR_SYSTEM,
          ...systems.filter(
            (system) =>
              !system.some((body) => SOLAR_SYSTEM_BODY_NAMES.has(body.name)),
          ),
        ];
      }

      systems = [[galacticCenter], ...systems];

      const asset: WorldAsset = {
        schemaVersion: firstChunk.schemaVersion,
        generatedAt: firstChunk.generatedAt,
        systems,
      };

      WorldAssetService.worldAsset = asset;
      return asset;
    })().catch((error: unknown) => {
      WorldAssetService.worldAssetPromise = undefined;
      throw error;
    });

    return WorldAssetService.worldAssetPromise;
  }
}

function normalizeWorldSystems(systems: SerializedWorldBody[][]) {
  const bodyById = new Map<string, SerializedWorldBody>();

  systems.flat().forEach((body) => {
    if (body.id) bodyById.set(body.id, body);
  });

  return systems.map((system) =>
    system.map((body) => {
      const relativeToId = body.position.relativeToId;
      const reference = relativeToId ? bodyById.get(relativeToId) : undefined;

      return {
        ...body,
        minZoomRenderShape:
          body.minZoomRenderShape ?? getMinZoomRenderShape(body.radius),
        position: {
          ...body.position,
          ...(reference ? { relativeTo: reference.name } : {}),
        },
      };
    }),
  );
}

function getMinZoomRenderShape(radius: string) {
  const radiusNumber = Number(radius);
  if (!Number.isFinite(radiusNumber) || radiusNumber <= 0) return 0;

  return MIN_RENDER_SHAPE_SCREEN_WIDTH / 2 / radiusNumber;
}
