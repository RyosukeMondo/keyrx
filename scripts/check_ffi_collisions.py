#!/usr/bin/env python3
import os
import re
import sys
from collections import defaultdict

def scan_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    exports = []

    # Pattern 1: Explicit #[no_mangle] extern "C" fn name
    # We handle multiline attributes loosely
    # This regex looks for no_mangle, then eventually "fn <name>"
    # It's a heuristic but should catch most cases in this codebase
    no_mangle_matches = re.finditer(r'#\[no_mangle\].*?fn\s+([a-zA-Z0-9_]+)\s*\(', content, re.DOTALL | re.MULTILINE)
    for m in no_mangle_matches:
        exports.append((m.group(1), 'no_mangle', m.start()))

    # Pattern 2: #[ffi_export] fn name -> keyrx_name
    # This assumes the macro prefixes with "keyrx_"
    ffi_export_matches = re.finditer(r'#\[ffi_export\].*?fn\s+([a-zA-Z0-9_]+)\s*\(', content, re.DOTALL | re.MULTILINE)
    for m in ffi_export_matches:
        rust_name = m.group(1)
        symbol_name = f"keyrx_{rust_name}"
        exports.append((symbol_name, 'ffi_export', m.start()))

    return exports

def check_collisions(root_dir):
    symbols = defaultdict(list)

    for root, dirs, files in os.walk(root_dir):
        for file in files:
            if file.endswith('.rs'):
                path = os.path.join(root, file)
                # Skip target directory if it was somehow walked (unlikely with os.walk on src)
                if 'target' in path:
                    continue

                file_exports = scan_file(path)
                for symbol, source, offset in file_exports:
                    # Check if this specific definition is commented out
                    # Simple check: looks for "//" before the match in the same line
                    # Ideally we'd parse comments properly, but this is a fail-fast script
                    # We can improve this if false positives occur.
                    # For now, we assume the regex matched active code.
                    # (The regex above doesn't filter comments, so we might get false positives if we don't check)

                    # Let's check for comments
                    with open(path, 'r', encoding='utf-8') as f:
                        f.seek(0)
                        content = f.read()
                        # Find the line start
                        line_start = content.rfind('\n', 0, offset) + 1
                        line = content[line_start:content.find('\n', offset)]
                        # Check if line is commented
                        if line.strip().startswith('//'):
                             continue

                    symbols[symbol].append(path)

    collisions = []
    for symbol, paths in symbols.items():
        if len(paths) > 1:
            collisions.append((symbol, paths))

    return collisions

def main():
    core_src = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'core', 'src')
    print(f"Scanning {core_src} for duplicate FFI exports...")

    collisions = check_collisions(core_src)

    if collisions:
        print("\n❌ Duplicate FFI symbols detected!")
        for symbol, paths in collisions:
            print(f"  Symbol '{symbol}' defined in:")
            for p in paths:
                print(f"    - {os.path.relpath(p, os.getcwd())}")
        sys.exit(1)
    else:
        print("✅ No duplicate FFI exports found.")
        sys.exit(0)

if __name__ == '__main__':
    main()
