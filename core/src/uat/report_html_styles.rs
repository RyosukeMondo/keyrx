//! CSS styles for HTML report generation.
//!
//! This module contains the embedded CSS used in standalone HTML reports.

/// Embedded CSS styles for HTML reports.
///
/// These styles provide a clean, modern look for viewing reports in web browsers.
/// The CSS uses CSS custom properties for easy theming.
pub const REPORT_CSS: &str = r#":root {
  --color-pass: #22c55e;
  --color-fail: #ef4444;
  --color-skip: #f59e0b;
  --color-bg: #f8fafc;
  --color-card: #ffffff;
  --color-border: #e2e8f0;
  --color-text: #1e293b;
  --color-text-muted: #64748b;
}
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: var(--color-bg);
  color: var(--color-text);
  line-height: 1.6;
  padding: 2rem;
}
.container { max-width: 1200px; margin: 0 auto; }
header { margin-bottom: 2rem; }
h1 { font-size: 2rem; font-weight: 700; }
h2 { font-size: 1.5rem; font-weight: 600; margin: 1.5rem 0 1rem; }
h3 { font-size: 1.25rem; font-weight: 600; margin: 1rem 0 0.5rem; }
.timestamp { color: var(--color-text-muted); font-size: 0.875rem; }
.card {
  background: var(--color-card);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}
.summary-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1rem;
}
.stat {
  text-align: center;
  padding: 1rem;
  border-radius: 0.5rem;
  background: var(--color-bg);
}
.stat-value { font-size: 2rem; font-weight: 700; }
.stat-label { font-size: 0.875rem; color: var(--color-text-muted); }
.stat.pass { border-left: 4px solid var(--color-pass); }
.stat.fail { border-left: 4px solid var(--color-fail); }
.stat.skip { border-left: 4px solid var(--color-skip); }
.stat.total { border-left: 4px solid #3b82f6; }
.badge {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  font-weight: 600;
}
.badge-pass { background: #dcfce7; color: #166534; }
.badge-fail { background: #fee2e2; color: #991b1b; }
.badge-skip { background: #fef3c7; color: #92400e; }
.badge-p0 { background: #fee2e2; color: #991b1b; }
.badge-p1 { background: #fef3c7; color: #92400e; }
.badge-p2 { background: #e0e7ff; color: #3730a3; }
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}
th, td {
  padding: 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--color-border);
}
th { font-weight: 600; background: var(--color-bg); }
tr:hover { background: var(--color-bg); }
.progress-bar {
  height: 8px;
  background: var(--color-border);
  border-radius: 4px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: var(--color-pass);
  transition: width 0.3s;
}
.gate-pass { color: var(--color-pass); }
.gate-fail { color: var(--color-fail); }
.violation { background: #fee2e2; padding: 0.5rem; border-radius: 0.25rem; margin: 0.25rem 0; }
.coverage-verified { color: var(--color-pass); }
.coverage-atrisk { color: var(--color-fail); }
.coverage-uncovered { color: var(--color-text-muted); }
.perf-metric { display: inline-block; margin-right: 1.5rem; }
.perf-value { font-size: 1.5rem; font-weight: 600; }
.perf-label { font-size: 0.75rem; color: var(--color-text-muted); }
.error-msg { color: var(--color-fail); font-family: monospace; font-size: 0.8rem; }
"#;
