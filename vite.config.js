import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue()],
  // Tauri expects a fixed port and shows its own errors.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  test: {
    environment: 'jsdom',
    include: ['src/tests/**/*.test.js'],
    setupFiles: ['src/tests/setup.js'],
    // One jsdom per worker instead of one per file, each file still isolated: creating it 15 times
    // took a third of the run on the small CI runners and made the first test of a file time out.
    pool: 'vmThreads',
    // The first mount of the app in a file still takes up to 16 s on the loaded Windows runners, against
    // half a second locally: CI gets more time, while a test stuck locally still fails fast.
    testTimeout: process.env.CI ? 30000 : 5000,
  },
});
