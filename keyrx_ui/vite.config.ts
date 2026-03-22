/// <reference types="vitest" />
import { defineConfig } from 'vitest/config';
import compression from 'vite-plugin-compression';
import { visualizer } from 'rollup-plugin-visualizer';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';
import fs from 'fs';
import { execSync } from 'child_process';

/**
 * Read version directly from Cargo.toml (SSOT) and inject build metadata.
 * Eliminates the generate-version.js intermediate step.
 */
function getBuildDefines() {
  const cargoToml = fs.readFileSync(
    path.resolve(__dirname, '../Cargo.toml'),
    'utf8',
  );
  const match = cargoToml.match(
    /\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/,
  );
  const version = match ? match[1] : '0.0.0';

  let gitCommit = 'unknown';
  let gitBranch = 'unknown';
  let gitDirty = false;
  try {
    gitCommit = execSync('git rev-parse --short HEAD', {
      encoding: 'utf8',
    }).trim();
    gitBranch = execSync('git rev-parse --abbrev-ref HEAD', {
      encoding: 'utf8',
    }).trim();
    const status = execSync('git status --porcelain', {
      encoding: 'utf8',
    });
    gitDirty = status.trim().length > 0;
  } catch {
    // git not available (CI container, etc.) — use defaults
  }

  return {
    __APP_VERSION__: JSON.stringify(version),
    __BUILD_TIME__: JSON.stringify(new Date().toISOString()),
    __GIT_COMMIT__: JSON.stringify(
      `${gitCommit}${gitDirty ? '-dirty' : ''}`,
    ),
    __GIT_BRANCH__: JSON.stringify(gitBranch),
    __GIT_DIRTY__: JSON.stringify(gitDirty),
  };
}

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
    // Force Vite to include env-shim so it's available for WASM
    include: [],
    esbuildOptions: {
      // Ensure 'env' resolves to our shim during pre-bundling
      alias: {
        'env': './src/wasm/env-shim.js',
      },
    },
  },
  define: getBuildDefines(),
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
