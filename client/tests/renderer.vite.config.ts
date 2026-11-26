import { defineConfig } from 'vite'
export default defineConfig({
  build: {
    outDir: '.renderer-fixture',
    rollupOptions: { input: 'tests/renderer/index.html' },
  },
})
