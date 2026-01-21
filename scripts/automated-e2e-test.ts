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
 *   --report-json <path>      Output JSON report to file
 *   --help                    Show this help message
 */

import { spawn, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// CLI Options
interface CliOptions {
  daemonPath: string;
  port: number;
  maxIterations: number;
  fix: boolean;
  reportJson?: string;
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
}

interface TestSuiteResult {
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  duration: number;
  results: TestResult[];
}

// Daemon management
class DaemonManager {
  private process?: ChildProcess;
  private readonly daemonPath: string;
  private readonly port: number;
  private readonly logs: string[] = [];

  constructor(daemonPath: string, port: number) {
    this.daemonPath = daemonPath;
    this.port = port;
  }

  /**
   * Start the daemon process
   */
  async start(): Promise<void> {
    console.log(`Starting daemon: ${this.daemonPath} on port ${this.port}...`);

    // Check if daemon binary exists
    if (!fs.existsSync(this.daemonPath)) {
      throw new Error(`Daemon binary not found: ${this.daemonPath}`);
    }

    // Determine platform-specific command
    const isWindows = process.platform === 'win32';
    const daemonArgs = ['--port', this.port.toString()];

    this.process = spawn(this.daemonPath, daemonArgs, {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: {
        ...process.env,
        RUST_LOG: 'debug',
      },
    });

    // Capture stdout
    this.process.stdout?.on('data', (data) => {
      const output = data.toString();
      this.logs.push(output);
      if (process.env.DEBUG) {
        console.log('[daemon stdout]', output);
      }
    });

    // Capture stderr
    this.process.stderr?.on('data', (data) => {
      const output = data.toString();
      this.logs.push(output);
      if (process.env.DEBUG) {
        console.error('[daemon stderr]', output);
      }
    });

    // Handle process exit
    this.process.on('exit', (code, signal) => {
      console.log(`Daemon exited with code ${code}, signal ${signal}`);
    });

    // Wait for daemon to be ready
    await this.waitUntilReady(30000);
    console.log('Daemon is ready');
  }

  /**
   * Wait for daemon to be healthy
   */
  private async waitUntilReady(timeoutMs: number): Promise<void> {
    const startTime = Date.now();
    const checkInterval = 100; // ms

    while (Date.now() - startTime < timeoutMs) {
      try {
        const response = await fetch(`http://localhost:${this.port}/api/status`);
        if (response.ok) {
          const data = await response.json();
          if (data.status === 'ready' || data.running === true) {
            return;
          }
        }
      } catch (error) {
        // Connection refused or other error - daemon not ready yet
      }

      await new Promise((resolve) => setTimeout(resolve, checkInterval));
    }

    throw new Error(`Daemon failed to become ready within ${timeoutMs}ms`);
  }

  /**
   * Stop the daemon gracefully
   */
  async stop(): Promise<void> {
    if (!this.process) {
      return;
    }

    console.log('Stopping daemon...');

    return new Promise((resolve) => {
      if (!this.process) {
        resolve();
        return;
      }

      const killTimeout = 5000; // 5 seconds

      // Set up timeout to force kill
      const forceKillTimer = setTimeout(() => {
        if (this.process && !this.process.killed) {
          console.warn('Daemon did not stop gracefully, force killing...');
          this.process.kill('SIGKILL');
        }
      }, killTimeout);

      // Handle process exit
      this.process.once('exit', () => {
        clearTimeout(forceKillTimer);
        console.log('Daemon stopped');
        resolve();
      });

      // Send termination signal
      if (process.platform === 'win32') {
        // Windows doesn't support SIGTERM
        this.process.kill('SIGINT');
      } else {
        this.process.kill('SIGTERM');
      }
    });
  }

  /**
   * Get collected logs
   */
  getLogs(): string[] {
    return this.logs;
  }
}

// Test execution (placeholder - will be implemented in later tasks)
async function executeTests(port: number): Promise<TestSuiteResult> {
  console.log('Executing test suite...');

  // Placeholder: Return empty test results
  // This will be replaced with actual test execution in Task 2
  const result: TestSuiteResult = {
    total: 0,
    passed: 0,
    failed: 0,
    skipped: 0,
    duration: 0,
    results: [],
  };

  console.log('Test execution complete (placeholder)');
  return result;
}

// Auto-fix engine (placeholder - will be implemented in later tasks)
async function applyAutoFixes(
  results: TestSuiteResult,
  daemonManager: DaemonManager,
  port: number,
  iteration: number,
  maxIterations: number
): Promise<TestSuiteResult> {
  console.log(`Auto-fix iteration ${iteration}/${maxIterations} (placeholder)`);

  // Placeholder: Return original results unchanged
  // This will be replaced with actual auto-fix logic in Task 4
  return results;
}

// Report generation
function generateReport(results: TestSuiteResult, outputPath?: string): void {
  const report = {
    version: '1.0',
    timestamp: new Date().toISOString(),
    summary: {
      total: results.total,
      passed: results.passed,
      failed: results.failed,
      skipped: results.skipped,
      duration: results.duration,
    },
    results: results.results,
  };

  if (outputPath) {
    fs.writeFileSync(outputPath, JSON.stringify(report, null, 2));
    console.log(`JSON report written to: ${outputPath}`);
  }
}

// Display human-readable summary
function displaySummary(results: TestSuiteResult): void {
  console.log('\n=== Test Summary ===');
  console.log(`Total:   ${results.total}`);
  console.log(`Passed:  ${results.passed}`);
  console.log(`Failed:  ${results.failed}`);
  console.log(`Skipped: ${results.skipped}`);
  console.log(`Duration: ${results.duration}ms`);

  if (results.failed > 0) {
    console.log('\nFailed tests:');
    results.results
      .filter((r) => r.status === 'fail')
      .forEach((r) => {
        console.log(`  - ${r.name}: ${r.error}`);
      });
  }
}

// Parse CLI arguments
function parseArgs(): CliOptions {
  const args = process.argv.slice(2);
  const options: CliOptions = {
    daemonPath: path.join(process.cwd(), 'target', 'release', 'keyrx_daemon'),
    port: 9867,
    maxIterations: 3,
    fix: false,
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
  --report-json <path>      Output JSON report to file
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
    } else if (arg === '--report-json') {
      options.reportJson = args[++i];
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
  const daemonManager = new DaemonManager(options.daemonPath, options.port);

  // Handle cleanup on exit
  let cleanupDone = false;
  const cleanup = async () => {
    if (cleanupDone) return;
    cleanupDone = true;
    console.log('\nCleaning up...');
    await daemonManager.stop();
  };

  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('exit', () => {
    if (!cleanupDone) {
      console.log('Forcing cleanup on exit...');
    }
  });

  try {
    // Start daemon
    await daemonManager.start();

    // Execute tests
    let results = await executeTests(options.port);

    // Apply auto-fixes if enabled
    if (options.fix && results.failed > 0) {
      console.log('\nAuto-fix enabled, attempting to fix failures...');

      for (let iteration = 1; iteration <= options.maxIterations; iteration++) {
        results = await applyAutoFixes(
          results,
          daemonManager,
          options.port,
          iteration,
          options.maxIterations
        );

        if (results.failed === 0) {
          console.log('All tests fixed!');
          break;
        }

        if (iteration < options.maxIterations) {
          console.log(
            `${results.failed} test(s) still failing, retrying (${iteration}/${options.maxIterations})...`
          );
        }
      }
    }

    // Display results
    displaySummary(results);

    // Generate report
    if (options.reportJson) {
      generateReport(results, options.reportJson);
    }

    // Exit with appropriate code
    const exitCode = results.failed > 0 ? 1 : 0;

    // Clean up
    await cleanup();

    process.exit(exitCode);
  } catch (error) {
    console.error('Error during test execution:', error);

    // Try to collect daemon logs
    const logs = daemonManager.getLogs();
    if (logs.length > 0) {
      console.error('\nDaemon logs:');
      console.error(logs.join(''));
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
export { DaemonManager, parseArgs, executeTests, applyAutoFixes, generateReport };
