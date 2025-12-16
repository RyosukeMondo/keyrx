#!/usr/bin/env bash
# Generate FFI Contract Metrics Dashboard
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACT_DIR="$PROJECT_ROOT/core/src/ffi/contracts"
OUTPUT_DIR="$PROJECT_ROOT/.spec-workflow/analysis"
OUTPUT_JSON="$OUTPUT_DIR/contract-metrics.json"
OUTPUT_HTML="$OUTPUT_DIR/contract-dashboard.html"

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

echo "Analyzing FFI contracts..."

# Initialize metrics
total_functions=0
functions_by_domain=()
declare -A domain_counts
total_params=0
function_count=0
result_count=0
non_result_count=0
domains=""

# Parse contract files
for contract_file in "$CONTRACT_DIR"/*.ffi-contract.json; do
    if [[ ! -f "$contract_file" ]]; then
        continue
    fi

    # Extract domain
    domain=$(jq -r '.domain' "$contract_file")

    # Count functions in this domain
    func_count=$(jq '.functions | length' "$contract_file")
    total_functions=$((total_functions + func_count))
    domain_counts["$domain"]=$func_count

    # Count parameters
    params=$(jq '[.functions[].parameters | length] | add // 0' "$contract_file")
    total_params=$((total_params + params))
    function_count=$((function_count + func_count))

    # Count Result vs non-Result return types
    result_funcs=$(jq '[.functions[].returns.type | select(contains("Result") or . == "object")] | length' "$contract_file" || echo 0)
    result_count=$((result_count + result_funcs))
    non_result_count=$((non_result_count + func_count - result_funcs))
done

# Calculate average params per function
avg_params=0
if [[ $function_count -gt 0 ]]; then
    avg_params=$(echo "scale=2; $total_params / $function_count" | bc)
fi

# Build functions by domain array for JSON
functions_by_domain_json="["
first=true
for domain in "${!domain_counts[@]}"; do
    if [[ "$first" == "true" ]]; then
        first=false
    else
        functions_by_domain_json+=","
    fi
    functions_by_domain_json+="{\"domain\":\"$domain\",\"count\":${domain_counts[$domain]}}"
done
functions_by_domain_json+="]"

# Analyze git history for evolution metrics
echo "Analyzing git history..."

# Get commits in the last 12 weeks that touched FFI contracts
weeks_data="["
for week in {0..11}; do
    since_date=$(date -d "$((week * 7)) days ago" +%Y-%m-%d)
    until_date=$(date -d "$(((week - 1) * 7)) days ago" +%Y-%m-%d)

    # Count functions added (new files or additions to files)
    added=$(git log --since="$since_date" --until="$until_date" --pretty=format: --numstat -- "$CONTRACT_DIR/*.json" 2>/dev/null | \
        awk '{added+=$1} END {print added+0}')

    # Count functions removed
    removed=$(git log --since="$since_date" --until="$until_date" --pretty=format: --numstat -- "$CONTRACT_DIR/*.json" 2>/dev/null | \
        awk '{removed+=$2} END {print removed+0}')

    # Rough estimate: each function is about 15 lines
    funcs_added=$((added / 15))
    funcs_removed=$((removed / 15))

    if [[ $week -lt 11 ]]; then
        weeks_data+="{\"week\":\"$since_date\",\"added\":$funcs_added,\"removed\":$funcs_removed},"
    else
        weeks_data+="{\"week\":\"$since_date\",\"added\":$funcs_added,\"removed\":$funcs_removed}"
    fi
done
weeks_data+="]"

# Count breaking changes in git history (commits mentioning "breaking")
breaking_changes=$(git log --all --grep="breaking" --oneline -- "$CONTRACT_DIR/*.json" 2>/dev/null | wc -l)

# Generate JSON metrics
cat > "$OUTPUT_JSON" <<EOF
{
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "summary": {
    "total_functions": $total_functions,
    "total_domains": ${#domain_counts[@]},
    "avg_params_per_function": $avg_params,
    "result_return_type_count": $result_count,
    "non_result_return_type_count": $non_result_count,
    "breaking_changes_count": $breaking_changes
  },
  "functions_by_domain": $functions_by_domain_json,
  "evolution": $weeks_data
}
EOF

echo "Metrics written to $OUTPUT_JSON"

# Generate HTML dashboard
echo "Generating HTML dashboard..."

cat > "$OUTPUT_HTML" <<'HTMLEOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FFI Contract Metrics Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            min-height: 100vh;
            padding: 20px;
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
            margin-bottom: 40px;
            font-size: 1.1em;
        }
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }
        .metric-card {
            background: white;
            border-radius: 12px;
            padding: 25px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .metric-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 15px 40px rgba(0,0,0,0.3);
        }
        .metric-value {
            font-size: 3em;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 10px;
        }
        .metric-label {
            font-size: 0.95em;
            color: #666;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        .chart-container {
            background: white;
            border-radius: 12px;
            padding: 30px;
            margin-bottom: 30px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
        }
        .chart-title {
            font-size: 1.5em;
            margin-bottom: 20px;
            color: #333;
            font-weight: 600;
        }
        canvas {
            max-height: 400px;
        }
        .footer {
            text-align: center;
            color: rgba(255,255,255,0.8);
            margin-top: 40px;
            font-size: 0.9em;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>FFI Contract Metrics Dashboard</h1>
        <div class="subtitle" id="generated-time">Generated at: Loading...</div>

        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-value" id="total-functions">0</div>
                <div class="metric-label">Total FFI Functions</div>
            </div>
            <div class="metric-card">
                <div class="metric-value" id="total-domains">0</div>
                <div class="metric-label">Domains</div>
            </div>
            <div class="metric-card">
                <div class="metric-value" id="avg-params">0</div>
                <div class="metric-label">Avg Params/Function</div>
            </div>
            <div class="metric-card">
                <div class="metric-value" id="breaking-changes">0</div>
                <div class="metric-label">Breaking Changes</div>
            </div>
        </div>

        <div class="chart-container">
            <div class="chart-title">Functions by Domain</div>
            <canvas id="domainChart"></canvas>
        </div>

        <div class="chart-container">
            <div class="chart-title">Evolution Timeline (12 Weeks)</div>
            <canvas id="evolutionChart"></canvas>
        </div>

        <div class="chart-container">
            <div class="chart-title">Return Type Distribution</div>
            <canvas id="returnTypeChart"></canvas>
        </div>

        <div class="footer">
            KeyRx FFI Contract Metrics • Auto-generated from contract analysis
        </div>
    </div>

    <script>
        // Load metrics data
        fetch('contract-metrics.json')
            .then(response => response.json())
            .then(data => {
                // Update summary metrics
                document.getElementById('generated-time').textContent =
                    `Generated at: ${new Date(data.generated_at).toLocaleString()}`;
                document.getElementById('total-functions').textContent = data.summary.total_functions;
                document.getElementById('total-domains').textContent = data.summary.total_domains;
                document.getElementById('avg-params').textContent = data.summary.avg_params_per_function;
                document.getElementById('breaking-changes').textContent = data.summary.breaking_changes_count;

                // Functions by Domain Chart (Bar)
                const domainLabels = data.functions_by_domain.map(d => d.domain);
                const domainCounts = data.functions_by_domain.map(d => d.count);

                new Chart(document.getElementById('domainChart'), {
                    type: 'bar',
                    data: {
                        labels: domainLabels,
                        datasets: [{
                            label: 'Functions',
                            data: domainCounts,
                            backgroundColor: 'rgba(102, 126, 234, 0.8)',
                            borderColor: 'rgba(102, 126, 234, 1)',
                            borderWidth: 2,
                            borderRadius: 8
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: true,
                        plugins: {
                            legend: { display: false },
                            tooltip: {
                                backgroundColor: 'rgba(0,0,0,0.8)',
                                padding: 12,
                                titleFont: { size: 14 },
                                bodyFont: { size: 13 }
                            }
                        },
                        scales: {
                            y: {
                                beginAtZero: true,
                                ticks: {
                                    precision: 0,
                                    font: { size: 12 }
                                },
                                grid: { color: 'rgba(0,0,0,0.05)' }
                            },
                            x: {
                                ticks: { font: { size: 12 } },
                                grid: { display: false }
                            }
                        }
                    }
                });

                // Evolution Timeline Chart (Line)
                const evolutionLabels = data.evolution.map(e => e.week);
                const addedData = data.evolution.map(e => e.added);
                const removedData = data.evolution.map(e => e.removed);

                new Chart(document.getElementById('evolutionChart'), {
                    type: 'line',
                    data: {
                        labels: evolutionLabels,
                        datasets: [
                            {
                                label: 'Functions Added',
                                data: addedData,
                                borderColor: 'rgba(75, 192, 192, 1)',
                                backgroundColor: 'rgba(75, 192, 192, 0.2)',
                                borderWidth: 3,
                                fill: true,
                                tension: 0.4
                            },
                            {
                                label: 'Functions Removed',
                                data: removedData,
                                borderColor: 'rgba(255, 99, 132, 1)',
                                backgroundColor: 'rgba(255, 99, 132, 0.2)',
                                borderWidth: 3,
                                fill: true,
                                tension: 0.4
                            }
                        ]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: true,
                        plugins: {
                            legend: {
                                display: true,
                                position: 'top',
                                labels: { font: { size: 13 } }
                            },
                            tooltip: {
                                backgroundColor: 'rgba(0,0,0,0.8)',
                                padding: 12
                            }
                        },
                        scales: {
                            y: {
                                beginAtZero: true,
                                ticks: {
                                    precision: 0,
                                    font: { size: 12 }
                                },
                                grid: { color: 'rgba(0,0,0,0.05)' }
                            },
                            x: {
                                ticks: {
                                    font: { size: 11 },
                                    maxRotation: 45,
                                    minRotation: 45
                                },
                                grid: { display: false }
                            }
                        }
                    }
                });

                // Return Type Distribution Chart (Pie)
                new Chart(document.getElementById('returnTypeChart'), {
                    type: 'doughnut',
                    data: {
                        labels: ['Result/Object', 'Other Types'],
                        datasets: [{
                            data: [
                                data.summary.result_return_type_count,
                                data.summary.non_result_return_type_count
                            ],
                            backgroundColor: [
                                'rgba(102, 126, 234, 0.8)',
                                'rgba(118, 75, 162, 0.8)'
                            ],
                            borderColor: [
                                'rgba(102, 126, 234, 1)',
                                'rgba(118, 75, 162, 1)'
                            ],
                            borderWidth: 2
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: true,
                        plugins: {
                            legend: {
                                position: 'bottom',
                                labels: {
                                    font: { size: 13 },
                                    padding: 20
                                }
                            },
                            tooltip: {
                                backgroundColor: 'rgba(0,0,0,0.8)',
                                padding: 12,
                                callbacks: {
                                    label: function(context) {
                                        const label = context.label || '';
                                        const value = context.parsed || 0;
                                        const total = context.dataset.data.reduce((a, b) => a + b, 0);
                                        const percentage = ((value / total) * 100).toFixed(1);
                                        return `${label}: ${value} (${percentage}%)`;
                                    }
                                }
                            }
                        }
                    }
                });
            })
            .catch(error => {
                console.error('Error loading metrics:', error);
                document.querySelector('.container').innerHTML +=
                    '<div style="color:white;text-align:center;margin-top:50px;">Error loading metrics data</div>';
            });
    </script>
</body>
</html>
HTMLEOF

echo "Dashboard generated at $OUTPUT_HTML"
echo ""
echo "Summary:"
echo "  Total FFI Functions: $total_functions"
echo "  Total Domains: ${#domain_counts[@]}"
echo "  Average Params per Function: $avg_params"
echo "  Breaking Changes: $breaking_changes"
echo ""
echo "Open the dashboard: file://$OUTPUT_HTML"
