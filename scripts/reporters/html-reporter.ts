/**
 * HTML Test Report Generator
 *
 * Generates visual HTML reports for test results with:
 * - Summary statistics and pass/fail indicators
 * - Test results with filtering by status
 * - Detailed diff visualization with syntax highlighting
 * - Fix attempt history for auto-fixed tests
 * - Standalone HTML (no external dependencies)
 */

import * as fs from 'fs';
import * as path from 'path';
import { TestResult, TestSuiteResult } from '../test-executor/executor';
import { FixedTestResult, FixOrchestratorResult } from '../auto-fix/fix-orchestrator';

export interface ReportData {
  testSuite: TestSuiteResult;
  fixResults?: FixOrchestratorResult;
  timestamp: string;
  duration: number;
}

/**
 * HTML report generator for test results
 */
export class HtmlReporter {
  /**
   * Generate standalone HTML report from test results
   */
  generate(data: ReportData, outputPath: string): void {
    const html = this.generateHtml(data);

    // Ensure output directory exists
    const dir = path.dirname(outputPath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }

    fs.writeFileSync(outputPath, html, 'utf-8');
    console.log(`HTML report generated: ${outputPath}`);
  }

  /**
   * Generate HTML content
   */
  private generateHtml(data: ReportData): string {
    const passRate = data.testSuite.total > 0
      ? ((data.testSuite.passed / data.testSuite.total) * 100).toFixed(1)
      : '0.0';

    const fixedCount = data.fixResults?.fixedTests || 0;
    const unfixedCount = data.fixResults?.unfixedTests || 0;

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>KeyRx E2E Test Report - ${data.timestamp}</title>
  <style>
    ${this.getStyles()}
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>KeyRx API E2E Test Report</h1>
      <div class="timestamp">${data.timestamp}</div>
    </header>

    <section class="summary">
      <div class="summary-card">
        <div class="summary-title">Total Tests</div>
        <div class="summary-value">${data.testSuite.total}</div>
      </div>
      <div class="summary-card pass">
        <div class="summary-title">Passed</div>
        <div class="summary-value">${data.testSuite.passed}</div>
      </div>
      <div class="summary-card fail">
        <div class="summary-title">Failed</div>
        <div class="summary-value">${data.testSuite.failed}</div>
      </div>
      <div class="summary-card error">
        <div class="summary-title">Errors</div>
        <div class="summary-value">${data.testSuite.errors}</div>
      </div>
      <div class="summary-card">
        <div class="summary-title">Pass Rate</div>
        <div class="summary-value">${passRate}%</div>
      </div>
      <div class="summary-card">
        <div class="summary-title">Duration</div>
        <div class="summary-value">${(data.duration / 1000).toFixed(1)}s</div>
      </div>
      ${data.fixResults ? `
      <div class="summary-card fixed">
        <div class="summary-title">Auto-Fixed</div>
        <div class="summary-value">${fixedCount}</div>
      </div>
      <div class="summary-card unfixed">
        <div class="summary-title">Unfixed</div>
        <div class="summary-value">${unfixedCount}</div>
      </div>
      ` : ''}
    </section>

    <section class="filters">
      <button class="filter-btn active" data-filter="all">All (${data.testSuite.total})</button>
      <button class="filter-btn" data-filter="pass">Passed (${data.testSuite.passed})</button>
      <button class="filter-btn" data-filter="fail">Failed (${data.testSuite.failed})</button>
      <button class="filter-btn" data-filter="error">Errors (${data.testSuite.errors})</button>
      ${data.fixResults && fixedCount > 0 ? `<button class="filter-btn" data-filter="fixed">Fixed (${fixedCount})</button>` : ''}
    </section>

    <section class="test-list">
      ${this.generateTestList(data.testSuite.results, data.fixResults?.results)}
    </section>
  </div>

  <script>
    ${this.getScript(data)}
  </script>
</body>
</html>`;
  }

  /**
   * Generate test list HTML
   */
  private generateTestList(tests: TestResult[], fixResults?: FixedTestResult[]): string {
    return tests.map((test, index) => {
      const fixResult = fixResults?.find(f => f.testId === test.id);
      const statusClass = test.status;
      const statusIcon = this.getStatusIcon(test.status);
      const hasFixAttempts = fixResult && fixResult.fixAttempts.length > 0;

      return `
      <div class="test-item" data-status="${test.status}" data-fixed="${hasFixAttempts ? 'true' : 'false'}">
        <div class="test-header" onclick="toggleTest(${index})">
          <div class="test-status ${statusClass}">${statusIcon}</div>
          <div class="test-name">${this.escapeHtml(test.name)}</div>
          <div class="test-duration">${test.duration}ms</div>
          ${hasFixAttempts ? `<span class="fix-badge">${fixResult.fixAttempts.length} fix${fixResult.fixAttempts.length > 1 ? 'es' : ''}</span>` : ''}
          <div class="test-expand">▼</div>
        </div>
        <div class="test-details" id="test-${index}">
          ${test.error ? `
          <div class="test-section">
            <h3>Error</h3>
            <pre class="error-message">${this.escapeHtml(test.error)}</pre>
          </div>
          ` : ''}

          ${test.diff && test.diff.length > 0 ? `
          <div class="test-section">
            <h3>Differences</h3>
            <div class="diff-container">
              ${test.diff.map(d => `
              <div class="diff-item">
                <div class="diff-path">${this.escapeHtml(d.path)}</div>
                <div class="diff-values">
                  <div class="diff-expected">
                    <div class="diff-label">Expected:</div>
                    <pre>${this.formatJson(d.expected)}</pre>
                  </div>
                  <div class="diff-actual">
                    <div class="diff-label">Actual:</div>
                    <pre>${this.formatJson(d.actual)}</pre>
                  </div>
                </div>
              </div>
              `).join('')}
            </div>
          </div>
          ` : ''}

          ${hasFixAttempts ? `
          <div class="test-section">
            <h3>Fix Attempts (${fixResult.iterations} iteration${fixResult.iterations > 1 ? 's' : ''})</h3>
            <div class="fix-attempts">
              ${fixResult.fixAttempts.map((attempt, i) => `
              <div class="fix-attempt ${attempt.success ? 'success' : 'failed'}">
                <div class="fix-attempt-header">
                  <span class="fix-attempt-number">#${i + 1}</span>
                  <span class="fix-strategy">${this.escapeHtml(attempt.strategy)}</span>
                  <span class="fix-status">${attempt.success ? '✓ Success' : '✗ Failed'}</span>
                </div>
                <div class="fix-message">${this.escapeHtml(attempt.message)}</div>
              </div>
              `).join('')}
            </div>
            <div class="fix-final-status">
              Final Status: <span class="${fixResult.finalStatus}">${fixResult.finalStatus.toUpperCase()}</span>
            </div>
          </div>
          ` : ''}
        </div>
      </div>
      `;
    }).join('');
  }

  /**
   * Get status icon
   */
  private getStatusIcon(status: string): string {
    switch (status) {
      case 'pass': return '✓';
      case 'fail': return '✗';
      case 'error': return '⚠';
      case 'timeout': return '⏱';
      default: return '?';
    }
  }

  /**
   * Escape HTML
   */
  private escapeHtml(text: string): string {
    const map: Record<string, string> = {
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#039;'
    };
    return text.replace(/[&<>"']/g, m => map[m]);
  }

  /**
   * Format JSON with syntax highlighting
   */
  private formatJson(value: unknown): string {
    try {
      const json = JSON.stringify(value, null, 2);
      return this.escapeHtml(json);
    } catch {
      return this.escapeHtml(String(value));
    }
  }

  /**
   * Get CSS styles
   */
  private getStyles(): string {
    return `
      * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
      }

      body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
        line-height: 1.6;
        color: #333;
        background: #f5f5f5;
        padding: 20px;
      }

      .container {
        max-width: 1200px;
        margin: 0 auto;
        background: white;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        overflow: hidden;
      }

      header {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        padding: 30px;
        text-align: center;
      }

      header h1 {
        font-size: 2em;
        margin-bottom: 10px;
      }

      .timestamp {
        opacity: 0.9;
        font-size: 0.9em;
      }

      .summary {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 15px;
        padding: 30px;
        background: #fafafa;
        border-bottom: 1px solid #e0e0e0;
      }

      .summary-card {
        background: white;
        padding: 20px;
        border-radius: 8px;
        text-align: center;
        box-shadow: 0 1px 3px rgba(0,0,0,0.1);
      }

      .summary-title {
        font-size: 0.85em;
        color: #666;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-bottom: 8px;
      }

      .summary-value {
        font-size: 2em;
        font-weight: bold;
        color: #333;
      }

      .summary-card.pass .summary-value { color: #28a745; }
      .summary-card.fail .summary-value { color: #dc3545; }
      .summary-card.error .summary-value { color: #ffc107; }
      .summary-card.fixed .summary-value { color: #17a2b8; }
      .summary-card.unfixed .summary-value { color: #6c757d; }

      .filters {
        padding: 20px 30px;
        background: white;
        border-bottom: 1px solid #e0e0e0;
        display: flex;
        gap: 10px;
        flex-wrap: wrap;
      }

      .filter-btn {
        padding: 8px 16px;
        border: 2px solid #e0e0e0;
        background: white;
        border-radius: 20px;
        cursor: pointer;
        font-size: 0.9em;
        transition: all 0.2s;
      }

      .filter-btn:hover {
        border-color: #667eea;
        color: #667eea;
      }

      .filter-btn.active {
        background: #667eea;
        color: white;
        border-color: #667eea;
      }

      .test-list {
        padding: 20px 30px 30px;
      }

      .test-item {
        margin-bottom: 15px;
        border: 1px solid #e0e0e0;
        border-radius: 8px;
        overflow: hidden;
        transition: box-shadow 0.2s;
      }

      .test-item:hover {
        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
      }

      .test-header {
        display: flex;
        align-items: center;
        padding: 15px 20px;
        cursor: pointer;
        background: #fafafa;
        gap: 15px;
      }

      .test-status {
        font-size: 1.2em;
        font-weight: bold;
        width: 30px;
        text-align: center;
      }

      .test-status.pass { color: #28a745; }
      .test-status.fail { color: #dc3545; }
      .test-status.error { color: #ffc107; }
      .test-status.timeout { color: #6c757d; }

      .test-name {
        flex: 1;
        font-weight: 500;
      }

      .test-duration {
        color: #666;
        font-size: 0.9em;
        font-family: monospace;
      }

      .fix-badge {
        background: #17a2b8;
        color: white;
        padding: 4px 10px;
        border-radius: 12px;
        font-size: 0.85em;
        font-weight: 500;
      }

      .test-expand {
        color: #666;
        transition: transform 0.2s;
      }

      .test-item.expanded .test-expand {
        transform: rotate(180deg);
      }

      .test-details {
        display: none;
        padding: 20px;
        background: white;
        border-top: 1px solid #e0e0e0;
      }

      .test-item.expanded .test-details {
        display: block;
      }

      .test-section {
        margin-bottom: 25px;
      }

      .test-section:last-child {
        margin-bottom: 0;
      }

      .test-section h3 {
        font-size: 1.1em;
        margin-bottom: 12px;
        color: #555;
        border-bottom: 2px solid #667eea;
        padding-bottom: 5px;
      }

      .error-message {
        background: #fff3cd;
        border: 1px solid #ffc107;
        border-radius: 4px;
        padding: 15px;
        color: #856404;
        font-family: monospace;
        font-size: 0.9em;
        white-space: pre-wrap;
        overflow-x: auto;
      }

      .diff-container {
        display: flex;
        flex-direction: column;
        gap: 20px;
      }

      .diff-item {
        border: 1px solid #e0e0e0;
        border-radius: 4px;
        overflow: hidden;
      }

      .diff-path {
        background: #f8f9fa;
        padding: 10px 15px;
        font-family: monospace;
        font-size: 0.9em;
        font-weight: 500;
        border-bottom: 1px solid #e0e0e0;
      }

      .diff-values {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1px;
        background: #e0e0e0;
      }

      .diff-expected,
      .diff-actual {
        background: white;
        padding: 15px;
      }

      .diff-label {
        font-weight: 500;
        margin-bottom: 8px;
        font-size: 0.9em;
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }

      .diff-expected .diff-label { color: #28a745; }
      .diff-actual .diff-label { color: #dc3545; }

      .diff-values pre {
        font-family: monospace;
        font-size: 0.85em;
        white-space: pre-wrap;
        overflow-x: auto;
        background: #f8f9fa;
        padding: 10px;
        border-radius: 4px;
      }

      .fix-attempts {
        display: flex;
        flex-direction: column;
        gap: 10px;
      }

      .fix-attempt {
        border: 1px solid #e0e0e0;
        border-radius: 4px;
        padding: 15px;
        background: #fafafa;
      }

      .fix-attempt.success {
        border-color: #28a745;
        background: #d4edda;
      }

      .fix-attempt.failed {
        border-color: #dc3545;
        background: #f8d7da;
      }

      .fix-attempt-header {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-bottom: 8px;
      }

      .fix-attempt-number {
        background: #667eea;
        color: white;
        padding: 2px 8px;
        border-radius: 10px;
        font-size: 0.85em;
        font-weight: 500;
      }

      .fix-strategy {
        font-family: monospace;
        font-size: 0.9em;
        flex: 1;
      }

      .fix-status {
        font-weight: 500;
        font-size: 0.9em;
      }

      .fix-message {
        font-size: 0.9em;
        color: #555;
        margin-left: 30px;
      }

      .fix-final-status {
        margin-top: 15px;
        padding-top: 15px;
        border-top: 1px solid #e0e0e0;
        font-weight: 500;
      }

      .fix-final-status .pass,
      .fix-final-status .fixed {
        color: #28a745;
      }

      .fix-final-status .fail,
      .fix-final-status .unfixed {
        color: #dc3545;
      }

      @media (max-width: 768px) {
        .summary {
          grid-template-columns: repeat(2, 1fr);
        }

        .diff-values {
          grid-template-columns: 1fr;
        }
      }
    `;
  }

  /**
   * Get JavaScript for interactivity
   */
  private getScript(data: ReportData): string {
    return `
      // Embedded test data
      const testData = ${JSON.stringify(data, null, 2)};

      // Toggle test details
      function toggleTest(index) {
        const item = document.querySelectorAll('.test-item')[index];
        item.classList.toggle('expanded');
      }

      // Filter tests
      document.querySelectorAll('.filter-btn').forEach(btn => {
        btn.addEventListener('click', function() {
          // Update active button
          document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
          this.classList.add('active');

          const filter = this.dataset.filter;
          const tests = document.querySelectorAll('.test-item');

          tests.forEach(test => {
            const status = test.dataset.status;
            const fixed = test.dataset.fixed === 'true';

            if (filter === 'all') {
              test.style.display = 'block';
            } else if (filter === 'fixed') {
              test.style.display = fixed ? 'block' : 'none';
            } else {
              test.style.display = status === filter ? 'block' : 'none';
            }
          });
        });
      });
    `;
  }
}

/**
 * Generate HTML report from JSON test results file
 */
export async function generateReportFromJson(
  jsonPath: string,
  outputPath: string
): Promise<void> {
  const data = JSON.parse(fs.readFileSync(jsonPath, 'utf-8')) as ReportData;
  const reporter = new HtmlReporter();
  reporter.generate(data, outputPath);
}

// CLI support
if (require.main === module) {
  const args = process.argv.slice(2);
  if (args.length < 2) {
    console.error('Usage: tsx html-reporter.ts <input.json> <output.html>');
    process.exit(1);
  }

  generateReportFromJson(args[0], args[1])
    .then(() => console.log('Report generated successfully'))
    .catch(err => {
      console.error('Failed to generate report:', err);
      process.exit(1);
    });
}
