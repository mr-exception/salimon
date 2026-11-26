import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { PNG } from 'pngjs'
import { scoutEntities, SCOUT_ANCHORS } from '../src/scoutLayout.ts'
import { validateAssetRegistry } from '../src/rendering/assetRegistry.ts'

export const authoringSources = new URL('../../art/sources/', import.meta.url)
const registryFile = new URL('../assets/registry.json', import.meta.url)

export function alphaBounds(pixels: PNG): readonly [number, number, number, number] | undefined {
  let left = pixels.width, top = pixels.height, right = -1, bottom = -1
  for (let y = 0; y < pixels.height; y++) {
    for (let x = 0; x < pixels.width; x++) {
      if (!pixels.data[(y * pixels.width + x) * 4 + 3]) continue
      left = Math.min(left, x)
      top = Math.min(top, y)
      right = Math.max(right, x)
      bottom = Math.max(bottom, y)
    }
  }
  return right < 0 ? undefined : [left, top, right - left + 1, bottom - top + 1]
}

export function readAssetRegistry(sourceLimit = 4096) {
  const props = new Map(scoutEntities().map(entity => [entity.id, entity.footprint]))
  return validateAssetRegistry(JSON.parse(readFileSync(registryFile, 'utf8')), {
    source(path) {
      let bytes: Buffer
      try {
        bytes = readFileSync(new URL(path, authoringSources))
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === 'ENOENT') return undefined
        throw error
      }
      if (!path.endsWith('.png')) return {}
      // Check IHDR before decoding so an oversize source cannot allocate an
      // unbounded pixel surface while the registry validator inspects alpha.
      if (bytes.length < 33 || bytes.toString('hex', 0, 8) !== '89504e470d0a1a0a' ||
        bytes.toString('ascii', 12, 16) !== 'IHDR') throw new Error(`Invalid PNG source ${path}`)
      const width = bytes.readUInt32BE(16), height = bytes.readUInt32BE(20)
      if (!width || !height || width > sourceLimit || height > sourceLimit) {
        throw new Error(`PNG source ${path} exceeds source dimension limit or has an invalid size`)
      }
      return { alphaBounds: alphaBounds(PNG.sync.read(bytes)) }
    },
    footprint(ref) {
      const footprint = props.get(ref)
      return footprint && [2 * footprint.halfWidth, 2 * footprint.halfHeight]
    },
    anchor: ref => Object.hasOwn(SCOUT_ANCHORS, ref),
  })
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const registry = readAssetRegistry()
  console.log(`Validated asset registry v${registry.schemaVersion}: ${registry.assets.length} assets`)
}
