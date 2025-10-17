import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default defineConfig(async () => {
  const baseConfig = await viteConfig({ command: 'serve', mode: 'test' });

  return mergeConfig(baseConfig, {
    test: {
      globals: true,
      environment: 'jsdom',
      css: true,
      include: ['src/**/*.{test,spec}.{ts,tsx}'],
      exclude: ['node_modules/**', 'e2e/**/*'],
    },
  });
});
