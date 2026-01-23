#!/usr/bin/env node

/**
 * Automated E2E Test Runner
 *
 * Orchestrates automated end-to-end testing of the KeyRx daemon REST API.
 * Manages daemon lifecycle, executes test suite, and optionally applies auto-fixes.
 *
 * Usage:
 *   npx tsx scripts/automated-e2e-test.ts [options]
 *
 * Options:
 *   --daemon-path <path>      Path to daemon binary (default: target/release/keyrx_daemon)
 *   --port <number>           Port for daemon API (default: 9867)
 *   --max-iterations <number> Max auto-fix iterations (default: 3)
 *   --fix                     Enable auto-fix mode
 *   --test-mode               Run daemon in test mode (enables IPC)
 *   --report-json <path>      Output JSON report to file
 *   --help                    Show this help message
 */

import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import { DaemonFixture } from './fixtures/daemon-fixture.js';
import { ApiClient } from './api-client/client.js';
import { TestExecutor } from './test-executor/executor.js';
import { ValidationReporter } from './comparator/validation-reporter.js';
import { FixOrchestrator } from './auto-fix/fix-orchestrator.js';
import { getAllTestCases } from './test-cases/api-tests.js';
import { TestMetrics } from './metrics/test-metrics.js';
import type { ExpectedResults } from './test-cases/types.js';
import type { TestSuiteResult as ExecutorTestSuiteResult } from './test-cases/types.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// CLI Options
interface CliOptions {
  daemonPath: string;
  port: number;
  maxIterations: number;
  fix: boolean;
  testMode: boolean;
  reportJson?: string;
  metricsFile?: string;
}

// Test result types
interface TestResult {
  id: string;
  name: string;
  status: 'pass' | 'fail' | 'skip';
  duration: number;
  error?: string;
  actual?: unknown;
  expected?: unknown;
  fixAttempts?: number;
  fixSuccesses?: number;
}

interface TestSuiteResult {
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  duration: number;
  results: TestResult[];
}

// Type alias for backward compatibility
type DaemonManager = DaemonFixture;

// Load expected results database
function loadExpectedResults(): ExpectedResults {
  const expectedResultsPath = path.join(__dirname, 'fixtures', 'expected-results.json');

  if (!fs.existsSync(expectedResultsPath)) {
    console.warn('⚠️  Expected results file not found, using defaults');
    return {
      version: '1.0',
      apiVersion: '0.1.0',
      endpoints: {},
    };
  }

  const content = fs.readFileSync(expectedResultsPath, 'utf-8');
  return JSON.parse(content);
}

// Test execution using TestExecutor and ApiClient
async function executeTests(port: number): Promise<TestSuiteResult> {
  console.log('\n📋 Loading test suite...');

  // Create API client
  const apiClient = new ApiClient({
    baseUrl: `http://localhost:${port}`,
    timeout: 5000,
    maxRetries: 3,
  });

  // Load expected results
  const expectedResults = loadExpectedResults();

  // Get all test cases
  const testCases = getAllTestCases();
  console.log(`✓ Loaded ${testCases.length} test cases`);

  // Create test executor
  const executor = new TestExecutor({
    testTimeout: 30000,
    verbose: false,
    expectedResults,
  });

  // Run all tests
  console.log('\n🧪 Executing test suite...\n');
  const executorResult = await executor.runAll(apiClient, testCases);

  // Convert executor result to our TestSuiteResult format
  const result: TestSuiteResult = {
    total: executorResult.total,
    passed: executorResult.passed,
    failed: executorResult.failed,
    skipped: executorResult.skipped,
    duration: executorResult.duration,
    results: executorResult.results.map((r) => ({
      id: r.id,
      name: r.name,
      status: r.status === 'passed' ? 'pass' : r.status === 'skipped' ? 'skip' : 'fail',
      duration: r.duration,
      error: r.error,
      actual: r.actual,
      expected: r.expected,
    })),
  };

  return result;
}

// Auto-fix engine using FixOrchestrator
async function applyAutoFixes(
  results: TestSuiteResult,
  daemonFixture: DaemonFixture,
  port: number,
  maxIterations: number
): Promise<TestSuiteResult> {
  console.log(`\n🔧 Starting auto-fix engine (max ${maxIterations} iterations)...`);

  // Create API client
  const apiClient = new ApiClient({
    baseUrl: `http://localhost:${port}`,
    timeout: 5000,
    maxRetries: 3,
  });

  // Load test cases and expected results
  const testCases = getAllTestCases();
  const expectedResults = loadExpectedResults();

  // Create test executor for retrying tests
  const testExecutor = new TestExecutor({
    testTimeout: 30000,
    verbose: false,
    expectedResults,
  });

  // Create fix orchestrator
  const fixOrchestrator = new FixOrchestrator({
    maxIterations,
    globalTimeout: 5 * 60 * 1000, // 5 minutes
    verbose: true,
  });

  // Convert our results to executor format for fix orchestrator
  const executorResults = results.results.map((r) => ({
    id: r.id,
    name: r.name,
    endpoint: testCases.find((tc) => tc.id === r.id)?.endpoint ?? '',
    scenario: testCases.find((tc) => tc.id === r.id)?.scenario ?? '',
    category: testCases.find((tc) => tc.id === r.id)?.category ?? 'status',
    priority: testCases.find((tc) => tc.id === r.id)?.priority ?? 3,
    status: r.status === 'pass' ? 'passed' : r.status === 'skip' ? 'skipped' : 'failed',
    duration: r.duration,
    error: r.error,
    actual: r.actual,
    expected: r.expected,
  }));

  // Apply fixes
  const fixResult = await fixOrchestrator.fixAndRetry(
    executorResults,
    testCases,
    testExecutor,
    apiClient,
    {
      daemonFixture,
      apiClient,
      expectedResultsPath: path.join(__dirname, 'fixtures', 'expected-results.json'),
    }
  );

  console.log(`\n✓ Auto-fix complete: ${fixResult.fixedTests} fixed, ${fixResult.failedTests} still failing`);

  // Re-run all tests to get final results
  console.log('\n🔄 Running final test suite...');
  const finalExecutorResult = await testExecutor.runAll(apiClient, testCases);

  // Track fix attempts and successes per test
  const fixAttemptsByTest = new Map<string, { attempts: number; successes: number }>();
  for (const testResult of fixResult.testResults) {
    for (const fix of testResult.fixAttempts) {
      const existing = fixAttemptsByTest.get(testResult.testId) || { attempts: 0, successes: 0 };
      existing.attempts++;
      if (fix.success) {
        existing.successes++;
      }
      fixAttemptsByTest.set(testResult.testId, existing);
    }
  }

  // Convert back to our format with fix metrics
  return {
    total: finalExecutorResult.total,
    passed: finalExecutorResult.passed,
    failed: finalExecutorResult.failed,
    skipped: finalExecutorResult.skipped,
    duration: finalExecutorResult.duration,
    results: finalExecutorResult.results.map((r) => {
      const fixMetrics = fixAttemptsByTest.get(r.id);
      return {
        id: r.id,
        name: r.name,
        status: r.status === 'passed' ? 'pass' : r.status === 'skipped' ? 'skip' : 'fail',
        duration: r.duration,
        error: r.error,
        actual: r.actual,
        expected: r.expected,
        fixAttempts: fixMetrics?.attempts,
        fixSuccesses: fixMetrics?.successes,
      };
    }),
  };
}

// Report generation using ValidationReporter
function generateReport(results: TestSuiteResult, outputPath?: string): void {
  const reporter = new ValidationReporter();

  // Convert to format expected by ValidationReporter
  const reporterResult = {
    name: 'Automated API E2E Tests',
    total: results.total,
    passed: results.passed,
    failed: results.failed,
    skipped: results.skipped,
    errors: 0,
    duration: results.duration,
    timestamp: new Date().toISOString(),
    results: results.results.map((r) => ({
      id: r.id,
      name: r.name,
      status: r.status,
      duration: r.duration,
      error: r.error,
      actual: r.actual,
      expected: r.expected,
    })),
  };

  // Generate and save JSON report if path provided
  if (outputPath) {
    const jsonReport = reporter.formatJson(reporterResult);
    fs.writeFileSync(outputPath, JSON.stringify(jsonReport, null, 2));
    console.log(`\n📄 JSON report written to: ${outputPath}`);
  }
}

// Display human-readable summary using ValidationReporter
function displaySummary(results: TestSuiteResult): void {
  const reporter = new ValidationReporter();

  // Convert to format expected by ValidationReporter
  const reporterResult = {
    name: 'Automated API E2E Tests',
    total: results.total,
    passed: results.passed,
    failed: results.failed,
    skipped: results.skipped,
    errors: 0,
    duration: results.duration,
    timestamp: new Date().toISOString(),
    results: results.results.map((r) => ({
      id: r.id,
      name: r.name,
      status: r.status,
      duration: r.duration,
      error: r.error,
      actual: r.actual,
      expected: r.expected,
    })),
  };

  // Display formatted output
  const humanOutput = reporter.formatHuman(reporterResult);
  console.log(humanOutput);
}

// Parse CLI arguments
function parseArgs(): CliOptions {
  const args = process.argv.slice(2);
  const options: CliOptions = {
    daemonPath: path.join(process.cwd(), 'target', 'release', 'keyrx_daemon'),
    port: 9867,
    maxIterations: 3,
    fix: false,
    testMode: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    if (arg === '--help' || arg === '-h') {
      console.log(`
Automated E2E Test Runner

Usage: npx tsx scripts/automated-e2e-test.ts [options]

Options:
  --daemon-path <path>      Path to daemon binary (default: target/release/keyrx_daemon)
  --port <number>           Port for daemon API (default: 9867)
  --max-iterations <number> Max auto-fix iterations (default: 3)
  --fix                     Enable auto-fix mode
  --test-mode               Run daemon in test mode (enables IPC)
  --report-json <path>      Output JSON report to file
  --metrics-file <path>     Path to metrics JSONL file (default: metrics.jsonl)
  --help                    Show this help message
      `);
      process.exit(0);
    } else if (arg === '--daemon-path') {
      options.daemonPath = args[++i];
    } else if (arg === '--port') {
      options.port = parseInt(args[++i], 10);
    } else if (arg === '--max-iterations') {
      options.maxIterations = parseInt(args[++i], 10);
    } else if (arg === '--fix') {
      options.fix = true;
    } else if (arg === '--test-mode') {
      options.testMode = true;
    } else if (arg === '--report-json') {
      options.reportJson = args[++i];
    } else if (arg === '--metrics-file') {
      options.metricsFile = args[++i];
    }
  }

  // Add .exe extension on Windows if not present
  if (process.platform === 'win32' && !options.daemonPath.endsWith('.exe')) {
    options.daemonPath += '.exe';
  }

  return options;
}

// Main execution
async function main(): Promise<void> {
  const options = parseArgs();
  const daemonFixture = new DaemonFixture({
    daemonPath: options.daemonPath,
    port: options.port,
    testMode: options.testMode,
  });

  // Handle cleanup on exit
  let cleanupDone = false;
  const cleanup = async () => {
    if (cleanupDone) return;
    cleanupDone = true;
    console.log('\nCleaning up...');
    await daemonFixture.stop();
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('exit', () => {
    if (!cleanupDone) {
      console.log('Forcing cleanup on exit...');
    }
  });

  try {
    console.log('═'.repeat(80));
    console.log('🚀 Automated E2E Test Runner');
    console.log('═'.repeat(80));
    console.log(`Daemon:    ${options.daemonPath}`);
    console.log(`Port:      ${options.port}`);
    console.log(`Test mode: ${options.testMode ? 'enabled' : 'disabled'}`);
    console.log(`Auto-fix:  ${options.fix ? 'enabled' : 'disabled'}`);
    if (options.fix) {
      console.log(`Max iterations: ${options.maxIterations}`);
    }
    console.log('═'.repeat(80));

    // Start daemon
    console.log(`\n🔄 Starting daemon...`);
    await daemonFixture.start();
    console.log(`✓ Daemon is ready on port ${daemonFixture.getPort()}`);

    // Execute tests
    let results = await executeTests(daemonFixture.getPort());

    // Progress update after initial run
    console.log(`\n📊 Initial results: ${results.passed} passed, ${results.failed} failed, ${results.skipped} skipped`);

    // Apply auto-fixes if enabled and there are failures
    if (options.fix && results.failed > 0) {
      results = await applyAutoFixes(
        results,
        daemonFixture,
        daemonFixture.getPort(),
        options.maxIterations
      );

      // Final status after auto-fix
      if (results.failed === 0) {
        console.log('\n✅ All tests fixed!');
      } else {
        console.log(`\n⚠️  ${results.failed} test(s) still failing after auto-fix`);
      }
    }

    // Display results
    displaySummary(results);

    // Generate report
    if (options.reportJson) {
      generateReport(results, options.reportJson);
    }

    // Record metrics
    const metricsFile = options.metricsFile || 'metrics.jsonl';
    const metrics = new TestMetrics(metricsFile);
    await metrics.record({
      total: results.total,
      passed: results.passed,
      failed: results.failed,
      duration: results.duration,
      results: results.results,
    });
    console.log(`\n📊 Metrics recorded to: ${metricsFile}`);

    // Exit with appropriate code
    const exitCode = results.failed > 0 ? 1 : 0;

    // Clean up
    await cleanup();

    process.exit(exitCode);
  } catch (error) {
    console.error('\n❌ Error during test execution:', error);

    // Try to collect daemon logs
    const logs = daemonFixture.getLogs();
    if (logs.length > 0) {
      console.error('\n📋 Daemon logs:');
      console.error(logs.join('\n'));
    }

    await cleanup();
    process.exit(1);
  }
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error('Unhandled error:', error);
    process.exit(1);
  });
}

// Export for testing
export { parseArgs, executeTests, applyAutoFixes, generateReport };
