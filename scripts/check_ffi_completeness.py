#!/usr/bin/env python3
"""
Check FFI completeness between Rust and Dart.

This script detects:
1. Rust exports that are missing in Dart bindings.
2. Dart bindings that refer to missing Rust exports.

Usage:
    python3 scripts/check_ffi_completeness.py
"""

import os
import re
import sys
from collections import defaultdict

def scan_rust_exports(root_dir):
    """Scan Rust source files for FFI exports."""
    exports = set()

    # Regex for #[no_mangle] pub extern "C" fn name
    # Captures 'name' from: #[no_mangle] ... fn name(
    no_mangle_re = re.compile(r'#\[no_mangle\].*?fn\s+([a-zA-Z0-9_]+)\s*\(', re.DOTALL | re.MULTILINE)

    # Regex for #[ffi_export] fn name
    # Captures 'name' from: #[ffi_export] ... fn name( -> implies keyrx_name
    ffi_export_re = re.compile(r'#\[ffi_export\].*?fn\s+([a-zA-Z0-9_]+)\s*\(', re.DOTALL | re.MULTILINE)

    for root, dirs, files in os.walk(root_dir):
        for file in files:
            if file.endswith('.rs'):
                path = os.path.join(root, file)
                if 'target' in path:
                    continue

                try:
                    with open(path, 'r', encoding='utf-8') as f:
                        content = f.read()

                    # Scan no_mangle
                    for m in no_mangle_re.finditer(content):
                        exports.add(m.group(1))

                    # Scan ffi_export
                    for m in ffi_export_re.finditer(content):
                        exports.add(f"keyrx_{m.group(1)}")

                except Exception as e:
                    print(f"Warning: Failed to read {path}: {e}")

    return exports

def scan_dart_lookups(file_paths):
    """Scan Dart files for FFI lookups."""
    lookups = set()

    # Regex for lookups
    # Matches: _lookup<...>( 'name' )
    # Matches: lookupFunction<...>( 'name' )
    lookup_re = re.compile(r"(?:_lookup|lookupFunction)<.*?>\s*\(\s*'([a-zA-Z0-9_]+)'\s*(?:,|\))", re.DOTALL | re.MULTILINE)

    for path in file_paths:
        if not os.path.exists(path):
            print(f"Warning: Dart file not found: {path}")
            continue

        try:
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()

            for m in lookup_re.finditer(content):
                lookups.add(m.group(1))

        except Exception as e:
            print(f"Warning: Failed to read {path}: {e}")

    return lookups

def main():
    # Determine paths
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)

    core_src = os.path.join(project_root, 'core', 'src')
    dart_ffi_dir = os.path.join(project_root, 'ui', 'lib', 'ffi')

    dart_files = [
        os.path.join(dart_ffi_dir, 'generated', 'bindings_generated.dart'),
        os.path.join(dart_ffi_dir, 'bindings.dart'),
    ]

    print("Scanning Rust exports...")
    rust_exports = scan_rust_exports(core_src)
    print(f"Found {len(rust_exports)} Rust exports.")

    print("Scanning Dart bindings...")
    dart_lookups = scan_dart_lookups(dart_files)
    print(f"Found {len(dart_lookups)} Dart lookups.")

    errors = 0

    # Check 1: Rust exports missing in Dart (Not Wired)
    # We filter out some common false positives or internal testing functions if needed
    missing_in_dart = rust_exports - dart_lookups
    if missing_in_dart:
        print("\n[WARNING] Rust exports missing in Dart bindings (Not Wired):")
        for sym in sorted(missing_in_dart):
            print(f"  - {sym}")
        # This might be intentional (unused API), so we treat as warning or error depending on strictness
        # The user asked to "detect" it.

    # Check 2: Dart lookups missing in Rust (Broken Link)
    missing_in_rust = dart_lookups - rust_exports
    if missing_in_rust:
        print("\n[ERROR] Dart bindings referring to missing Rust exports (Broken Link):")
        for sym in sorted(missing_in_rust):
            print(f"  - {sym}")
        errors += 1

    if errors > 0:
        print("\nFAILED: FFI consistency check failed.")
        sys.exit(1)

    print("\nSUCCESS: FFI bindings are consistent.")
    sys.exit(0)

if __name__ == '__main__':
    main()
