#!/usr/bin/env npx tsx
/**
 * Automated E2E Test Runner
 *
 * This script orchestrates automated API testing against the KeyRx daemon.
 * It manages daemon lifecycle, executes test suites, compares results with
 * expected values, and optionally applies auto-fixes for common failures.
 *
 * Usage:
 *   npx tsx scripts/automated-e2e-test.ts [options]
 *
 * Options:
 *   --daemon-path PATH    Path to daemon executable (default: target/release/keyrx_daemon)
 *   --port PORT          Port for daemon API (default: 9867)
 *   --max-iterations N   Maximum fix/retry iterations (default: 3)
 *   --fix                Enable auto-fix engine
 *   --report-json PATH   Output JSON report to file
 *   --help, -h           Show help message
 *
 * Exit codes:
 *   0 - All tests passed
 *   1 - Test failures remain after auto-fix
 *   2 - Daemon startup failed
 *   3 - Configuration error
 */

import { spawn, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as process from 'process';

interface CliOptions {
  daemonPath: string;
  port: number;
  maxIterations: number;
  enableFix: boolean;
  reportJsonPath?: string;
}

interface TestSuiteResult {
  total: number;
  passed: number;
  failed: number;
  duration: number;
  results: TestResult[];
}

interface TestResult {
  id: string;
  name: string;
  status: 'pass' | 'fail';
  duration: number;
  error?: string;
  actual?: unknown;
  expected?: unknown;
}

/**
 * Parse command line arguments
 */
function parseArgs(): CliOptions {
  const args = process.argv.slice(2);
  const options: CliOptions = {
    daemonPath: path.join('target', 'release', process.platform === 'win32' ? 'keyrx_daemon.exe' : 'keyrx_daemon'),
    port: 9867,
    maxIterations: 3,
    enableFix: false,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--daemon-path':
        if (!args[i + 1]) {
          throw new Error('--daemon-path requires a value');
        }
        options.daemonPath = args[i + 1];
        i++;
        break;
      case '--port':
        if (!args[i + 1]) {
          throw new Error('--port requires a value');
        }
        options.port = parseInt(args[i + 1], 10);
        if (isNaN(options.port)) {
          throw new Error('--port must be a number');
        }
        i++;
        break;
      case '--max-iterations':
        if (!args[i + 1]) {
          throw new Error('--max-iterations requires a value');
        }
        options.maxIterations = parseInt(args[i + 1], 10);
        if (isNaN(options.maxIterations)) {
          throw new Error('--max-iterations must be a number');
        }
        i++;
        break;
      case '--fix':
        options.enableFix = true;
        break;
      case '--report-json':
        if (!args[i + 1]) {
          throw new Error('--report-json requires a value');
        }
        options.reportJsonPath = args[i + 1];
        i++;
        break;
      case '--help':
      case '-h':
        showHelp();
        process.exit(0);
        break;
      default:
        console.error(`Unknown option: ${args[i]}`);
        showHelp();
        process.exit(3);
    }
  }

  return options;
}

/**
 * Show help message
 */
function showHelp(): void {
  console.log(`
Automated E2E Test Runner

Orchestrates automated API testing against the KeyRx daemon with daemon lifecycle
management, test execution, result comparison, and optional auto-fix.

Usage:
  npx tsx scripts/automated-e2e-test.ts [options]

Options:
  --daemon-path PATH    Path to daemon executable
                        Default: target/release/keyrx_daemon (or .exe on Windows)
  --port PORT          Port for daemon API (default: 9867)
  --max-iterations N   Maximum fix/retry iterations (default: 3)
  --fix                Enable auto-fix engine for common failures
  --report-json PATH   Output JSON report to specified file
  --help, -h           Show this help message

Examples:
  # Run tests with default settings
  npx tsx scripts/automated-e2e-test.ts

  # Run with auto-fix enabled and save JSON report
  npx tsx scripts/automated-e2e-test.ts --fix --report-json results.json

  # Use custom daemon path and port
  npx tsx scripts/automated-e2e-test.ts --daemon-path ./build/daemon --port 8080

Exit Codes:
  0 - All tests passed
  1 - Test failures remain after auto-fix
  2 - Daemon startup failed
  3 - Configuration error
`);
}

/**
 * Check if daemon executable exists
 */
function checkDaemonExists(daemonPath: string): void {
  if (!fs.existsSync(daemonPath)) {
    console.error(`❌ Daemon executable not found: ${daemonPath}`);
    console.error('\nPlease build the daemon first:');
    console.error('  cargo build --release -p keyrx_daemon');
    process.exit(3);
  }
}

/**
 * Main test orchestration function
 */
async function main(): Promise<void> {
  let options: CliOptions;

  try {
    options = parseArgs();
  } catch (error) {
    console.error(`❌ Configuration error: ${error instanceof Error ? error.message : String(error)}`);
    process.exit(3);
  }

  console.log('🧪 Automated E2E Test Runner\n');
  console.log(`Daemon: ${options.daemonPath}`);
  console.log(`Port: ${options.port}`);
  console.log(`Max iterations: ${options.maxIterations}`);
  console.log(`Auto-fix: ${options.enableFix ? 'enabled' : 'disabled'}`);
  if (options.reportJsonPath) {
    console.log(`JSON report: ${options.reportJsonPath}`);
  }
  console.log('');

  // Verify daemon exists
  checkDaemonExists(options.daemonPath);

  // TODO: Phase 1 - Start daemon and wait for health check
  console.log('⏳ Starting daemon...');
  // Implementation will use DaemonFixture class from task 1.2

  // TODO: Phase 2 - Execute test suite
  console.log('⏳ Running test suite...');
  // Implementation will use TestExecutor from task 2.3

  // TODO: Phase 3 - Compare results
  console.log('⏳ Comparing results...');
  // Implementation will use ResponseComparator from task 3.1

  // TODO: Phase 4 - Apply auto-fix if enabled
  if (options.enableFix) {
    console.log('⏳ Applying auto-fixes...');
    // Implementation will use FixOrchestrator from task 4.3
  }

  // TODO: Phase 5 - Generate reports
  console.log('⏳ Generating reports...');
  // Implementation will use ValidationReporter from task 3.2

  // Placeholder: Report success
  console.log('\n✅ Test runner framework initialized');
  console.log('Note: Full implementation requires completion of phases 1-5');
}

// Handle cleanup on exit
process.on('SIGINT', () => {
  console.log('\n\n⚠️  Interrupted by user');
  // TODO: Cleanup daemon process
  process.exit(130);
});

process.on('SIGTERM', () => {
  console.log('\n\n⚠️  Terminated');
  // TODO: Cleanup daemon process
  process.exit(143);
});

// Run main function
main().catch((error) => {
  console.error('\n❌ Unexpected error:', error);
  process.exit(1);
});
