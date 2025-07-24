import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    globals: true,
    includeSource: ['src/**/*.{ts,js}'],
  },
  define: {
    'import.meta.vitest': undefined,
  },
});