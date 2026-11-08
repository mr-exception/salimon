import { readFile, writeFile } from 'node:fs/promises'
import { compareImages, sha256 } from './visual-comparison.ts'

export interface UIManifest {
  schema: 1
  review: 'PENDING_OWNER_REVIEW'
  environment: unknown
  images: Record<string, string>
}
/** Missing, corrupt or unregistered references never become automatic passes. */
export async function compareUIReference(directory: string, manifest: UIManifest, name: string, actual: Buffer, output: string) {
  if (!Object.hasOwn(manifest.images, name)) throw Error(`UI_BASELINE_MISSING: ${name}`)
  const expected = await readFile(`${directory}/${name}`)
  if (sha256(expected) !== manifest.images[name]) throw Error(`UI_BASELINE_HASH: ${name}`)
  const { diff, ...measurement } = compareImages(expected, actual)
  await writeFile(`${output}/${name.replace('.png', '-expected.png')}`, expected)
  await writeFile(`${output}/${name.replace('.png', '-diff.png')}`, diff)
  await writeFile(`${output}/${name.replace('.png', '-comparison.json')}`, JSON.stringify({ ...measurement, expected: sha256(expected), actual: sha256(actual) }, null, 2) + '\n')
  if (!measurement.pass) throw Error(`UI_PIXEL_MISMATCH: ${name} (${measurement.changedPixels} pixels)`)
  return measurement
}
