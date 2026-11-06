// Perturb exactly one expected image in an isolated copy. Rehash it so this
// proves a pixel comparison failure, rather than an integrity/startup failure.
import { cp, mkdtemp, mkdir, readFile, writeFile, symlink, rm } from 'node:fs/promises'
import { spawnSync } from 'node:child_process'
import { tmpdir } from 'node:os'
import { join, resolve, basename } from 'node:path'
import { createHash } from 'node:crypto'
import { PNG } from 'pngjs'
const root = process.cwd(), output = resolve(process.env.SALIMON_PANORAMIC_NEGATIVE_DIR ?? 'test-results/panoramic-negative')
const scratch = await mkdtemp(join(tmpdir(), 'salimon-panoramic-negative-'))
const hash = v => createHash('sha256').update(v).digest('hex')
try {
 await cp(root, scratch, { recursive: true, filter: p => !['node_modules', 'test-results', 'playwright-report', '.git'].includes(basename(p)) })
 await symlink(join(root, 'node_modules'), join(scratch, 'node_modules'), 'dir')
 const source = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim()
 await writeFile(join(scratch, '.revision-for-negative'), source)
 const name = 'panoramic-telescope.png', path = join(scratch, 'tests/navigation-baselines', name)
 const original = await readFile(path), png = PNG.sync.read(original)
 for (let y = 350; y < 450; y++) for (let x = 590; x < 690; x++) {
  const i = (y * png.width + x) * 4; png.data.set([255, 0, 255, 255], i)
 }
 const changed = PNG.sync.write(png); await writeFile(path, changed)
 const manifestPath = join(scratch, 'tests/navigation-baselines/manifest.json')
 const manifest = JSON.parse(await readFile(manifestPath, 'utf8')); manifest.images[name] = hash(changed)
 await writeFile(manifestPath, JSON.stringify(manifest))
 const child = spawnSync(process.execPath, [join(root, 'node_modules/@playwright/test/cli.js'), 'test', '--project=navigation-captures', '--grep', 'PANO-VIS'], {
  cwd: scratch, env: { ...process.env, CI: 'true', SALIMON_PANORAMIC_CANDIDATES: '', SALIMON_EVIDENCE_DIR: output }, encoding: 'utf8', timeout: 480000, maxBuffer: 2e6,
 })
 await mkdir(output, { recursive: true }); await writeFile(join(output, 'command.log'), `${child.stdout ?? ''}\n${child.stderr ?? ''}`)
 const report = JSON.parse(await readFile(join(output, 'results.json'), 'utf8'))
 const specs = suite => [...(suite.specs ?? []), ...(suite.suites ?? []).flatMap(specs)]
 const results = report.suites.flatMap(specs).flatMap(s => s.tests ?? []).flatMap(t => t.results ?? [])
 const attachment = results[0]?.attachments?.find(a => a.name === 'panoramic-summary.json')
 const summary = attachment ? JSON.parse(attachment.body ? Buffer.from(attachment.body, 'base64').toString() : await readFile(attachment.path, 'utf8')) : null
 const failed = summary?.results.filter(r => !r.pass)
 const valid = child.status === 1 && !child.error && report.stats.unexpected === 1 && results.length === 1 &&
  failed?.length === 1 && failed[0].id === 'telescope' && failed[0].changedPixels >= 10000 &&
  results[0].errors.some(e => e.message?.includes('PANORAMIC_PIXEL_MISMATCH telescope'))
 await writeFile(join(output, 'mutation-summary.json'), JSON.stringify({ status: valid ? 'PASS' : 'INVALID_DEMONSTRATION', source, name, original: hash(original), mutated: hash(changed), exit: child.status, summary }, null, 2))
 if (!valid) throw Error(`Expected telescope screenshot failure was not isolated: ${output}`)
 console.log('PASS: mutated expected telescope fails its pixel assertion; other panoramic references pass; originals, diff and trace retained')
} finally { await rm(scratch, { recursive: true, force: true }) }
