import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  preview: {
    allowedHosts: ['salimon.net'],
  },
  resolve: {
    alias: {
      '@components': fileURLToPath(
        new URL('./src/components/index.ts', import.meta.url),
      ),
      '@store': fileURLToPath(new URL('./src/store/index.ts', import.meta.url)),
      '@types': fileURLToPath(new URL('./src/types/index.ts', import.meta.url)),
      '@uitls': fileURLToPath(new URL('./src/utils/index.ts', import.meta.url)),
    },
  },
});
