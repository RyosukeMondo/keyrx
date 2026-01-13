#!/usr/bin/env node
/**
 * Validate coverage thresholds for critical paths
 *
 * This script reads the coverage report and ensures that critical code paths
 * (hooks/, api/, services/) meet the higher 90% coverage threshold.
 *
 * Exit codes:
 * 0 - All thresholds met
 * 1 - One or more critical paths below 90% threshold
 */

import { readFileSync } from 'fs';
import { resolve } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Critical paths that require 90% coverage
const CRITICAL_PATHS = [
  'src/hooks/',
  'src/api/',
  'src/services/',
];

const CRITICAL_THRESHOLD = 90;

function main() {
  try {
    // Read the coverage summary
    const coveragePath = resolve(__dirname, '../coverage/coverage-summary.json');
    const coverageData = JSON.parse(readFileSync(coveragePath, 'utf-8'));

    console.log('\n📊 Coverage Validation for Critical Paths\n');
    console.log(`Threshold: ${CRITICAL_THRESHOLD}%\n`);

    let allPassed = true;
    const results = [];

    // Check each file in the coverage report
    for (const [filePath, metrics] of Object.entries(coverageData)) {
      // Skip the 'total' summary
      if (filePath === 'total') continue;

      // Check if this file is in a critical path
      const isCritical = CRITICAL_PATHS.some(path => filePath.includes(path));

      if (isCritical) {
        const lineCoverage = metrics.lines.pct;
        const branchCoverage = metrics.branches.pct;
        const functionCoverage = metrics.functions.pct;
        const statementCoverage = metrics.statements.pct;

        const minCoverage = Math.min(
          lineCoverage,
          branchCoverage,
          functionCoverage,
          statementCoverage
        );

        const passed = minCoverage >= CRITICAL_THRESHOLD;

        if (!passed) {
          allPassed = false;
        }

        results.push({
          file: filePath.replace(/^.*\/keyrx_ui\//, ''),
          line: lineCoverage.toFixed(2),
          branch: branchCoverage.toFixed(2),
          function: functionCoverage.toFixed(2),
          statement: statementCoverage.toFixed(2),
          min: minCoverage.toFixed(2),
          passed,
        });
      }
    }

    // Sort by minimum coverage (lowest first)
    results.sort((a, b) => parseFloat(a.min) - parseFloat(b.min));

    // Display results
    if (results.length === 0) {
      console.log('⚠️  No critical path files found in coverage report\n');
      console.log('This may indicate that:');
      console.log('  - Tests are not covering critical paths');
      console.log('  - Critical paths are excluded from coverage');
      console.log('  - Coverage report is incomplete\n');
      process.exit(1);
    }

    console.log('┌────────────────────────────────────────────────────────────────────────────┐');
    console.log('│ File                                   │ Line │ Branch │ Func │ Stmt │ Pass │');
    console.log('├────────────────────────────────────────────────────────────────────────────┤');

    for (const result of results) {
      const status = result.passed ? '✅' : '❌';
      const fileName = result.file.substring(0, 38).padEnd(38);
      console.log(
        `│ ${fileName} │ ${result.line.padStart(4)}% │ ${result.branch.padStart(6)}% │ ${result.function.padStart(4)}% │ ${result.statement.padStart(4)}% │ ${status}  │`
      );
    }

    console.log('└────────────────────────────────────────────────────────────────────────────┘\n');

    // Summary
    const passed = results.filter(r => r.passed).length;
    const total = results.length;
    console.log(`Summary: ${passed}/${total} critical files meet the ${CRITICAL_THRESHOLD}% threshold\n`);

    if (allPassed) {
      console.log('✅ All critical paths meet the coverage threshold!\n');
      process.exit(0);
    } else {
      console.log('❌ Some critical paths are below the coverage threshold\n');
      console.log('Please add tests to improve coverage for the failing files.\n');
      process.exit(1);
    }

  } catch (error) {
    if (error.code === 'ENOENT') {
      console.error('\n❌ Coverage report not found\n');
      console.error('Please run: npm run test:coverage\n');
    } else {
      console.error('\n❌ Error validating coverage:\n');
      console.error(error.message);
      console.error('\n');
    }
    process.exit(1);
  }
}

main();
