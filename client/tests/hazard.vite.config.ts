import { defineConfig } from 'vite'
// Separate output: normal dist has only index.html and cannot serve this entry.
export default defineConfig({ build: { outDir: '.hazard-fixture', rollupOptions: { input: 'tests/hazards.html' } } })
