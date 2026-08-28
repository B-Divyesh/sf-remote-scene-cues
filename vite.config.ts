import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

export default defineConfig({
  root: 'frontend',
  plugins: [svelte()],
  build: {
    outDir: resolve(__dirname, 'dist'),
    emptyOutDir: true,
    target: 'es2022',
    sourcemap: true
  },
  server: {
    proxy: { '/api': 'http://localhost:8080', '/health': 'http://localhost:8080' }
  }
});
