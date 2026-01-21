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
import { DaemonFixture } from './fixtures/daemon-fixture';
import { ApiClient } from './api-client/client';
import { createExecutor, TestSuiteResult, TestResult } from './test-executor/executor';
import { getAllTestCases } from './test-cases/api-tests';
import { createReporter } from './comparator/validation-reporter';
import { createOrchestrator, FixOrchestratorResult } from './auto-fix/fix-orchestrator';
import { createClassifier } from './auto-fix/issue-classifier';
import { createFixRegistry } from './auto-fix/fix-strategies';

interface CliOptions {
  daemonPath: string;
  port: number;
  maxIterations: number;
  enableFix: boolean;
  reportJsonPath?: string;
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

  // Phase 1: Start daemon and wait for health check
  console.log('⏳ Starting daemon...');
  const daemon = new DaemonFixture({
    daemonPath: options.daemonPath,
    port: options.port,
    debug: false,
  });

  globalDaemon = daemon; // Set for signal handler cleanup
  let daemonStarted = false;

  try {
    await daemon.start();
    await daemon.waitUntilReady(30000);
    daemonStarted = true;
    console.log('✓ Daemon started and ready\n');

    // Phase 2: Execute test suite
    console.log('⏳ Running test suite...');
    const apiClient = new ApiClient({
      baseUrl: `http://localhost:${options.port}`,
    });

    const testCases = getAllTestCases();
    const executor = createExecutor({ verbose: true });

    console.log(`Found ${testCases.length} test cases\n`);
    const startTime = Date.now();
    let testResults = await executor.runAll(apiClient, testCases);

    console.log(`\nInitial results: ${testResults.passed}/${testResults.total} passed\n`);

    // Phase 4: Apply auto-fix if enabled and tests failed
    let fixResults: FixOrchestratorResult | undefined;
    if (options.enableFix && testResults.failed > 0) {
      console.log('⏳ Applying auto-fixes...\n');

      const orchestrator = createOrchestrator({
        maxIterations: options.maxIterations,
        maxTotalTime: 5 * 60 * 1000, // 5 minutes
        daemon,
        apiClient,
        testCases,
        executor,
      });

      fixResults = await orchestrator.fixAndRetry(testResults.results);

      console.log(`\n✓ Auto-fix complete: ${fixResults.fixedTests} test(s) fixed\n`);

      // Re-run all tests to get final results
      console.log('⏳ Running final test suite...\n');
      testResults = await executor.runAll(apiClient, testCases);
    }

    const totalDuration = Date.now() - startTime;

    // Phase 5: Generate reports
    console.log('\n⏳ Generating reports...\n');
    const reporter = createReporter();

    // Print human-readable report
    console.log(reporter.formatHuman(testResults));

    // Save JSON report if requested (with complete data for HTML reporter)
    if (options.reportJsonPath) {
      const reportData = {
        testSuite: testResults,
        fixResults,
        timestamp: new Date().toISOString(),
        duration: totalDuration,
      };
      const jsonReport = JSON.stringify(reportData, null, 2);
      fs.writeFileSync(options.reportJsonPath, jsonReport, 'utf-8');
      console.log(`✓ JSON report saved: ${options.reportJsonPath}\n`);
    }

    // Exit with appropriate code
    if (testResults.failed === 0 && testResults.errors === 0) {
      console.log('✅ All tests passed!');
      await daemon.stop();
      process.exit(0);
    } else {
      console.log(`❌ ${testResults.failed + testResults.errors} test(s) failed`);
      await daemon.stop();
      process.exit(1);
    }
  } catch (error) {
    console.error('\n❌ Error during test execution:', error);

    if (daemonStarted) {
      console.log('⏳ Stopping daemon...');
      await daemon.stop();
    }

    if (error instanceof Error && error.message.includes('failed to become ready')) {
      console.error('\nDaemon failed to start. Check logs above for details.');
      process.exit(2);
    }

    process.exit(1);
  }
}

// Global daemon reference for cleanup
let globalDaemon: DaemonFixture | null = null;

// Handle cleanup on exit
process.on('SIGINT', async () => {
  console.log('\n\n⚠️  Interrupted by user');
  if (globalDaemon) {
    console.log('⏳ Stopping daemon...');
    await globalDaemon.stop();
  }
  process.exit(130);
});

process.on('SIGTERM', async () => {
  console.log('\n\n⚠️  Terminated');
  if (globalDaemon) {
    console.log('⏳ Stopping daemon...');
    await globalDaemon.stop();
  }
  process.exit(143);
});

// Run main function
main().catch((error) => {
  console.error('\n❌ Unexpected error:', error);
  if (globalDaemon) {
    globalDaemon.stop().then(() => process.exit(1));
  } else {
    process.exit(1);
  }
});
