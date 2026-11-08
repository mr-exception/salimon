// Run the same browser assertions with real CSS defects. Only the named
// assertion failure counts; startup, missing references and timeouts fail this driver.
import { spawnSync } from 'node:child_process'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'
const summaries = []
for (const [mutation, signature] of [['visual', 'UI_PIXEL_MISMATCH: gallery-planet-default-960.png'], ['focus', 'UI_FOCUS_REGRESSION']]) {
  const output = resolve(`test-results/ui-negative-${mutation}`)
  const result = spawnSync(process.execPath, ['node_modules/@playwright/test/cli.js', 'test', '--project=expedition', '--grep', 'UI-GALLERY.*960'], {
    env: { ...process.env, SALIMON_UI_CANDIDATES: '', SALIMON_UI_MUTATION: mutation, SALIMON_EVIDENCE_DIR: output },
    encoding: 'utf8', timeout: 120000, maxBuffer: 2e6,
  })
  await mkdir(output, { recursive: true })
  await writeFile(`${output}/command.log`, `${result.stdout}\n${result.stderr}`)
  const report = JSON.parse(await readFile(`${output}/results.json`, 'utf8'))
  const specs = suite => [...(suite.specs ?? []), ...(suite.suites ?? []).flatMap(specs)]
  const results = report.suites.flatMap(specs).flatMap(s => s.tests).flatMap(t => t.results)
  const errors = results.flatMap(r => r.errors ?? []).map(e => e.message).join('\n')
  const detected = !result.error && result.status === 1 && report.stats.unexpected === 1 && results.length === 1 && results[0].status === 'failed' && errors.includes(signature)
  const summary = { mutation, signature, exit: result.status, status: detected ? 'PASS' : 'INVALID_DEMONSTRATION' }
  summaries.push(summary)
  await writeFile(`${output}/mutation-summary.json`, JSON.stringify(summary, null, 2) + '\n')
  if (!detected) throw Error(`UI negative demonstration invalid: ${mutation}; inspect ${output}`)
}
console.log(JSON.stringify(summaries))
