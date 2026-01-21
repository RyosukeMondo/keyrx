#!/usr/bin/env tsx
/**
 * HTML Test Report Generator
 *
 * Generates standalone HTML reports for automated E2E test results.
 * The HTML file contains embedded data and is fully functional offline.
 *
 * Features:
 * - Visual summary dashboard with pass/fail statistics
 * - Filterable test list (all/passed/failed)
 * - Syntax-highlighted JSON diffs
 * - Fix attempt history visualization
 * - Responsive design for mobile/desktop
 * - No external dependencies (works offline)
 *
 * Usage:
 *   npx tsx html-reporter.ts <input.json> <output.html>
 */

import fs from 'fs/promises';
import path from 'path';
import type { TestSuiteResult } from '../comparator/validation-reporter.js';
import type { OrchestrationResult } from '../auto-fix/fix-orchestrator.js';

/**
 * Combined report data structure
 */
export interface CombinedReportData {
  testSuite: TestSuiteResult;
  autoFix?: OrchestrationResult;
}

/**
 * HTML report generator
 */
export class HtmlReporter {
  /**
   * Generate HTML report from test results
   *
   * @param data - Combined test suite and auto-fix results
   * @param outputPath - Path to write HTML file
   */
  async generate(data: CombinedReportData, outputPath: string): Promise<void> {
    const html = this.generateHtml(data);
    await fs.writeFile(outputPath, html, 'utf-8');
  }

  /**
   * Generate complete HTML document
   */
  private generateHtml(data: CombinedReportData): string {
    const json = JSON.stringify(data, null, 2);

    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>E2E Test Report - ${new Date(data.testSuite.timestamp || '').toLocaleString()}</title>
    <style>
${this.generateCss()}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🧪 Automated E2E Test Report</h1>
            <p class="timestamp">Generated: ${new Date(data.testSuite.timestamp || '').toLocaleString()}</p>
        </header>

        <div id="app"></div>
    </div>

    <script>
        const DATA = ${json};
    </script>
    <script>
${this.generateJavaScript()}
    </script>
</body>
</html>`;
  }

  /**
   * Generate CSS styles
   */
  private generateCss(): string {
    return `
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: #f5f5f5;
    color: #333;
    line-height: 1.6;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

header {
    background: white;
    padding: 30px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    margin-bottom: 20px;
}

header h1 {
    font-size: 2em;
    margin-bottom: 10px;
}

.timestamp {
    color: #666;
    font-size: 0.9em;
}

.summary-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 15px;
    margin-bottom: 20px;
}

.card {
    background: white;
    padding: 20px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.card h3 {
    font-size: 0.9em;
    color: #666;
    margin-bottom: 10px;
    text-transform: uppercase;
}

.card .value {
    font-size: 2em;
    font-weight: bold;
}

.card.pass .value { color: #22c55e; }
.card.fail .value { color: #ef4444; }
.card.skip .value { color: #f59e0b; }
.card.duration .value { font-size: 1.5em; }

.pass-rate {
    margin-top: 10px;
    font-size: 0.9em;
    color: #666;
}

.progress-bar {
    width: 100%;
    height: 8px;
    background: #e5e7eb;
    border-radius: 4px;
    overflow: hidden;
    margin-top: 10px;
}

.progress-fill {
    height: 100%;
    background: #22c55e;
    transition: width 0.3s ease;
}

.progress-fill.low { background: #ef4444; }
.progress-fill.medium { background: #f59e0b; }
.progress-fill.high { background: #22c55e; }

.filters {
    background: white;
    padding: 20px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    margin-bottom: 20px;
}

.filter-buttons {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}

.btn {
    padding: 8px 16px;
    border: 2px solid #e5e7eb;
    background: white;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9em;
    transition: all 0.2s;
}

.btn:hover {
    border-color: #3b82f6;
    background: #eff6ff;
}

.btn.active {
    border-color: #3b82f6;
    background: #3b82f6;
    color: white;
}

.test-list {
    display: flex;
    flex-direction: column;
    gap: 15px;
}

.test-item {
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    overflow: hidden;
}

.test-header {
    padding: 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    user-select: none;
}

.test-header:hover {
    background: #f9fafb;
}

.test-info {
    flex: 1;
}

.test-name {
    font-weight: 600;
    font-size: 1.1em;
    margin-bottom: 5px;
}

.test-id {
    color: #666;
    font-size: 0.85em;
}

.test-status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: 6px;
    font-weight: 600;
    font-size: 0.9em;
}

.test-status.pass {
    background: #dcfce7;
    color: #166534;
}

.test-status.fail {
    background: #fee2e2;
    color: #991b1b;
}

.test-status.skip {
    background: #fef3c7;
    color: #92400e;
}

.test-status.error {
    background: #fce7f3;
    color: #831843;
}

.test-duration {
    color: #666;
    font-size: 0.85em;
    margin-left: 10px;
}

.test-details {
    border-top: 1px solid #e5e7eb;
    padding: 20px;
    display: none;
}

.test-details.expanded {
    display: block;
}

.details-section {
    margin-bottom: 20px;
}

.details-section:last-child {
    margin-bottom: 0;
}

.details-section h4 {
    font-size: 1em;
    margin-bottom: 10px;
    color: #666;
    text-transform: uppercase;
}

.error-message {
    background: #fef2f2;
    border-left: 4px solid #ef4444;
    padding: 15px;
    border-radius: 4px;
    color: #991b1b;
    font-family: monospace;
    font-size: 0.9em;
}

.diff-container {
    background: #1f2937;
    border-radius: 6px;
    overflow: hidden;
    margin-top: 10px;
}

.diff-header {
    background: #111827;
    padding: 10px 15px;
    color: #9ca3af;
    font-size: 0.85em;
    border-bottom: 1px solid #374151;
}

.diff-content {
    padding: 15px;
    overflow-x: auto;
    max-height: 500px;
}

.diff-content pre {
    color: #e5e7eb;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 0.85em;
    line-height: 1.5;
}

.diff-line {
    display: block;
}

.diff-line.added {
    background: rgba(34, 197, 94, 0.2);
    color: #86efac;
}

.diff-line.removed {
    background: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
}

.diff-line.same {
    color: #9ca3af;
}

.fix-attempts {
    list-style: none;
}

.fix-attempt {
    background: #f9fafb;
    border-left: 4px solid #e5e7eb;
    padding: 15px;
    border-radius: 4px;
    margin-bottom: 10px;
}

.fix-attempt.success {
    border-left-color: #22c55e;
    background: #f0fdf4;
}

.fix-attempt.failed {
    border-left-color: #ef4444;
    background: #fef2f2;
}

.fix-attempt-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
}

.fix-strategy {
    font-weight: 600;
    font-size: 0.95em;
}

.fix-iteration {
    color: #666;
    font-size: 0.85em;
}

.fix-message {
    color: #666;
    font-size: 0.9em;
}

.auto-fix-summary {
    background: white;
    padding: 20px;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    margin-bottom: 20px;
}

.auto-fix-summary h2 {
    margin-bottom: 15px;
}

.auto-fix-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 15px;
}

.stat-item {
    padding: 10px;
    background: #f9fafb;
    border-radius: 6px;
}

.stat-label {
    font-size: 0.85em;
    color: #666;
    margin-bottom: 5px;
}

.stat-value {
    font-size: 1.5em;
    font-weight: bold;
}

@media (max-width: 768px) {
    .container {
        padding: 10px;
    }

    header {
        padding: 20px;
    }

    header h1 {
        font-size: 1.5em;
    }

    .summary-cards {
        grid-template-columns: 1fr;
    }

    .test-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 10px;
    }

    .card .value {
        font-size: 1.5em;
    }
}
    `;
  }

  /**
   * Generate JavaScript for interactive functionality
   */
  private generateJavaScript(): string {
    return `
// Application state
let currentFilter = 'all';
let expandedTests = new Set();

// Initialize app
function init() {
    renderApp();
    attachEventListeners();
}

// Render the application
function renderApp() {
    const app = document.getElementById('app');

    const html = \`
        \${renderAutoFixSummary()}
        \${renderSummaryCards()}
        \${renderFilters()}
        \${renderTestList()}
    \`;

    app.innerHTML = html;
    attachEventListeners();
}

// Render auto-fix summary if available
function renderAutoFixSummary() {
    if (!DATA.autoFix) {
        return '';
    }

    const af = DATA.autoFix;
    const fixSuccessRate = af.totalFixAttempts > 0
        ? Math.round((af.successfulFixAttempts / af.totalFixAttempts) * 100)
        : 0;

    return \`
        <div class="auto-fix-summary">
            <h2>🔧 Auto-Fix Summary</h2>
            <div class="auto-fix-stats">
                <div class="stat-item">
                    <div class="stat-label">Tests Fixed</div>
                    <div class="stat-value" style="color: #22c55e;">\${af.fixedTests}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Tests Failed</div>
                    <div class="stat-value" style="color: #ef4444;">\${af.failedTests}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Tests Skipped</div>
                    <div class="stat-value" style="color: #f59e0b;">\${af.skippedTests}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Fix Attempts</div>
                    <div class="stat-value">\${af.totalFixAttempts}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Success Rate</div>
                    <div class="stat-value">\${fixSuccessRate}%</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Duration</div>
                    <div class="stat-value">\${formatDuration(af.duration)}</div>
                </div>
            </div>
        </div>
    \`;
}

// Render summary cards
function renderSummaryCards() {
    const suite = DATA.testSuite;
    const passRate = suite.total > 0
        ? Math.round((suite.passed / suite.total) * 100)
        : 0;

    const passRateClass = passRate >= 80 ? 'high' : passRate >= 50 ? 'medium' : 'low';

    return \`
        <div class="summary-cards">
            <div class="card">
                <h3>Total Tests</h3>
                <div class="value">\${suite.total}</div>
            </div>
            <div class="card pass">
                <h3>Passed</h3>
                <div class="value">\${suite.passed}</div>
            </div>
            <div class="card fail">
                <h3>Failed</h3>
                <div class="value">\${suite.failed}</div>
            </div>
            \${suite.skipped > 0 ? \`
            <div class="card skip">
                <h3>Skipped</h3>
                <div class="value">\${suite.skipped}</div>
            </div>
            \` : ''}
            <div class="card duration">
                <h3>Duration</h3>
                <div class="value">\${formatDuration(suite.duration)}</div>
            </div>
            <div class="card">
                <h3>Pass Rate</h3>
                <div class="value">\${passRate}%</div>
                <div class="progress-bar">
                    <div class="progress-fill \${passRateClass}" style="width: \${passRate}%"></div>
                </div>
            </div>
        </div>
    \`;
}

// Render filters
function renderFilters() {
    return \`
        <div class="filters">
            <div class="filter-buttons">
                <button class="btn \${currentFilter === 'all' ? 'active' : ''}" data-filter="all">
                    All (\${DATA.testSuite.total})
                </button>
                <button class="btn \${currentFilter === 'pass' ? 'active' : ''}" data-filter="pass">
                    ✓ Passed (\${DATA.testSuite.passed})
                </button>
                <button class="btn \${currentFilter === 'fail' ? 'active' : ''}" data-filter="fail">
                    ✗ Failed (\${DATA.testSuite.failed})
                </button>
                \${DATA.testSuite.skipped > 0 ? \`
                <button class="btn \${currentFilter === 'skip' ? 'active' : ''}" data-filter="skip">
                    ○ Skipped (\${DATA.testSuite.skipped})
                </button>
                \` : ''}
                \${DATA.testSuite.errors > 0 ? \`
                <button class="btn \${currentFilter === 'error' ? 'active' : ''}" data-filter="error">
                    ⚠ Errors (\${DATA.testSuite.errors})
                </button>
                \` : ''}
            </div>
        </div>
    \`;
}

// Render test list
function renderTestList() {
    const filteredTests = DATA.testSuite.results.filter(test => {
        if (currentFilter === 'all') return true;
        return test.status === currentFilter;
    });

    if (filteredTests.length === 0) {
        return \`
            <div class="card">
                <p style="text-align: center; color: #666; padding: 40px;">
                    No tests match the current filter.
                </p>
            </div>
        \`;
    }

    return \`
        <div class="test-list">
            \${filteredTests.map(test => renderTestItem(test)).join('')}
        </div>
    \`;
}

// Render individual test item
function renderTestItem(test) {
    const isExpanded = expandedTests.has(test.id);
    const autoFixData = DATA.autoFix?.testResults.find(r => r.testId === test.id);

    return \`
        <div class="test-item" data-test-id="\${test.id}">
            <div class="test-header">
                <div class="test-info">
                    <div class="test-name">\${escapeHtml(test.name)}</div>
                    <div class="test-id">\${escapeHtml(test.id)}</div>
                </div>
                <div>
                    <span class="test-status \${test.status}">
                        \${getStatusIcon(test.status)} \${test.status.toUpperCase()}
                    </span>
                    <span class="test-duration">\${formatDuration(test.duration)}</span>
                </div>
            </div>
            \${(test.error || test.comparison || autoFixData) ? \`
            <div class="test-details \${isExpanded ? 'expanded' : ''}">
                \${renderTestDetails(test, autoFixData)}
            </div>
            \` : ''}
        </div>
    \`;
}

// Render test details
function renderTestDetails(test, autoFixData) {
    let html = '';

    // Error message
    if (test.error) {
        html += \`
            <div class="details-section">
                <h4>Error</h4>
                <div class="error-message">\${escapeHtml(test.error)}</div>
            </div>
        \`;
    }

    // Comparison diffs
    if (test.comparison && !test.comparison.matches) {
        html += \`
            <div class="details-section">
                <h4>Differences</h4>
                \${renderDiffs(test)}
            </div>
        \`;
    }

    // Auto-fix attempts
    if (autoFixData && autoFixData.fixAttempts.length > 0) {
        html += \`
            <div class="details-section">
                <h4>Fix Attempts (\${autoFixData.fixAttempts.length})</h4>
                <ul class="fix-attempts">
                    \${autoFixData.fixAttempts.map(attempt => renderFixAttempt(attempt)).join('')}
                </ul>
            </div>
        \`;
    }

    return html;
}

// Render diffs
function renderDiffs(test) {
    if (!test.expected || !test.actual) {
        return '<p style="color: #666;">No diff data available</p>';
    }

    const expectedJson = JSON.stringify(test.expected, null, 2);
    const actualJson = JSON.stringify(test.actual, null, 2);

    const expectedLines = expectedJson.split('\\n');
    const actualLines = actualJson.split('\\n');

    return \`
        <div class="diff-container">
            <div class="diff-header">Expected vs Actual</div>
            <div class="diff-content">
                <pre>\${renderSideBySideDiff(expectedLines, actualLines)}</pre>
            </div>
        </div>
    \`;
}

// Render side-by-side diff
function renderSideBySideDiff(expectedLines, actualLines) {
    const maxLines = Math.max(expectedLines.length, actualLines.length);
    const limit = 100; // Max 100 lines

    let html = '';

    for (let i = 0; i < Math.min(maxLines, limit); i++) {
        const expectedLine = i < expectedLines.length ? expectedLines[i] : '';
        const actualLine = i < actualLines.length ? actualLines[i] : '';

        if (expectedLine === actualLine) {
            html += \`<span class="diff-line same">  \${escapeHtml(expectedLine)}\\n</span>\`;
        } else {
            if (expectedLine) {
                html += \`<span class="diff-line removed">- \${escapeHtml(expectedLine)}\\n</span>\`;
            }
            if (actualLine) {
                html += \`<span class="diff-line added">+ \${escapeHtml(actualLine)}\\n</span>\`;
            }
        }
    }

    if (maxLines > limit) {
        html += \`<span class="diff-line same">... (\${maxLines - limit} more lines)\\n</span>\`;
    }

    return html;
}

// Render fix attempt
function renderFixAttempt(attempt) {
    const statusClass = attempt.success ? 'success' : 'failed';

    return \`
        <li class="fix-attempt \${statusClass}">
            <div class="fix-attempt-header">
                <span class="fix-strategy">
                    \${attempt.success ? '✓' : '✗'} \${escapeHtml(attempt.strategyName)}
                </span>
                <span class="fix-iteration">Iteration \${attempt.iteration}</span>
            </div>
            <div class="fix-message">\${escapeHtml(attempt.message)}</div>
            \${attempt.error ? \`
                <div class="error-message" style="margin-top: 10px;">
                    \${escapeHtml(attempt.error)}
                </div>
            \` : ''}
        </li>
    \`;
}

// Attach event listeners
function attachEventListeners() {
    // Filter buttons
    document.querySelectorAll('.btn[data-filter]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            currentFilter = e.target.dataset.filter;
            renderApp();
        });
    });

    // Test headers (expand/collapse)
    document.querySelectorAll('.test-header').forEach(header => {
        header.addEventListener('click', (e) => {
            const testItem = e.currentTarget.closest('.test-item');
            const testId = testItem.dataset.testId;
            const details = testItem.querySelector('.test-details');

            if (details) {
                if (expandedTests.has(testId)) {
                    expandedTests.delete(testId);
                    details.classList.remove('expanded');
                } else {
                    expandedTests.add(testId);
                    details.classList.add('expanded');
                }
            }
        });
    });
}

// Utility functions
function formatDuration(ms) {
    if (ms < 1000) {
        return \`\${ms.toFixed(0)}ms\`;
    } else if (ms < 60000) {
        return \`\${(ms / 1000).toFixed(2)}s\`;
    } else {
        const minutes = Math.floor(ms / 60000);
        const seconds = ((ms % 60000) / 1000).toFixed(0);
        return \`\${minutes}m \${seconds}s\`;
    }
}

function getStatusIcon(status) {
    switch (status) {
        case 'pass': return '✓';
        case 'fail': return '✗';
        case 'skip': return '○';
        case 'error': return '⚠';
        default: return '';
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Start the app
init();
    `;
  }

  /**
   * Create HTML reporter instance
   */
  static create(): HtmlReporter {
    return new HtmlReporter();
  }
}

/**
 * Main CLI entry point
 */
async function main() {
  const args = process.argv.slice(2);

  if (args.length < 2) {
    console.error('Usage: html-reporter.ts <input.json> <output.html>');
    console.error('');
    console.error('Example:');
    console.error('  npx tsx html-reporter.ts test-results.json report.html');
    process.exit(1);
  }

  const [inputPath, outputPath] = args;

  try {
    // Read input JSON
    const jsonContent = await fs.readFile(inputPath, 'utf-8');
    const data: CombinedReportData = JSON.parse(jsonContent);

    // Validate data structure
    if (!data.testSuite) {
      throw new Error('Invalid input: missing testSuite property');
    }

    // Generate HTML report
    const reporter = HtmlReporter.create();
    await reporter.generate(data, outputPath);

    console.log(`✓ HTML report generated: ${outputPath}`);
    console.log(`  Tests: ${data.testSuite.total}`);
    console.log(`  Passed: ${data.testSuite.passed}`);
    console.log(`  Failed: ${data.testSuite.failed}`);
    if (data.autoFix) {
      console.log(`  Auto-fixed: ${data.autoFix.fixedTests}`);
    }

    // Check file size
    const stats = await fs.stat(outputPath);
    const sizeKB = Math.round(stats.size / 1024);
    console.log(`  File size: ${sizeKB}KB`);

    if (stats.size > 500 * 1024) {
      console.warn(`⚠ Warning: File size (${sizeKB}KB) exceeds 500KB limit`);
    }
  } catch (error) {
    console.error('Error generating HTML report:');
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { HtmlReporter };
