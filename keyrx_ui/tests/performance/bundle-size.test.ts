import { describe, it, expect, beforeAll } from 'vitest';
import fs from 'fs';
import path from 'path';
import { gzipSync } from 'zlib';

/**
 * Bundle Size Performance Tests
 *
 * Verifies that production bundle sizes meet performance requirements:
 * - Main bundle < 500KB (gzipped)
 * - Monaco chunk < 2MB (gzipped)
 * - WASM module < 1MB (uncompressed)
 */

const DIST_DIR = path.resolve(__dirname, '../../dist');

// Bundle size limits (in bytes)
const LIMITS = {
  main: 500 * 1024,      // 500KB
  monaco: 2 * 1024 * 1024, // 2MB
  wasm: 1 * 1024 * 1024    // 1MB
};

interface BundleInfo {
  file: string;
  size: number;
  gzippedSize?: number;
}

/**
 * Get size of a file in bytes
 */
function getFileSize(filePath: string): number {
  const stats = fs.statSync(filePath);
  return stats.size;
}

/**
 * Get gzipped size of a file
 */
function getGzippedSize(filePath: string): number {
  const content = fs.readFileSync(filePath);
  const compressed = gzipSync(content);
  return compressed.length;
}

/**
 * Find bundle files matching pattern
 */
function findBundles(pattern: RegExp): BundleInfo[] {
  const assetsDir = path.join(DIST_DIR, 'assets');

  if (!fs.existsSync(assetsDir)) {
    return [];
  }

  const files = fs.readdirSync(assetsDir);
  return files
    .filter(file => pattern.test(file))
    .map(file => {
      const filePath = path.join(assetsDir, file);
      const size = getFileSize(filePath);
      const gzippedSize = getGzippedSize(filePath);
      return { file, size, gzippedSize };
    });
}

/**
 * Format bytes as human-readable string
 */
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)}MB`;
}

describe('Bundle Size Performance Tests', () => {
  beforeAll(() => {
    if (!fs.existsSync(DIST_DIR)) {
      throw new Error(
        `Build output not found at ${DIST_DIR}. Run 'npm run build' first.`
      );
    }
  });

  it('should have dist/index.html', () => {
    const indexPath = path.join(DIST_DIR, 'index.html');
    expect(fs.existsSync(indexPath)).toBe(true);
  });

  it('main bundle should be < 500KB gzipped', () => {
    // Find main bundle (typically index-[hash].js)
    const bundles = findBundles(/^index-[a-f0-9]+\.js$/);

    expect(bundles.length).toBeGreaterThan(0);

    const mainBundle = bundles[0];
    console.log(`Main bundle: ${mainBundle.file}`);
    console.log(`  Raw size: ${formatBytes(mainBundle.size)}`);
    console.log(`  Gzipped: ${formatBytes(mainBundle.gzippedSize!)}`);

    expect(mainBundle.gzippedSize).toBeLessThanOrEqual(LIMITS.main);
  });

  it('Monaco chunk should be < 2MB gzipped', () => {
    // Find Monaco chunk (typically includes 'monaco' or 'editor' in name)
    const bundles = findBundles(/monaco|editor/i);

    if (bundles.length === 0) {
      console.warn('Warning: Monaco chunk not found (may be bundled with vendor)');
      return;
    }

    const monacoBundle = bundles[0];
    console.log(`Monaco bundle: ${monacoBundle.file}`);
    console.log(`  Raw size: ${formatBytes(monacoBundle.size)}`);
    console.log(`  Gzipped: ${formatBytes(monacoBundle.gzippedSize!)}`);

    expect(monacoBundle.gzippedSize).toBeLessThanOrEqual(LIMITS.monaco);
  });

  it('WASM module should be < 1MB uncompressed', () => {
    // Find WASM file
    const assetsDir = path.join(DIST_DIR, 'assets');
    const files = fs.readdirSync(assetsDir);
    const wasmFiles = files.filter(file => file.endsWith('.wasm'));

    if (wasmFiles.length === 0) {
      console.warn('Warning: WASM module not found in build output');
      return;
    }

    const wasmFile = wasmFiles[0];
    const wasmPath = path.join(assetsDir, wasmFile);
    const wasmSize = getFileSize(wasmPath);

    console.log(`WASM module: ${wasmFile}`);
    console.log(`  Size: ${formatBytes(wasmSize)}`);

    expect(wasmSize).toBeLessThanOrEqual(LIMITS.wasm);
  });

  it('should print bundle summary', () => {
    const assetsDir = path.join(DIST_DIR, 'assets');
    const files = fs.readdirSync(assetsDir);

    let totalSize = 0;
    let totalGzipped = 0;

    console.log('\n=== Bundle Summary ===');

    files.forEach(file => {
      const filePath = path.join(assetsDir, file);
      const size = getFileSize(filePath);
      totalSize += size;

      if (file.endsWith('.js') || file.endsWith('.css')) {
        const gzipped = getGzippedSize(filePath);
        totalGzipped += gzipped;
        console.log(`${file}: ${formatBytes(size)} → ${formatBytes(gzipped)} gzipped`);
      } else {
        console.log(`${file}: ${formatBytes(size)}`);
      }
    });

    console.log(`\nTotal: ${formatBytes(totalSize)}`);
    console.log(`Total (JS+CSS gzipped): ${formatBytes(totalGzipped)}`);
    console.log('=====================\n');

    // This test always passes, it's just for reporting
    expect(true).toBe(true);
  });
});
