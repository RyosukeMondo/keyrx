import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    css: {
      modules: {
        classNameStrategy: 'non-scoped',
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      // Mock WASM module imports for testing
      './pkg/keyrx_core': path.resolve(__dirname, './src/wasm/__mocks__/pkg.ts'),
      // Mock Monaco editor for testing
      'monaco-editor': path.resolve(__dirname, './src/test/__mocks__/monaco.ts'),
    },
  },
});
