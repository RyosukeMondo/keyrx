#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Incremental Test Coverage Analysis Worker
Analyzes test coverage gaps in chunks to avoid timeouts
"""

import os
import json
import sys
from pathlib import Path
from datetime import datetime, timezone
import re

# Force UTF-8 encoding on Windows
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')

# Paths
SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent.parent
STATE_DIR = SCRIPT_DIR.parent / "state"
RESULTS_DIR = SCRIPT_DIR.parent / "metrics"
STATE_FILE = STATE_DIR / "testgaps-progress.json"
RESULTS_FILE = RESULTS_DIR / "testgaps-results.jsonl"

# Ensure directories exist
STATE_DIR.mkdir(parents=True, exist_ok=True)
RESULTS_DIR.mkdir(parents=True, exist_ok=True)

# Crates to analyze (in priority order)
CRATES = [
    "keyrx_core",
    "keyrx_daemon",
    "keyrx_compiler",
]

def get_state():
    """Read or initialize state"""
    if not STATE_FILE.exists():
        state = {
            "current_index": 0,
            "completed_crates": [],
            "last_analyzed": None,
        }
        save_state(state)
        return state

    with open(STATE_FILE) as f:
        return json.load(f)

def save_state(state):
    """Save state to file"""
    with open(STATE_FILE, 'w') as f:
        json.dump(state, f, indent=2)

def count_in_files(path, pattern, extensions=None):
    """Count occurrences of a pattern in files"""
    if extensions is None:
        extensions = ['.rs']

    count = 0
    for file_path in Path(path).rglob("*"):
        if file_path.is_file() and file_path.suffix in extensions:
            try:
                content = file_path.read_text(encoding='utf-8', errors='ignore')
                count += len(re.findall(pattern, content, re.MULTILINE))
            except Exception:
                pass
    return count

def count_files(path, pattern="*", extensions=None):
    """Count files matching a pattern"""
    if extensions is None:
        extensions = ['.rs']

    count = 0
    for file_path in Path(path).rglob(pattern):
        if file_path.is_file() and file_path.suffix in extensions:
            count += 1
    return count

def count_lines(path, extensions=None):
    """Count total lines of code"""
    if extensions is None:
        extensions = ['.rs']

    lines = 0
    for file_path in Path(path).rglob("*"):
        if file_path.is_file() and file_path.suffix in extensions:
            try:
                lines += len(file_path.read_text(encoding='utf-8', errors='ignore').splitlines())
            except Exception:
                pass
    return lines

def analyze_crate(crate_name):
    """Analyze a single crate for test coverage gaps"""
    crate_path = PROJECT_ROOT / crate_name

    if not crate_path.exists():
        return {
            "success": False,
            "crate": crate_name,
            "error": "Crate not found"
        }

    print(f"Analyzing test coverage for: {crate_name}")

    gaps = []
    metrics = {}

    src_path = crate_path / "src"
    tests_path = crate_path / "tests"

    # 1. Check for untested public functions
    print("  Checking for untested public functions...")
    pub_fn_count = count_in_files(src_path, r'pub\s+fn\s+')
    test_fn_count = count_in_files(tests_path, r'#\[test\]')

    metrics['public_functions'] = pub_fn_count
    metrics['test_functions'] = test_fn_count

    if pub_fn_count > 0 and test_fn_count < pub_fn_count / 2:
        gaps.append({
            "type": "untested-functions",
            "severity": "high",
            "suggestion": f"{pub_fn_count} public functions but only {test_fn_count} tests. Add more unit tests."
        })

    # 2. Check for error handling test gaps
    print("  Checking error handling coverage...")
    result_count = count_in_files(src_path, r'Result<')
    error_test_count = count_in_files(tests_path, r'(should_panic|expect_err|is_err|unwrap_err)')

    metrics['result_types'] = result_count
    metrics['error_tests'] = error_test_count

    if result_count > 0 and error_test_count < result_count / 3:
        gaps.append({
            "type": "missing-error-tests",
            "severity": "high",
            "suggestion": f"{result_count} Result types but only {error_test_count} error tests. Add error path testing."
        })

    # 3. Check for integration tests
    print("  Checking integration test coverage...")
    integration_tests = count_files(tests_path) if tests_path.exists() else 0

    metrics['integration_tests'] = integration_tests

    if integration_tests < 3:
        gaps.append({
            "type": "missing-integration-tests",
            "severity": "medium",
            "suggestion": f"Only {integration_tests} integration test files. Add more end-to-end tests."
        })

    # 4. Check for documentation tests
    print("  Checking documentation test coverage...")
    doc_tests = count_in_files(src_path, r'```rust')

    metrics['doc_tests'] = doc_tests

    if doc_tests < 5:
        gaps.append({
            "type": "missing-doc-tests",
            "severity": "low",
            "suggestion": f"Only {doc_tests} documentation tests. Add examples to public APIs."
        })

    # 5. Estimate coverage (simple heuristic)
    print("  Estimating coverage...")
    source_lines = count_lines(src_path)
    test_lines = count_lines(tests_path) if tests_path.exists() else 0

    estimated_coverage = (test_lines * 100 // source_lines) if source_lines > 0 else 0

    metrics['source_lines'] = source_lines
    metrics['test_lines'] = test_lines
    metrics['estimated_coverage'] = estimated_coverage

    # Build result
    result = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "crate": crate_name,
        "gapCount": len(gaps),
        "estimatedCoverage": estimated_coverage,
        "gaps": gaps,
        "metrics": metrics
    }

    # Append to results file
    with open(RESULTS_FILE, 'a', encoding='utf-8') as f:
        f.write(json.dumps(result) + '\n')

    # Summary
    print(f"[OK] Analysis complete: {crate_name}")
    print(f"  Estimated coverage: ~{estimated_coverage}% (test lines / source lines)")
    print(f"  Found {len(gaps)} test gaps")
    if gaps:
        print("\nGaps identified:")
        for g in gaps:
            print(f"  - {g['type']}: {g['suggestion']}")

    return {
        "success": True,
        "crate": crate_name,
        "gapCount": len(gaps),
        "estimatedCoverage": estimated_coverage,
        "metrics": metrics,
        "message": f"Analyzed {crate_name}: ~{estimated_coverage}% coverage, found {len(gaps)} gaps"
    }

def main():
    """Main entry point"""
    # Get current state
    state = get_state()
    current_index = state["current_index"]

    # Wrap around if needed
    if current_index >= len(CRATES):
        current_index = 0
        state["current_index"] = 0

    # Get crate to analyze
    crate_name = CRATES[current_index]

    # Analyze
    result = analyze_crate(crate_name)

    # Update state
    state["current_index"] = (current_index + 1) % len(CRATES)
    state["last_analyzed"] = datetime.now(timezone.utc).isoformat()
    if crate_name not in state["completed_crates"]:
        state["completed_crates"].append(crate_name)
    save_state(state)

    # Output JSON result
    print("\n" + json.dumps(result, indent=2))

    return 0 if result.get("success", False) else 1

if __name__ == "__main__":
    sys.exit(main())
