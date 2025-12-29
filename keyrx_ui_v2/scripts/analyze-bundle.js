#!/usr/bin/env node
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, extname } from 'path';
import { gzipSync, brotliCompressSync } from 'zlib';

const DIST_DIR = './dist';
const MAX_JS_SIZE_KB = 250; // 250KB gzipped target
const MAX_CSS_SIZE_KB = 50; // 50KB gzipped target

function getDirectorySize(dir, extension) {
  const files = readdirSync(dir, { recursive: true });
  let totalSize = 0;
  let totalGzipped = 0;
  let totalBrotli = 0;
  const fileList = [];

  for (const file of files) {
    const filePath = join(dir, file);
    if (statSync(filePath).isFile() && extname(filePath) === extension) {
      const content = readFileSync(filePath);
      const size = content.length;
      const gzipped = gzipSync(content).length;
      const brotli = brotliCompressSync(content).length;

      totalSize += size;
      totalGzipped += gzipped;
      totalBrotli += brotli;

      fileList.push({
        name: file,
        size: (size / 1024).toFixed(2),
        gzipped: (gzipped / 1024).toFixed(2),
        brotli: (brotli / 1024).toFixed(2),
      });
    }
  }

  return {
    totalSize: (totalSize / 1024).toFixed(2),
    totalGzipped: (totalGzipped / 1024).toFixed(2),
    totalBrotli: (totalBrotli / 1024).toFixed(2),
    files: fileList,
  };
}

console.log('\n📊 Bundle Size Analysis\n');
console.log('=' .repeat(80));

// Analyze JavaScript
console.log('\n📦 JavaScript Files:');
const jsStats = getDirectorySize(join(DIST_DIR, 'assets'), '.js');
console.log(`\n  Total: ${jsStats.totalSize} KB (raw)`);
console.log(`  Gzipped: ${jsStats.totalGzipped} KB`);
console.log(`  Brotli: ${jsStats.totalBrotli} KB`);
console.log(`  Target: ≤${MAX_JS_SIZE_KB} KB (gzipped)\n`);

if (jsStats.files.length > 0) {
  console.log('  Individual files:');
  jsStats.files
    .sort((a, b) => parseFloat(b.gzipped) - parseFloat(a.gzipped))
    .forEach((file) => {
      console.log(
        `    ${file.name.padEnd(40)} ${file.gzipped.padStart(8)} KB (gzipped)`
      );
    });
}

// Analyze CSS
console.log('\n🎨 CSS Files:');
const cssStats = getDirectorySize(join(DIST_DIR, 'assets'), '.css');
console.log(`\n  Total: ${cssStats.totalSize} KB (raw)`);
console.log(`  Gzipped: ${cssStats.totalGzipped} KB`);
console.log(`  Brotli: ${cssStats.totalBrotli} KB`);
console.log(`  Target: ≤${MAX_CSS_SIZE_KB} KB (gzipped)\n`);

if (cssStats.files.length > 0) {
  console.log('  Individual files:');
  cssStats.files
    .sort((a, b) => parseFloat(b.gzipped) - parseFloat(a.gzipped))
    .forEach((file) => {
      console.log(
        `    ${file.name.padEnd(40)} ${file.gzipped.padStart(8)} KB (gzipped)`
      );
    });
}

// Check budgets
console.log('\n✅ Budget Check:');
const jsPass = parseFloat(jsStats.totalGzipped) <= MAX_JS_SIZE_KB;
const cssPass = parseFloat(cssStats.totalGzipped) <= MAX_CSS_SIZE_KB;

console.log(
  `  JavaScript: ${jsPass ? '✅ PASS' : '❌ FAIL'} (${jsStats.totalGzipped} KB / ${MAX_JS_SIZE_KB} KB)`
);
console.log(
  `  CSS: ${cssPass ? '✅ PASS' : '❌ FAIL'} (${cssStats.totalGzipped} KB / ${MAX_CSS_SIZE_KB} KB)`
);

console.log('\n' + '='.repeat(80));

if (!jsPass || !cssPass) {
  console.log('\n❌ Bundle size exceeds budget!');
  console.log('\nRecommendations:');
  if (!jsPass) {
    console.log('  - Review large dependencies with bundle visualizer (dist/stats.html)');
    console.log('  - Ensure code splitting is working for routes');
    console.log('  - Check for duplicate dependencies');
    console.log('  - Consider lazy loading heavy libraries');
  }
  if (!cssPass) {
    console.log('  - Review unused Tailwind classes with PurgeCSS');
    console.log('  - Check for duplicate CSS imports');
    console.log('  - Consider splitting critical CSS');
  }
  process.exit(1);
} else {
  console.log('\n✅ All bundles within budget!');
  console.log('\n📈 View detailed analysis: dist/stats.html');
  process.exit(0);
}
