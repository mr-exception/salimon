import { defineConfig } from '@playwright/test'
import { VIEWPORT } from './tests/renderer/model.ts'

const evidenceDir = process.env.SALIMON_EVIDENCE_DIR ?? 'test-results/renderer'

export default defineConfig({
  testDir: './tests',
  testMatch: /renderer-benchmark\.spec\.ts/,
  outputDir: evidenceDir,
  timeout: 240_000,
  expect: { timeout: 10_000 },
  workers: 1,
  reporter: [['list'], ['json', { outputFile: `${evidenceDir}/results.json` }]],
  use: {
    viewport: VIEWPORT,
    deviceScaleFactor: 1,
    locale: 'en-US',
    timezoneId: 'UTC',
    colorScheme: 'dark',
    reducedMotion: 'reduce',
    screenshot: 'on',
    trace: { mode: 'on', snapshots: false, screenshots: false, sources: true },
    launchOptions: {
      executablePath: process.env.CHROMIUM_PATH,
      args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--disable-lcd-text'],
    },
  },
  webServer: {
    command: 'vite build --config tests/renderer.vite.config.ts && vite preview --outDir .renderer-fixture --host 127.0.0.1 --port 4375 --strictPort',
    url: 'http://127.0.0.1:4375/tests/renderer/index.html',
  },
})
