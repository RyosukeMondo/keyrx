/**
 * Test Metrics Collection
 *
 * Collects and tracks E2E test metrics over time for monitoring test health,
 * identifying flaky tests, and tracking performance trends.
 *
 * Uses JSON Lines format for efficient appending and querying.
 */

import * as fs from 'fs';
import * as path from 'path';
import { TestSuiteResult } from '../test-executor/executor';
import { FixOrchestratorResult } from '../auto-fix/fix-orchestrator';

export interface MetricsEntry {
  timestamp: string;
  totalTests: number;
  passedTests: number;
  failedTests: number;
  errorTests: number;
  duration: number;
  fixAttempts: number;
  fixSuccesses: number;
  averageTestDuration: number;
  slowestTests: Array<{ name: string; duration: number }>;
  gitCommit?: string;
  gitBranch?: string;
  ciRun?: boolean;
}

export interface TrendReport {
  totalRuns: number;
  averagePassRate: number;
  passRateTrend: Array<{ timestamp: string; passRate: number }>;
  averageDuration: number;
  durationTrend: Array<{ timestamp: string; duration: number }>;
  flakyTests: Array<{ name: string; failureRate: number; occurrences: number }>;
  slowestTests: Array<{ name: string; avgDuration: number; occurrences: number }>;
}

const DEFAULT_METRICS_FILE = path.join(process.cwd(), 'scripts', 'metrics', 'metrics.jsonl');
const MAX_LINES = 1000;

/**
 * Test metrics collector
 */
export class TestMetrics {
  private metricsFile: string;

  constructor(metricsFile: string = DEFAULT_METRICS_FILE) {
    this.metricsFile = metricsFile;
  }

  /**
   * Record test results to metrics file
   */
  record(
    testSuite: TestSuiteResult,
    fixResults?: FixOrchestratorResult,
    metadata?: {
      gitCommit?: string;
      gitBranch?: string;
      ciRun?: boolean;
    }
  ): void {
    // Calculate metrics
    const slowestTests = [...testSuite.results]
      .sort((a, b) => b.duration - a.duration)
      .slice(0, 5)
      .map(t => ({ name: t.name, duration: t.duration }));

    const entry: MetricsEntry = {
      timestamp: new Date().toISOString(),
      totalTests: testSuite.total,
      passedTests: testSuite.passed,
      failedTests: testSuite.failed,
      errorTests: testSuite.errors,
      duration: testSuite.duration,
      fixAttempts: fixResults?.results.reduce((sum, r) => sum + r.fixAttempts.length, 0) || 0,
      fixSuccesses: fixResults?.fixedTests || 0,
      averageTestDuration: testSuite.total > 0
        ? testSuite.results.reduce((sum, t) => sum + t.duration, 0) / testSuite.total
        : 0,
      slowestTests,
      gitCommit: metadata?.gitCommit,
      gitBranch: metadata?.gitBranch,
      ciRun: metadata?.ciRun,
    };

    // Ensure directory exists
    const dir = path.dirname(this.metricsFile);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }

    // Append to JSONL file
    const line = JSON.stringify(entry) + '\n';
    fs.appendFileSync(this.metricsFile, line, 'utf-8');

    // Rotate file if too large
    this.rotateIfNeeded();
  }

  /**
   * Load all metrics entries
   */
  loadAll(): MetricsEntry[] {
    if (!fs.existsSync(this.metricsFile)) {
      return [];
    }

    const content = fs.readFileSync(this.metricsFile, 'utf-8');
    const lines = content.trim().split('\n').filter(line => line.length > 0);

    return lines.map(line => JSON.parse(line) as MetricsEntry);
  }

  /**
   * Load metrics from last N days
   */
  loadRecent(days: number = 30): MetricsEntry[] {
    const all = this.loadAll();
    const cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - days);

    return all.filter(entry => new Date(entry.timestamp) >= cutoff);
  }

  /**
   * Generate trend report from metrics
   */
  report(days: number = 30): TrendReport {
    const entries = this.loadRecent(days);

    if (entries.length === 0) {
      return {
        totalRuns: 0,
        averagePassRate: 0,
        passRateTrend: [],
        averageDuration: 0,
        durationTrend: [],
        flakyTests: [],
        slowestTests: [],
      };
    }

    // Pass rate trend
    const passRateTrend = entries.map(entry => ({
      timestamp: entry.timestamp,
      passRate: entry.totalTests > 0 ? (entry.passedTests / entry.totalTests) * 100 : 0,
    }));

    const averagePassRate = passRateTrend.reduce((sum, t) => sum + t.passRate, 0) / passRateTrend.length;

    // Duration trend
    const durationTrend = entries.map(entry => ({
      timestamp: entry.timestamp,
      duration: entry.duration,
    }));

    const averageDuration = entries.reduce((sum, e) => sum + e.duration, 0) / entries.length;

    // Flaky tests (tests that sometimes pass, sometimes fail)
    const testStats = new Map<string, { total: number; failures: number }>();

    for (const entry of entries) {
      // Note: We don't have per-test data in metrics, so we can't track flakiness
      // This would require storing individual test results
      // For now, return empty array
    }

    // Slowest tests (average across runs)
    const testDurations = new Map<string, { total: number; count: number }>();

    for (const entry of entries) {
      for (const test of entry.slowestTests) {
        const existing = testDurations.get(test.name) || { total: 0, count: 0 };
        testDurations.set(test.name, {
          total: existing.total + test.duration,
          count: existing.count + 1,
        });
      }
    }

    const slowestTests = Array.from(testDurations.entries())
      .map(([name, stats]) => ({
        name,
        avgDuration: stats.total / stats.count,
        occurrences: stats.count,
      }))
      .sort((a, b) => b.avgDuration - a.avgDuration)
      .slice(0, 10);

    return {
      totalRuns: entries.length,
      averagePassRate,
      passRateTrend,
      averageDuration,
      durationTrend,
      flakyTests: [], // Not tracked yet (requires per-test data)
      slowestTests,
    };
  }

  /**
   * Rotate metrics file if it exceeds max lines
   */
  private rotateIfNeeded(): void {
    if (!fs.existsSync(this.metricsFile)) {
      return;
    }

    const content = fs.readFileSync(this.metricsFile, 'utf-8');
    const lines = content.trim().split('\n').filter(line => line.length > 0);

    if (lines.length > MAX_LINES) {
      // Keep most recent MAX_LINES entries
      const recentLines = lines.slice(-MAX_LINES);
      const newContent = recentLines.join('\n') + '\n';

      // Archive old file
      const archiveFile = this.metricsFile.replace('.jsonl', `.${Date.now()}.jsonl`);
      fs.renameSync(this.metricsFile, archiveFile);

      // Write recent entries
      fs.writeFileSync(this.metricsFile, newContent, 'utf-8');

      console.log(`Metrics file rotated. Archived to: ${archiveFile}`);
    }
  }

  /**
   * Clear all metrics
   */
  clear(): void {
    if (fs.existsSync(this.metricsFile)) {
      fs.unlinkSync(this.metricsFile);
    }
  }
}

/**
 * Print metrics report to console
 */
export function printReport(report: TrendReport): void {
  console.log('\n📊 Test Metrics Report (Last 30 days)\n');
  console.log(`Total Runs: ${report.totalRuns}`);
  console.log(`Average Pass Rate: ${report.averagePassRate.toFixed(1)}%`);
  console.log(`Average Duration: ${(report.averageDuration / 1000).toFixed(1)}s`);
  console.log('');

  if (report.passRateTrend.length > 0) {
    const recent = report.passRateTrend.slice(-10);
    console.log('Pass Rate Trend (Last 10 runs):');
    for (const entry of recent) {
      const date = new Date(entry.timestamp).toLocaleString();
      const bar = '█'.repeat(Math.round(entry.passRate / 5));
      console.log(`  ${date}: ${bar} ${entry.passRate.toFixed(1)}%`);
    }
    console.log('');
  }

  if (report.slowestTests.length > 0) {
    console.log('Slowest Tests (Top 10):');
    for (const test of report.slowestTests) {
      console.log(`  ${test.avgDuration.toFixed(0)}ms - ${test.name} (${test.occurrences} runs)`);
    }
    console.log('');
  }

  if (report.flakyTests.length > 0) {
    console.log('Flaky Tests (Top 10):');
    for (const test of report.flakyTests) {
      console.log(`  ${(test.failureRate * 100).toFixed(1)}% - ${test.name} (${test.occurrences} runs)`);
    }
    console.log('');
  }
}

// CLI support
if (require.main === module) {
  const args = process.argv.slice(2);
  const command = args[0];

  const metrics = new TestMetrics();

  switch (command) {
    case 'report':
      const days = args[1] ? parseInt(args[1], 10) : 30;
      const report = metrics.report(days);
      printReport(report);
      break;

    case 'clear':
      metrics.clear();
      console.log('✓ Metrics cleared');
      break;

    case 'export':
      if (!args[1]) {
        console.error('Usage: tsx test-metrics.ts export <output.json>');
        process.exit(1);
      }
      const entries = metrics.loadAll();
      fs.writeFileSync(args[1], JSON.stringify(entries, null, 2), 'utf-8');
      console.log(`✓ Exported ${entries.length} entries to ${args[1]}`);
      break;

    default:
      console.log('Test Metrics CLI\n');
      console.log('Usage:');
      console.log('  tsx test-metrics.ts report [days]    Show metrics report (default: 30 days)');
      console.log('  tsx test-metrics.ts clear             Clear all metrics');
      console.log('  tsx test-metrics.ts export <file>     Export metrics to JSON file');
      break;
  }
}
