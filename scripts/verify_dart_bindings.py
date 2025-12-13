#!/usr/bin/env python3
"""
Verify Dart Bindings

Checks if the generated Dart bindings match the FFI contracts.
Usage:
    python3 scripts/verify_dart_bindings.py

Exit Code:
    0: Bindings are up-to-date.
    1: Bindings are stale or missing.
"""

import sys
from pathlib import Path
from tempfile import TemporaryDirectory
import generate_dart_bindings

def main():
    root = Path(__file__).parent.parent
    contracts_dir = root / 'core' / 'src' / 'ffi' / 'contracts'
    existing_file = root / 'ui' / 'lib' / 'ffi' / 'generated' / 'bindings_generated.dart'

    print("Verifying Dart bindings against FFI contracts...")

    if not existing_file.exists():
        print(f"Error: Bindings file not found: {existing_file}")
        sys.exit(1)

    # Generate fresh bindings in memory/temp
    functions = []
    for f in contracts_dir.glob('*.ffi-contract.json'):
        functions.extend(generate_dart_bindings.ContractReflector.parse_contract(f))

    gen = generate_dart_bindings.DartBindingsGenerator()
    for fn in functions:
        gen.add_function(fn)

    generated_code = gen.generate()
    existing_code = existing_file.read_text(encoding='utf-8')

    # Normalize line endings
    generated_code = generated_code.replace('\r\n', '\n')
    existing_code = existing_code.replace('\r\n', '\n')

    if generated_code.strip() != existing_code.strip():
        print("Error: Dart bindings are stale!")
        print("Please run: python scripts/generate_dart_bindings.py")
        sys.exit(1)

    print("Success: Dart bindings are up-to-date.")
    sys.exit(0)

if __name__ == '__main__':
    main()
