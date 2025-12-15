#!/usr/bin/env bash
set -euo pipefail

# analyze_ffi_history.sh - Analyze implementation logs for FFI development metrics
# Calculates velocity, churn, coverage, and AI effectiveness from implementation logs

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SPEC_DIR="$PROJECT_ROOT/.spec-workflow/specs/rust-first-ai-native-ffi"
LOGS_DIR="$SPEC_DIR/Implementation Logs"
ANALYSIS_DIR="$PROJECT_ROOT/.spec-workflow/analysis"
OUTPUT_JSON="$ANALYSIS_DIR/implementation-analysis.json"
OUTPUT_HTML="$ANALYSIS_DIR/implementation-insights.html"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Ensure analysis directory exists
mkdir -p "$ANALYSIS_DIR"

echo -e "${BLUE}Analyzing FFI Implementation Logs${NC}"
echo "=================================================="
echo ""

# Check if logs directory exists
if [ ! -d "$LOGS_DIR" ]; then
    echo -e "${RED}Error: Logs directory not found: $LOGS_DIR${NC}"
    exit 1
fi

# Count total logs
TOTAL_LOGS=$(find "$LOGS_DIR" -name "task-*.md" | wc -l)
echo -e "${GREEN}Found $TOTAL_LOGS implementation logs${NC}"
echo ""

# Initialize metrics
declare -A function_modifications
declare -A domain_activity
declare -A weekly_velocity
declare -A task_coverage
total_functions_added=0
total_functions_modified=0
total_api_endpoints=0
total_components=0
total_classes=0
ai_generated_count=0
ai_success_count=0
total_lines_added=0
total_lines_removed=0

# Parse each log file
echo "Parsing implementation logs..."
for log_file in "$LOGS_DIR"/task-*.md; do
    if [ ! -f "$log_file" ]; then
        continue
    fi

    # Extract task ID
    task_id=$(basename "$log_file" | sed 's/task-\([0-9]*\)_.*/\1/')

    # Extract timestamp (ISO format)
    timestamp=$(grep "^**Timestamp:**" "$log_file" | sed 's/\*\*Timestamp:\*\* //' | tr -d '\n\r')

    # Calculate week from timestamp
    if [ -n "$timestamp" ]; then
        week=$(date -d "$timestamp" "+%Y-W%U" 2>/dev/null || date -jf "%Y-%m-%dT%H:%M:%S" "$timestamp" "+%Y-W%U" 2>/dev/null || echo "unknown")
    else
        week="unknown"
    fi

    # Extract statistics
    lines_added=$(grep "^- \*\*Lines Added:\*\*" "$log_file" | sed 's/.*+\([0-9]*\)/\1/' | tr -d '\n\r')
    lines_removed=$(grep "^- \*\*Lines Removed:\*\*" "$log_file" | sed 's/.*-\([0-9]*\)/\1/' | tr -d '\n\r')

    # Update totals
    if [ -n "$lines_added" ]; then
        total_lines_added=$((total_lines_added + lines_added))
    fi
    if [ -n "$lines_removed" ]; then
        total_lines_removed=$((total_lines_removed + lines_removed))
    fi

    # Count artifacts
    api_count=0
    component_count=0
    class_count=0
    function_count=0

    # Count API endpoints (under ### API Endpoints section)
    if grep -q "^### API Endpoints" "$log_file"; then
        api_count=$(sed -n '/^### API Endpoints$/,/^## /p' "$log_file" | grep -c "^#### " 2>/dev/null || echo 0)
    fi

    # Count components (items under ### Components section)
    if grep -q "^### Components" "$log_file"; then
        component_count=$(sed -n '/^### Components$/,/^## /p' "$log_file" | grep -c "^#### " 2>/dev/null || echo 0)
    fi

    # Count classes (items under ### Classes section)
    if grep -q "^### Classes" "$log_file"; then
        class_count=$(sed -n '/^### Classes$/,/^## /p' "$log_file" | grep -c "^#### " 2>/dev/null || echo 0)
    fi

    # Count functions (items under ### Functions section)
    if grep -q "^### Functions" "$log_file"; then
        function_count=$(sed -n '/^### Functions$/,/^## /p' "$log_file" | grep -c "^#### " 2>/dev/null || echo 0)
    fi

    # Total artifacts for this log
    total_artifacts=$((api_count + component_count + class_count + function_count))

    # Update artifact totals
    total_api_endpoints=$((total_api_endpoints + api_count))
    total_components=$((total_components + component_count))
    total_classes=$((total_classes + class_count))
    total_functions_added=$((total_functions_added + function_count))

    # Update weekly velocity (use total artifacts as proxy for work done)
    if [ "$week" != "unknown" ] && [ $total_artifacts -gt 0 ]; then
        weekly_velocity[$week]=$((${weekly_velocity[$week]:-0} + total_artifacts))
    fi

    # Extract domains from artifacts
    while IFS= read -r domain; do
        if [ -n "$domain" ]; then
            domain_activity[$domain]=$((${domain_activity[$domain]:-0} + 1))
        fi
    done < <(grep "domain = " "$log_file" | sed 's/.*domain = "\([^"]*\)".*/\1/' || true)

    # Check for AI-generated marker (look for common patterns)
    if grep -qi "ai.*generated\|claude\|assistant" "$log_file" 2>/dev/null; then
        ai_generated_count=$((ai_generated_count + 1))
        # Check for success indicators (tests pass, no errors mentioned)
        if grep -qi "tests pass\|success\|complete" "$log_file" && ! grep -qi "error\|fail\|issue" "$log_file"; then
            ai_success_count=$((ai_success_count + 1))
        fi
    fi

    # Track test coverage
    if grep -qi "test\|coverage" "$log_file"; then
        task_coverage[$task_id]="tested"
    else
        task_coverage[$task_id]="untested"
    fi
done

# Calculate derived metrics
tested_tasks=0
untested_tasks=0
for status in "${task_coverage[@]}"; do
    if [ "$status" = "tested" ]; then
        tested_tasks=$((tested_tasks + 1))
    else
        untested_tasks=$((untested_tasks + 1))
    fi
done
total_tasks=$((tested_tasks + untested_tasks))
coverage_percent=0
if [ $total_tasks -gt 0 ]; then
    coverage_percent=$(echo "scale=1; ($tested_tasks * 100) / $total_tasks" | bc)
fi

ai_effectiveness=0
if [ $ai_generated_count -gt 0 ]; then
    ai_effectiveness=$(echo "scale=1; ($ai_success_count * 100) / $ai_generated_count" | bc)
fi

# Calculate velocity metrics (functions per week)
sorted_weeks=($(printf '%s\n' "${!weekly_velocity[@]}" | sort))
avg_velocity=0
if [ ${#sorted_weeks[@]} -gt 0 ]; then
    total_velocity=0
    for week in "${sorted_weeks[@]}"; do
        total_velocity=$((total_velocity + weekly_velocity[$week]))
    done
    avg_velocity=$(echo "scale=2; $total_velocity / ${#sorted_weeks[@]}" | bc)
fi

# Identify churn (files modified in multiple logs)
declare -A file_modification_count
for log_file in "$LOGS_DIR"/task-*.md; do
    if [ ! -f "$log_file" ]; then
        continue
    fi

    # Extract modified files
    while IFS= read -r file; do
        if [ -n "$file" ]; then
            # Clean up the filename
            clean_file=$(echo "$file" | sed 's/^- //' | tr -d '\n\r')
            file_modification_count[$clean_file]=$((${file_modification_count[$clean_file]:-0} + 1))
        fi
    done < <(sed -n '/## Files Modified/,/## Files Created/p' "$log_file" | grep "^- " | grep -v "^## " || true)
done

# Find high-churn files (modified 3+ times)
high_churn_files=()
for file in "${!file_modification_count[@]}"; do
    count=${file_modification_count[$file]}
    if [ $count -ge 3 ]; then
        high_churn_files+=("$file:$count")
    fi
done

# Sort high-churn files by modification count (descending)
IFS=$'\n' sorted_churn=($(sort -t: -k2 -rn <<<"${high_churn_files[*]}"))
unset IFS

# Generate JSON report
echo "Generating JSON report..."
cat > "$OUTPUT_JSON" << EOF
{
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "summary": {
    "total_logs": $TOTAL_LOGS,
    "total_tasks_analyzed": $total_tasks,
    "total_lines_added": $total_lines_added,
    "total_lines_removed": $total_lines_removed,
    "net_change": $((total_lines_added - total_lines_removed))
  },
  "velocity": {
    "total_functions_added": $total_functions_added,
    "total_api_endpoints": $total_api_endpoints,
    "total_components": $total_components,
    "total_classes": $total_classes,
    "avg_functions_per_week": $avg_velocity,
    "weeks_analyzed": ${#sorted_weeks[@]},
    "by_week": {
EOF

# Add weekly data
first=true
for week in "${sorted_weeks[@]}"; do
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> "$OUTPUT_JSON"
    fi
    echo -n "      \"$week\": ${weekly_velocity[$week]}" >> "$OUTPUT_JSON"
done

cat >> "$OUTPUT_JSON" << EOF

    }
  },
  "churn": {
    "high_churn_files_count": ${#high_churn_files[@]},
    "files": [
EOF

# Add churn data
first=true
for entry in "${sorted_churn[@]}"; do
    if [ -n "$entry" ]; then
        file=$(echo "$entry" | cut -d: -f1)
        count=$(echo "$entry" | cut -d: -f2)
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$OUTPUT_JSON"
        fi
        echo -n "      {\"file\": \"$file\", \"modifications\": $count}" >> "$OUTPUT_JSON"
    fi
done

cat >> "$OUTPUT_JSON" << EOF

    ]
  },
  "coverage": {
    "tested_tasks": $tested_tasks,
    "untested_tasks": $untested_tasks,
    "total_tasks": $total_tasks,
    "coverage_percent": $coverage_percent
  },
  "ai_effectiveness": {
    "ai_generated_logs": $ai_generated_count,
    "ai_successful_logs": $ai_success_count,
    "effectiveness_percent": $ai_effectiveness,
    "note": "Based on log content analysis for AI-related keywords and success indicators"
  },
  "domain_activity": {
EOF

# Add domain activity
first=true
for domain in "${!domain_activity[@]}"; do
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> "$OUTPUT_JSON"
    fi
    echo -n "    \"$domain\": ${domain_activity[$domain]}" >> "$OUTPUT_JSON"
done

cat >> "$OUTPUT_JSON" << EOF

  }
}
EOF

echo -e "${GREEN}✓ JSON report generated: $OUTPUT_JSON${NC}"

# Generate HTML visualization
echo "Generating HTML visualization..."
cat > "$OUTPUT_HTML" << 'HTMLEOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FFI Implementation History Analysis</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 20px;
            color: #333;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
        }
        h1 {
            color: white;
            text-align: center;
            margin-bottom: 10px;
            font-size: 2.5em;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        .subtitle {
            color: rgba(255,255,255,0.9);
            text-align: center;
            margin-bottom: 30px;
            font-size: 1.1em;
        }
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .metric-card {
            background: white;
            padding: 25px;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .metric-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 8px 12px rgba(0,0,0,0.15);
        }
        .metric-label {
            font-size: 0.9em;
            color: #666;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 8px;
        }
        .metric-value {
            font-size: 2.5em;
            font-weight: bold;
            color: #667eea;
        }
        .metric-subtitle {
            font-size: 0.85em;
            color: #888;
            margin-top: 5px;
        }
        .chart-container {
            background: white;
            padding: 30px;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            margin-bottom: 30px;
        }
        .chart-title {
            font-size: 1.5em;
            margin-bottom: 20px;
            color: #333;
            font-weight: 600;
        }
        .chart-wrapper {
            position: relative;
            height: 400px;
        }
        .insights {
            background: white;
            padding: 30px;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            margin-bottom: 30px;
        }
        .insight-item {
            padding: 15px;
            margin-bottom: 15px;
            border-left: 4px solid #667eea;
            background: #f8f9fa;
            border-radius: 4px;
        }
        .insight-title {
            font-weight: 600;
            margin-bottom: 5px;
            color: #667eea;
        }
        .churn-list {
            list-style: none;
            padding: 0;
        }
        .churn-item {
            padding: 10px;
            margin-bottom: 8px;
            background: #f8f9fa;
            border-radius: 4px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .churn-file {
            font-family: monospace;
            font-size: 0.9em;
            flex: 1;
        }
        .churn-count {
            background: #667eea;
            color: white;
            padding: 4px 12px;
            border-radius: 12px;
            font-weight: bold;
            font-size: 0.9em;
        }
        .status-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 0.85em;
            font-weight: 600;
        }
        .status-good { background: #d4edda; color: #155724; }
        .status-warning { background: #fff3cd; color: #856404; }
        .status-danger { background: #f8d7da; color: #721c24; }
    </style>
</head>
<body>
    <div class="container">
        <h1>FFI Implementation History Analysis</h1>
        <div class="subtitle" id="generated-at"></div>

        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-label">Total Tasks</div>
                <div class="metric-value" id="total-tasks">-</div>
                <div class="metric-subtitle" id="total-logs">- implementation logs</div>
            </div>

            <div class="metric-card">
                <div class="metric-label">Functions Added</div>
                <div class="metric-value" id="total-functions">-</div>
                <div class="metric-subtitle">
                    <span id="api-endpoints">-</span> APIs •
                    <span id="components">-</span> Components •
                    <span id="classes">-</span> Classes
                </div>
            </div>

            <div class="metric-card">
                <div class="metric-label">Code Changes</div>
                <div class="metric-value" id="net-change">-</div>
                <div class="metric-subtitle">
                    +<span id="lines-added">-</span> /
                    -<span id="lines-removed">-</span> lines
                </div>
            </div>

            <div class="metric-card">
                <div class="metric-label">Avg Velocity</div>
                <div class="metric-value" id="avg-velocity">-</div>
                <div class="metric-subtitle">functions/week</div>
            </div>

            <div class="metric-card">
                <div class="metric-label">Test Coverage</div>
                <div class="metric-value" id="coverage-percent">-</div>
                <div class="metric-subtitle">
                    <span id="tested-tasks">-</span> tested /
                    <span id="untested-tasks">-</span> untested
                </div>
            </div>

            <div class="metric-card">
                <div class="metric-label">AI Effectiveness</div>
                <div class="metric-value" id="ai-effectiveness">-</div>
                <div class="metric-subtitle">
                    <span id="ai-success">-</span> /
                    <span id="ai-total">-</span> successful
                </div>
            </div>
        </div>

        <div class="chart-container">
            <div class="chart-title">Development Velocity (Functions/Week)</div>
            <div class="chart-wrapper">
                <canvas id="velocityChart"></canvas>
            </div>
        </div>

        <div class="chart-container">
            <div class="chart-title">Domain Activity Distribution</div>
            <div class="chart-wrapper">
                <canvas id="domainChart"></canvas>
            </div>
        </div>

        <div class="insights">
            <div class="chart-title">High-Churn Files (Modified 3+ Times)</div>
            <ul class="churn-list" id="churn-list">
                <li style="text-align: center; color: #888;">No high-churn files detected</li>
            </ul>
        </div>

        <div class="insights">
            <div class="chart-title">Key Insights</div>
            <div id="insights-container"></div>
        </div>
    </div>

    <script>
        fetch('implementation-analysis.json')
            .then(response => response.json())
            .then(data => {
                // Update summary metrics
                document.getElementById('generated-at').textContent =
                    'Generated: ' + new Date(data.generated_at).toLocaleString();
                document.getElementById('total-tasks').textContent =
                    data.summary.total_tasks_analyzed;
                document.getElementById('total-logs').textContent =
                    data.summary.total_logs + ' implementation logs';
                document.getElementById('total-functions').textContent =
                    data.velocity.total_functions_added;
                document.getElementById('api-endpoints').textContent =
                    data.velocity.total_api_endpoints;
                document.getElementById('components').textContent =
                    data.velocity.total_components;
                document.getElementById('classes').textContent =
                    data.velocity.total_classes;
                document.getElementById('net-change').textContent =
                    (data.summary.net_change >= 0 ? '+' : '') + data.summary.net_change;
                document.getElementById('lines-added').textContent =
                    data.summary.total_lines_added.toLocaleString();
                document.getElementById('lines-removed').textContent =
                    data.summary.total_lines_removed.toLocaleString();
                document.getElementById('avg-velocity').textContent =
                    parseFloat(data.velocity.avg_functions_per_week).toFixed(1);
                document.getElementById('coverage-percent').textContent =
                    parseFloat(data.coverage.coverage_percent).toFixed(0) + '%';
                document.getElementById('tested-tasks').textContent =
                    data.coverage.tested_tasks;
                document.getElementById('untested-tasks').textContent =
                    data.coverage.untested_tasks;
                document.getElementById('ai-effectiveness').textContent =
                    parseFloat(data.ai_effectiveness.effectiveness_percent).toFixed(0) + '%';
                document.getElementById('ai-success').textContent =
                    data.ai_effectiveness.ai_successful_logs;
                document.getElementById('ai-total').textContent =
                    data.ai_effectiveness.ai_generated_logs;

                // Velocity chart
                const weeks = Object.keys(data.velocity.by_week).sort();
                const velocityData = weeks.map(week => data.velocity.by_week[week]);

                new Chart(document.getElementById('velocityChart'), {
                    type: 'line',
                    data: {
                        labels: weeks,
                        datasets: [{
                            label: 'Functions Added',
                            data: velocityData,
                            borderColor: '#667eea',
                            backgroundColor: 'rgba(102, 126, 234, 0.1)',
                            borderWidth: 3,
                            fill: true,
                            tension: 0.4
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        plugins: {
                            legend: { display: false }
                        },
                        scales: {
                            y: {
                                beginAtZero: true,
                                ticks: { precision: 0 }
                            }
                        }
                    }
                });

                // Domain activity chart
                const domains = Object.keys(data.domain_activity);
                const domainCounts = domains.map(d => data.domain_activity[d]);

                if (domains.length > 0) {
                    new Chart(document.getElementById('domainChart'), {
                        type: 'bar',
                        data: {
                            labels: domains,
                            datasets: [{
                                label: 'Tasks',
                                data: domainCounts,
                                backgroundColor: '#667eea',
                                borderRadius: 8
                            }]
                        },
                        options: {
                            responsive: true,
                            maintainAspectRatio: false,
                            plugins: {
                                legend: { display: false }
                            },
                            scales: {
                                y: {
                                    beginAtZero: true,
                                    ticks: { precision: 0 }
                                }
                            }
                        }
                    });
                }

                // Churn list
                if (data.churn.files.length > 0) {
                    const churnList = document.getElementById('churn-list');
                    churnList.innerHTML = '';
                    data.churn.files.forEach(item => {
                        const li = document.createElement('li');
                        li.className = 'churn-item';
                        li.innerHTML = `
                            <span class="churn-file">${item.file}</span>
                            <span class="churn-count">${item.modifications}x</span>
                        `;
                        churnList.appendChild(li);
                    });
                }

                // Generate insights
                const insights = document.getElementById('insights-container');

                // Velocity insight
                const avgVel = parseFloat(data.velocity.avg_functions_per_week);
                const velStatus = avgVel > 5 ? 'status-good' : avgVel > 2 ? 'status-warning' : 'status-danger';
                insights.innerHTML += `
                    <div class="insight-item">
                        <div class="insight-title">Development Velocity <span class="${velStatus} status-badge">${avgVel.toFixed(1)}/week</span></div>
                        <div>${avgVel > 5 ? 'Excellent' : avgVel > 2 ? 'Good' : 'Low'} average velocity. ${
                            avgVel > 5 ? 'Team is highly productive.' :
                            avgVel > 2 ? 'Steady progress on FFI development.' :
                            'Consider reviewing blockers or resource allocation.'
                        }</div>
                    </div>
                `;

                // Coverage insight
                const coverage = parseFloat(data.coverage.coverage_percent);
                const covStatus = coverage >= 80 ? 'status-good' : coverage >= 50 ? 'status-warning' : 'status-danger';
                insights.innerHTML += `
                    <div class="insight-item">
                        <div class="insight-title">Test Coverage <span class="${covStatus} status-badge">${coverage.toFixed(0)}%</span></div>
                        <div>${coverage >= 80 ? 'Excellent' : coverage >= 50 ? 'Fair' : 'Low'} test coverage. ${
                            coverage >= 80 ? 'Well-tested implementation.' :
                            coverage >= 50 ? 'Consider adding more tests for untested tasks.' :
                            'Significant test coverage gaps detected. Add tests for ' + data.coverage.untested_tasks + ' tasks.'
                        }</div>
                    </div>
                `;

                // AI effectiveness insight
                const aiEff = parseFloat(data.ai_effectiveness.effectiveness_percent);
                const aiStatus = aiEff >= 75 ? 'status-good' : aiEff >= 50 ? 'status-warning' : 'status-danger';
                insights.innerHTML += `
                    <div class="insight-item">
                        <div class="insight-title">AI Effectiveness <span class="${aiStatus} status-badge">${aiEff.toFixed(0)}%</span></div>
                        <div>${aiEff >= 75 ? 'High' : aiEff >= 50 ? 'Moderate' : 'Low'} AI-generated code success rate. ${
                            aiEff >= 75 ? 'AI agents are highly effective.' :
                            aiEff >= 50 ? 'AI agents show moderate effectiveness.' :
                            'Review AI agent directives and examples to improve effectiveness.'
                        }</div>
                    </div>
                `;

                // Churn insight
                if (data.churn.high_churn_files_count > 0) {
                    insights.innerHTML += `
                        <div class="insight-item">
                            <div class="insight-title">Code Churn <span class="status-warning status-badge">${data.churn.high_churn_files_count} files</span></div>
                            <div>${data.churn.high_churn_files_count} files modified 3+ times. High churn may indicate design instability or refactoring opportunities.</div>
                        </div>
                    `;
                } else {
                    insights.innerHTML += `
                        <div class="insight-item">
                            <div class="insight-title">Code Churn <span class="status-good status-badge">Low</span></div>
                            <div>No high-churn files detected. Code appears stable with minimal rework.</div>
                        </div>
                    `;
                }
            })
            .catch(error => {
                console.error('Error loading analysis data:', error);
                document.querySelector('.container').innerHTML +=
                    '<div style="color: red; text-align: center; padding: 40px;">Error loading analysis data. Ensure implementation-analysis.json exists.</div>';
            });
    </script>
</body>
</html>
HTMLEOF

echo -e "${GREEN}✓ HTML visualization generated: $OUTPUT_HTML${NC}"
echo ""

# Print summary
echo "=================================================="
echo -e "${BLUE}Analysis Summary${NC}"
echo "=================================================="
echo -e "Total Logs:              ${GREEN}$TOTAL_LOGS${NC}"
echo -e "Functions Added:         ${GREEN}$total_functions_added${NC}"
echo -e "API Endpoints:           ${GREEN}$total_api_endpoints${NC}"
echo -e "Components:              ${GREEN}$total_components${NC}"
echo -e "Classes:                 ${GREEN}$total_classes${NC}"
echo -e "Avg Velocity:            ${GREEN}$avg_velocity functions/week${NC}"
echo -e "Test Coverage:           ${GREEN}$coverage_percent%${NC} ($tested_tasks tested, $untested_tasks untested)"
echo -e "AI Effectiveness:        ${GREEN}$ai_effectiveness%${NC} ($ai_success_count/$ai_generated_count successful)"
echo -e "High-Churn Files:        ${YELLOW}${#high_churn_files[@]}${NC}"
echo ""
echo -e "${GREEN}✓ Analysis complete!${NC}"
echo ""
echo "View results:"
echo "  - JSON: $OUTPUT_JSON"
echo "  - HTML: $OUTPUT_HTML"
echo ""
echo "Open dashboard:"
echo "  xdg-open $OUTPUT_HTML"
echo "  # or"
echo "  open $OUTPUT_HTML"
