import path from 'node:path';

export class WorldAssetService {
  static getAssetsDirectory() {
    return path.resolve(__dirname, '../../assets');
  }
}
