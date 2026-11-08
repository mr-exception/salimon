import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtemp, writeFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { PNG } from 'pngjs'
import { compareUIReference, type UIManifest } from './ui-reference.ts'
import { sha256 } from './visual-comparison.ts'

test('UI references fail closed for absent registration, missing file, hash drift and real pixel mutation', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'salimon-ui-reference-'))
  try {
    const png = new PNG({ width: 32, height: 32 }); png.data.fill(255)
    const image = PNG.sync.write(png)
    const manifest: UIManifest = { schema: 1, review: 'PENDING_OWNER_REVIEW', environment: {}, images: {} }
    await assert.rejects(compareUIReference(dir, manifest, 'state.png', image, dir), /UI_BASELINE_MISSING/)
    manifest.images['state.png'] = sha256(image)
    await assert.rejects(compareUIReference(dir, manifest, 'state.png', image, dir), /ENOENT/)
    await writeFile(join(dir, 'state.png'), image)
    assert.equal((await compareUIReference(dir, manifest, 'state.png', image, dir)).pass, true)
    manifest.images['state.png'] = 'wrong'
    await assert.rejects(compareUIReference(dir, manifest, 'state.png', image, dir), /UI_BASELINE_HASH/)
    manifest.images['state.png'] = sha256(image)
    png.data.fill(0); const defect = PNG.sync.write(png)
    await assert.rejects(compareUIReference(dir, manifest, 'state.png', defect, dir), /UI_PIXEL_MISMATCH/)
  } finally { await rm(dir, { recursive: true, force: true }) }
})
