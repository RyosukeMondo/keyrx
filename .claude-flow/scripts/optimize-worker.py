#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Incremental Performance Optimization Worker
Analyzes codebase in chunks to avoid timeouts
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
STATE_FILE = STATE_DIR / "optimize-progress.json"
RESULTS_FILE = RESULTS_DIR / "optimize-results.jsonl"

# Ensure directories exist
STATE_DIR.mkdir(parents=True, exist_ok=True)
RESULTS_DIR.mkdir(parents=True, exist_ok=True)

# Modules to analyze (in priority order)
MODULES = [
    "keyrx_core",
    "keyrx_daemon",
    "keyrx_compiler",
    "keyrx_ui/src",
]

def get_state():
    """Read or initialize state"""
    if not STATE_FILE.exists():
        state = {
            "current_index": 0,
            "completed_modules": [],
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

def count_patterns(module_path, pattern):
    """Count occurrences of a regex pattern in module files"""
    count = 0
    for path in Path(module_path).rglob("*"):
        if path.is_file() and path.suffix in ['.rs', '.ts', '.tsx', '.js', '.jsx']:
            try:
                content = path.read_text(encoding='utf-8', errors='ignore')
                count += len(re.findall(pattern, content))
            except Exception:
                pass
    return count

def analyze_module(module_name):
    """Analyze a single module for optimization opportunities"""
    module_path = PROJECT_ROOT / module_name

    if not module_path.exists():
        return {
            "success": False,
            "module": module_name,
            "error": "Module not found"
        }

    print(f"Analyzing module: {module_name}")

    suggestions = []
    metrics = {}

    # 1. Check for excessive clones
    print("  Checking for excessive clones...")
    clone_count = count_patterns(module_path, r'\.clone\(\)')
    metrics['clones'] = clone_count
    if clone_count > 50:
        suggestions.append({
            "type": "excessive-clones",
            "severity": "medium",
            "count": clone_count,
            "suggestion": f"Found {clone_count} .clone() calls. Review for unnecessary memory allocations."
        })

    # 2. Check for string allocations
    print("  Checking for string allocations...")
    string_alloc = count_patterns(module_path, r'String::from|to_string\(\)')
    metrics['string_allocations'] = string_alloc
    if string_alloc > 100:
        suggestions.append({
            "type": "string-allocations",
            "severity": "low",
            "count": string_alloc,
            "suggestion": f"Found {string_alloc} string allocations. Consider using &str or Cow<str>."
        })

    # 3. Check for N+1 patterns (queries in loops)
    print("  Checking for N+1 query patterns...")
    n_plus_one = count_patterns(module_path, r'(for|while).*\.(query|execute)')
    metrics['potential_n_plus_one'] = n_plus_one
    if n_plus_one > 0:
        suggestions.append({
            "type": "n+1-query",
            "severity": "high",
            "count": n_plus_one,
            "suggestion": f"Found {n_plus_one} potential N+1 query patterns. Use batch queries or joins."
        })

    # 4. React-specific checks
    if "ui" in module_name.lower():
        print("  Checking React optimizations...")

        # Missing memoization
        callbacks = count_patterns(module_path, r'(function|const\s+\w+\s*=)')
        memoized = count_patterns(module_path, r'(useCallback|useMemo|React\.memo)')
        metrics['callbacks'] = callbacks
        metrics['memoized'] = memoized

        if callbacks > 20 and memoized < callbacks / 4:
            suggestions.append({
                "type": "missing-memoization",
                "severity": "medium",
                "suggestion": f"Only {memoized} memoized out of {callbacks} functions. Use useCallback/useMemo."
            })

        # Inline objects in JSX
        inline_objects = count_patterns(module_path, r'=\{|\=\[')
        metrics['inline_objects'] = inline_objects
        if inline_objects > 50:
            suggestions.append({
                "type": "inline-objects",
                "severity": "low",
                "count": inline_objects,
                "suggestion": f"Found {inline_objects} inline objects/arrays. May cause re-renders."
            })

    # 5. Async patterns
    print("  Checking async patterns...")
    async_count = count_patterns(module_path, r'async (fn|function)')
    await_count = count_patterns(module_path, r'\bawait\b')
    metrics['async_functions'] = async_count
    metrics['awaits'] = await_count

    if async_count > 10 and await_count > async_count * 5:
        suggestions.append({
            "type": "sequential-awaits",
            "severity": "medium",
            "suggestion": "Potential sequential awaits detected. Consider Promise.all() for parallel operations."
        })

    # Build result
    result = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "module": module_name,
        "suggestionCount": len(suggestions),
        "suggestions": suggestions,
        "metrics": metrics
    }

    # Append to results file
    with open(RESULTS_FILE, 'a', encoding='utf-8') as f:
        f.write(json.dumps(result) + '\n')

    # Summary
    print(f"[OK] Analysis complete: {module_name}")
    print(f"  Found {len(suggestions)} optimization opportunities")
    if suggestions:
        print("\nSuggestions:")
        for s in suggestions:
            print(f"  - {s['type']}: {s['suggestion']}")

    return {
        "success": True,
        "module": module_name,
        "suggestionCount": len(suggestions),
        "metrics": metrics,
        "message": f"Analyzed {module_name}: found {len(suggestions)} optimization opportunities"
    }

def main():
    """Main entry point"""
    # Get current state
    state = get_state()
    current_index = state["current_index"]

    # Wrap around if needed
    if current_index >= len(MODULES):
        current_index = 0
        state["current_index"] = 0

    # Get module to analyze
    module_name = MODULES[current_index]

    # Analyze
    result = analyze_module(module_name)

    # Update state
    state["current_index"] = (current_index + 1) % len(MODULES)
    state["last_analyzed"] = datetime.now(timezone.utc).isoformat()
    if module_name not in state["completed_modules"]:
        state["completed_modules"].append(module_name)
    save_state(state)

    # Output JSON result
    print("\n" + json.dumps(result, indent=2))

    return 0 if result.get("success", False) else 1

if __name__ == "__main__":
    sys.exit(main())
