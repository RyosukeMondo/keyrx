#!/usr/bin/env python3
"""
Dart FFI Bindings Generator for KeyRX

Parses Rust FFI exports and generates type-safe Dart bindings.
Scans core/src/ffi/ for #[no_mangle] pub extern "C" functions and generates:
- Native and Dart typedef pairs
- Bindings class with dynamic library symbol lookups
- Type conversions for common FFI patterns

Usage:
    python3 scripts/generate_dart_bindings.py

Output:
    ui/lib/ffi/generated/bindings_generated.dart
"""

import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional, Tuple


@dataclass
class FfiFunction:
    """Represents a parsed FFI function from Rust."""
    name: str
    return_type: str
    parameters: List[Tuple[str, str]]  # [(param_name, param_type)]
    doc_comment: Optional[str] = None


class RustTypeMapper:
    """Maps Rust FFI types to Dart FFI types."""

    RUST_TO_DART_NATIVE = {
        '*const c_char': 'Pointer<Char>',
        '*mut c_char': 'Pointer<Char>',
        '*const u8': 'Pointer<Uint8>',
        '*mut u8': 'Pointer<Uint8>',
        'bool': 'Bool',
        'u8': 'Uint8',
        'u16': 'Uint16',
        'u32': 'Uint32',
        'u64': 'Uint64',
        'i8': 'Int8',
        'i16': 'Int16',
        'i32': 'Int32',
        'i64': 'Int64',
        'f32': 'Float',
        'f64': 'Double',
        'usize': 'Size',
        'isize': 'IntPtr',
        '()': 'Void',
        'Option<EventCallback>': 'Pointer<NativeFunction<EventCallbackNative>>',
    }

    RUST_TO_DART = {
        '*const c_char': 'Pointer<Char>',
        '*mut c_char': 'Pointer<Char>',
        '*const u8': 'Pointer<Uint8>',
        '*mut u8': 'Pointer<Uint8>',
        'bool': 'bool',
        'u8': 'int',
        'u16': 'int',
        'u32': 'int',
        'u64': 'int',
        'i8': 'int',
        'i16': 'int',
        'i32': 'int',
        'i64': 'int',
        'f32': 'double',
        'f64': 'double',
        'usize': 'int',
        'isize': 'int',
        '()': 'void',
        'Option<EventCallback>': 'Pointer<NativeFunction<EventCallbackNative>>',
    }

    @classmethod
    def to_dart_native(cls, rust_type: str) -> str:
        """Convert Rust type to Dart native FFI type."""
        rust_type = rust_type.strip()
        return cls.RUST_TO_DART_NATIVE.get(rust_type, 'Pointer<Void>')

    @classmethod
    def to_dart(cls, rust_type: str) -> str:
        """Convert Rust type to Dart type."""
        rust_type = rust_type.strip()
        return cls.RUST_TO_DART.get(rust_type, 'Pointer<Void>')


class RustFfiParser:
    """Parses Rust FFI export files."""

    # Pattern to match #[no_mangle] pub extern "C" functions
    # Use DOTALL to allow multiline parameters, but ensure we stop at the return type
    FUNCTION_PATTERN = re.compile(
        r'#\[no_mangle\]\s+'
        r'pub\s+(?:unsafe\s+)?extern\s+"C"\s+fn\s+'
        r'(\w+)\s*\(([^)]*)\)\s*->\s*([^\{]+)',
        re.MULTILINE
    )

    # Pattern for void return (no -> part)
    VOID_FUNCTION_PATTERN = re.compile(
        r'#\[no_mangle\]\s+'
        r'pub\s+(?:unsafe\s+)?extern\s+"C"\s+fn\s+'
        r'(\w+)\s*\((.*?)\)\s*\{',
        re.MULTILINE | re.DOTALL
    )

    # Pattern to extract doc comments
    DOC_COMMENT_PATTERN = re.compile(
        r'(?:///(.*?)\n)+\s*#\[no_mangle\]',
        re.MULTILINE
    )

    @classmethod
    def parse_file(cls, file_path: Path) -> List[FfiFunction]:
        """Parse a single Rust file for FFI exports."""
        try:
            content = file_path.read_text(encoding='utf-8')
        except Exception as e:
            print(f"Warning: Could not read {file_path}: {e}", file=sys.stderr)
            return []

        functions = []

        # Parse functions with return types
        for match in cls.FUNCTION_PATTERN.finditer(content):
            func_name = match.group(1)
            params_str = match.group(2)
            return_type = match.group(3).strip()

            # Extract doc comment
            doc_comment = cls._extract_doc_comment(content, match.start())

            # Parse parameters
            parameters = cls._parse_parameters(params_str)

            functions.append(FfiFunction(
                name=func_name,
                return_type=return_type,
                parameters=parameters,
                doc_comment=doc_comment
            ))

        # Parse void functions
        for match in cls.VOID_FUNCTION_PATTERN.finditer(content):
            func_name = match.group(1)
            params_str = match.group(2)

            # Skip if already found (has explicit return type)
            if any(f.name == func_name for f in functions):
                continue

            # Extract doc comment
            doc_comment = cls._extract_doc_comment(content, match.start())

            # Parse parameters
            parameters = cls._parse_parameters(params_str)

            functions.append(FfiFunction(
                name=func_name,
                return_type='()',
                parameters=parameters,
                doc_comment=doc_comment
            ))

        return functions

    @classmethod
    def _extract_doc_comment(cls, content: str, start_pos: int) -> Optional[str]:
        """Extract doc comment before a function."""
        # Look back from start_pos to find doc comments
        before = content[:start_pos]
        lines = before.split('\n')

        doc_lines = []
        for line in reversed(lines):
            stripped = line.strip()
            if stripped.startswith('///'):
                doc_lines.insert(0, stripped[3:].strip())
            elif stripped and not stripped.startswith('//'):
                break

        if doc_lines:
            return '\n'.join(doc_lines)
        return None

    @classmethod
    def _parse_parameters(cls, params_str: str) -> List[Tuple[str, str]]:
        """Parse function parameters string."""
        if not params_str.strip():
            return []

        parameters = []
        # Simple parameter parsing (handles basic cases)
        for param in params_str.split(','):
            param = param.strip()
            if not param:
                continue

            # Split by last colon to handle types like &str
            parts = param.rsplit(':', 1)
            if len(parts) == 2:
                param_name = parts[0].strip()
                param_type = parts[1].strip()

                # Normalize common types
                if param_type == '&str':
                    param_type = '*const c_char'
                elif param_type == 'bool':
                    param_type = 'bool'

                parameters.append((param_name, param_type))

        return parameters


class DartBindingsGenerator:
    """Generates Dart FFI bindings from parsed Rust functions."""

    HEADER = '''/// Auto-generated FFI bindings for KeyRx Core.
///
/// This file is generated by scripts/generate_dart_bindings.py
/// DO NOT EDIT MANUALLY - changes will be overwritten.
///
/// To regenerate: python3 scripts/generate_dart_bindings.py
library;

import 'dart:ffi';

// ────────────────────────────────────────────────────────────────
// Common Callback Typedefs
// ────────────────────────────────────────────────────────────────

/// Event callback signature: receives pointer to JSON bytes and length.
typedef EventCallbackNative = Void Function(Pointer<Uint8>, IntPtr);
typedef EventCallback = void Function(Pointer<Uint8>, int);

'''

    def __init__(self):
        self.typedef_section = []
        self.lookup_section = []

    def add_function(self, func: FfiFunction):
        """Add a function to the generated bindings."""
        # Generate typedef names
        dart_name = self._to_dart_name(func.name)
        native_name = f"{dart_name}Native"

        # Generate type signatures
        return_native = RustTypeMapper.to_dart_native(func.return_type)
        return_dart = RustTypeMapper.to_dart(func.return_type)

        params_native = []
        params_dart = []

        for param_name, param_type in func.parameters:
            param_native = RustTypeMapper.to_dart_native(param_type)
            param_dart = RustTypeMapper.to_dart(param_type)
            params_native.append(param_native)
            params_dart.append(param_dart)

        # Generate typedef section
        typedef_code = f"\n// {func.name}\n"

        if func.doc_comment:
            # Add doc comment
            for line in func.doc_comment.split('\n'):
                typedef_code += f"/// {line}\n"

        # Native typedef
        if params_native:
            params_native_str = ', '.join(params_native)
            typedef_code += f"typedef {native_name} = {return_native} Function({params_native_str});\n"
        else:
            typedef_code += f"typedef {native_name} = {return_native} Function();\n"

        # Dart typedef
        if params_dart:
            params_dart_str = ', '.join(params_dart)
            typedef_code += f"typedef {dart_name} = {return_dart} Function({params_dart_str});\n"
        else:
            typedef_code += f"typedef {dart_name} = {return_dart} Function();\n"

        self.typedef_section.append(typedef_code)

        # Generate lookup section
        # Use Dart-friendly name for the member, but lookup the original Rust name
        member_name = self._to_member_name(func.name)
        lookup_code = f"  late final {dart_name} {member_name} = " \
                     f"_lookup<NativeFunction<{native_name}>>('{func.name}').asFunction();\n"
        self.lookup_section.append(lookup_code)

    def generate(self) -> str:
        """Generate the complete Dart bindings file."""
        output = self.HEADER

        # Add typedefs
        output += "// ────────────────────────────────────────────────────────────────\n"
        output += "// Function Typedefs\n"
        output += "// ────────────────────────────────────────────────────────────────\n"
        output += '\n'.join(self.typedef_section)

        # Add bindings class
        output += "\n// ────────────────────────────────────────────────────────────────\n"
        output += "// Bindings Class\n"
        output += "// ────────────────────────────────────────────────────────────────\n\n"
        output += "/// FFI bindings to the KeyRx Core library.\n"
        output += "class KeyrxBindingsGenerated {\n"
        output += "  final DynamicLibrary _lib;\n\n"
        output += "  KeyrxBindingsGenerated(this._lib);\n\n"
        output += "  /// Look up a symbol in the dynamic library.\n"
        output += "  Pointer<T> _lookup<T extends NativeType>(String name) => _lib.lookup<T>(name);\n\n"
        output += "  // Function bindings\n"
        output += ''.join(self.lookup_section)
        output += "}\n"

        return output

    @staticmethod
    def _to_dart_name(rust_name: str) -> str:
        """Convert keyrx_function_name to KeyrxFunctionName."""
        if rust_name.startswith('keyrx_'):
            name = rust_name[6:]  # Remove 'keyrx_' prefix
        else:
            name = rust_name

        # Convert snake_case to PascalCase
        parts = name.split('_')
        return ''.join(word.capitalize() for word in parts)

    @staticmethod
    def _to_member_name(rust_name: str) -> str:
        """Convert keyrx_function_name to lowerCamelCase for Dart members."""
        if rust_name.startswith('keyrx_'):
            name = rust_name[6:]  # Remove 'keyrx_' prefix
        else:
            name = rust_name

        # Convert snake_case to lowerCamelCase
        parts = name.split('_')
        if not parts:
            return name

        # First part lowercase, rest capitalized
        return parts[0].lower() + ''.join(word.capitalize() for word in parts[1:])


def find_ffi_files(base_path: Path) -> List[Path]:
    """Find all Rust FFI export files."""
    ffi_dir = base_path / 'core' / 'src' / 'ffi'

    if not ffi_dir.exists():
        print(f"Error: FFI directory not found: {ffi_dir}", file=sys.stderr)
        sys.exit(1)

    # Find all .rs files that start with exports_ or are in domains/
    ffi_files = []

    # Core exports.rs file (contains init, version, free_string, event registration)
    exports_file = ffi_dir / 'exports.rs'
    if exports_file.exists():
        ffi_files.append(exports_file)

    # Old-style exports_*.rs files
    for file in ffi_dir.glob('exports_*.rs'):
        ffi_files.append(file)

    # New-style domains/*.rs files
    domains_dir = ffi_dir / 'domains'
    if domains_dir.exists():
        for file in domains_dir.glob('*.rs'):
            if file.name != 'mod.rs':
                ffi_files.append(file)

    return ffi_files


def main():
    """Main entry point."""
    # Determine project root (script is in scripts/ subdirectory)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    print("KeyRX Dart Bindings Generator")
    print("=" * 60)
    print(f"Project root: {project_root}")

    # Find FFI files
    ffi_files = find_ffi_files(project_root)
    print(f"Found {len(ffi_files)} FFI files to process")

    # Parse all FFI functions
    all_functions = []
    for file_path in ffi_files:
        print(f"Parsing: {file_path.relative_to(project_root)}")
        functions = RustFfiParser.parse_file(file_path)
        all_functions.extend(functions)
        print(f"  Found {len(functions)} exported functions")

    print(f"\nTotal FFI functions: {len(all_functions)}")

    # Generate Dart bindings
    print("\nGenerating Dart bindings...")
    generator = DartBindingsGenerator()
    for func in all_functions:
        generator.add_function(func)

    dart_code = generator.generate()

    # Write output
    output_dir = project_root / 'ui' / 'lib' / 'ffi' / 'generated'
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / 'bindings_generated.dart'
    output_file.write_text(dart_code, encoding='utf-8')

    print(f"Generated: {output_file.relative_to(project_root)}")
    print(f"Total lines: {len(dart_code.splitlines())}")
    print("\nSuccess! Dart bindings have been generated.")
    print("Remember to run 'dart format' on the generated file.")


if __name__ == '__main__':
    main()
