import fs from 'node:fs/promises';
import path from 'node:path';
import type { SerializedWorldSystems } from '@repo/types';

type WorldAsset = SerializedWorldSystems & {
  schemaVersion?: number;
  generatedAt?: string;
};

export class WorldAssetService {
  private static worldAssetPromise: Promise<WorldAsset> | undefined;

  static getAssetsDirectory() {
    return path.resolve(__dirname, '../../assets');
  }

  static async getWorldSystems(): Promise<SerializedWorldSystems> {
    const asset = await WorldAssetService.getWorldAsset();
    return {
      systems: asset.systems,
      ...(asset.asteroids ? { asteroids: asset.asteroids } : {}),
    };
  }

  private static async getWorldAsset() {
    WorldAssetService.worldAssetPromise ??= (async () => {
      const assetPath = path.join(
        WorldAssetService.getAssetsDirectory(),
        'world.json',
      );
      const content = await fs.readFile(assetPath, 'utf8');
      const asset = JSON.parse(content) as WorldAsset;

      if (!Array.isArray(asset.systems)) {
        throw new Error('World asset is missing a systems array.');
      }

      return asset;
    })().catch((error: unknown) => {
      WorldAssetService.worldAssetPromise = undefined;
      throw error;
    });

    return WorldAssetService.worldAssetPromise;
  }
}
