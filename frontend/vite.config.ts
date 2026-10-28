import { defineConfig, loadEnv } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  return {
    plugins: [svelte()],
    server: {
      proxy: { '/api': { target: env.API_PROXY_TARGET || 'http://localhost:3000', ws: true } },
    },
    test: { include: ['src/**/*.test.ts'] },
  };
});
