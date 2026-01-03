/// <reference types="vitest" />
import { defineConfig } from 'vitest/config';
import compression from 'vite-plugin-compression';
import { visualizer } from 'rollup-plugin-visualizer';
import react from '@vitejs/plugin-react';
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
  server: {
    port: 5173,
    proxy: {
      // Proxy API requests to daemon
      '/api': {
        target: 'http://localhost:9867',
        changeOrigin: true,
        ws: false, // Don't proxy WebSocket here (use /ws prefix instead)
      },
      // Proxy WebSocket connections to daemon
      '/ws': {
        target: 'ws://localhost:9867',
        ws: true,
        changeOrigin: true,
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      // Fix wasm-pack 'env' import issue
      'env': path.resolve(__dirname, './src/wasm/env-shim.js'),
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
      external: ['env'],  // Fix for wasm-pack "env" import issue
      output: {
        manualChunks: (id) => {
          // All node_modules in vendor chunk to avoid circular dependencies
          if (id.includes('node_modules')) {
            return 'vendor';
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
