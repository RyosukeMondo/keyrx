/// <reference types="vitest" />
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import compression from 'vite-plugin-compression';
import { visualizer } from 'rollup-plugin-visualizer';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';

export default defineConfig({
  plugins: [
    wasm(),
    topLevelAwait(),
    react(),
    compression({
      algorithm: 'gzip',
      ext: '.gz',
    }),
    compression({
      algorithm: 'brotliCompress',
      ext: '.br',
    }),
    visualizer({
      filename: './dist/stats.html',
      open: false,
      gzipSize: true,
      brotliSize: true,
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  optimizeDeps: {
    exclude: ['@/wasm/pkg/keyrx_core'],
  },
  build: {
    target: 'esnext',
    minify: 'terser',
    sourcemap: true, // Generate source maps for debugging production issues
    terserOptions: {
      compress: {
        drop_console: true, // Remove console.log in production
        drop_debugger: true, // Remove debugger statements
        passes: 2, // Multiple passes for better compression
      },
      mangle: {
        safari10: true, // Safari 10+ compatibility
      },
      format: {
        comments: false, // Remove all comments
      },
    },
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Vendor chunk: React core libraries
          if (
            id.includes('node_modules/react') ||
            id.includes('node_modules/react-dom') ||
            id.includes('node_modules/react-router-dom')
          ) {
            return 'vendor';
          }

          // Monaco chunk: Monaco editor
          if (id.includes('node_modules/@monaco-editor/react') || id.includes('node_modules/monaco-editor')) {
            return 'monaco';
          }

          // Charts chunk: Recharts
          if (id.includes('node_modules/recharts')) {
            return 'charts';
          }

          // State management
          if (id.includes('node_modules/zustand')) {
            return 'zustand';
          }

          // UI libraries
          if (id.includes('node_modules/@headlessui') || id.includes('node_modules/@floating-ui')) {
            return 'ui-libs';
          }

          // Animation libraries
          if (id.includes('node_modules/framer-motion')) {
            return 'framer-motion';
          }

          // Query library
          if (id.includes('node_modules/@tanstack/react-query')) {
            return 'react-query';
          }

          // Icons
          if (id.includes('node_modules/lucide-react')) {
            return 'lucide-icons';
          }

          // Virtual scrolling
          if (id.includes('node_modules/react-window')) {
            return 'react-window';
          }

          // Other node_modules as other
          if (id.includes('node_modules')) {
            return 'other';
          }
        },
      },
    },
    chunkSizeWarningLimit: 500,
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    css: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      exclude: [
        'node_modules/**',
        'dist/**',
        'src/test/**',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        'src/wasm/pkg/**',
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 80,
        statements: 80,
      },
    },
  },
});
