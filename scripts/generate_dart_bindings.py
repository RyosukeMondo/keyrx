#!/usr/bin/env python3
"""
Dart FFI Bindings Generator for KeyRX

Parses FFI contract JSON files (SSOT) and generates type-safe Dart bindings.
Handles:
- Type mapping from contract types (string, json, etc.) to Dart FFI types.
- Automatic injection of error pointers for functions returning complex types.
- Generation of Native and Dart typedefs.
- Generation of the KeyrxBindingsGenerated class.

Usage:
    python3 scripts/generate_dart_bindings.py

Output:
    ui/lib/ffi/generated/bindings_generated.dart
"""

import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional, Tuple, Any

@dataclass
class FfiFunction:
    name: str
    doc_comment: Optional[str]
    rust_name: str
    return_type: str
    parameters: List[Tuple[str, str]]
    domain: str
    has_error_ptr: bool

class ContractReflector:
    """Reflects contract definitions into Dart bindings."""

    # Map explicit C types from contracts to Dart FFI types
    RAW_TYPE_MAP = {
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
        'void': 'Void',
        # Special raw types
        'Option<EventCallback>': 'Pointer<NativeFunction<EventCallbackNative>>',
        'Option<unsafe extern "C" fn(*const u8, usize)>': 'Pointer<NativeFunction<EventCallbackNative>>',
        '*mut u64': 'Pointer<Uint64>',
        '*const u64': 'Pointer<Uint64>',
        'uint16': 'Uint16',
        'int32': 'Int32',
        'uint32': 'Uint32',
        'int': 'Int32',
        'Option<extern "C" fn(*const CLogEntry)>': 'Pointer<NativeFunction<EventCallbackNative>>',
    }

    # Dart types for the public API (typedef alias)
    DART_TYPE_MAP = {
        'Pointer<Char>': 'Pointer<Char>',
        'Pointer<Uint8>': 'Pointer<Uint8>',
        'Bool': 'bool',
        'Uint8': 'int',
        'Uint16': 'int',
        'Uint32': 'int',
        'Uint64': 'int',
        'Int8': 'int',
        'Int16': 'int',
        'Int32': 'int',
        'Int64': 'int',
        'Float': 'double',
        'Double': 'double',
        'Size': 'int',
        'IntPtr': 'int',
        'Void': 'void',
        'Pointer<NativeFunction<EventCallbackNative>>': 'Pointer<NativeFunction<EventCallbackNative>>',
    }

    @classmethod
    def resolve_type(cls, contract_type: str) -> Tuple[str, bool]:
        """
        Resolves a contract type to (dart_ffi_native_type, is_complex).
        is_complex=True means it's a high-level type like 'string' that implies macro handling.
        """
        contract_type = contract_type.strip()

        # High-level types
        if contract_type in ('string', 'json', 'object'):
            return ('Pointer<Char>', True)

        # Explicit C types
        if contract_type in cls.RAW_TYPE_MAP:
            return (cls.RAW_TYPE_MAP[contract_type], False)

        # Default/Fallback
        print(f"Warning: Unknown type '{contract_type}', defaulting to Pointer<Void>")
        return ('Pointer<Void>', False)

    @classmethod
    def get_dart_type(cls, native_type: str) -> str:
        """Maps native FFI type to Dart type."""
        return cls.DART_TYPE_MAP.get(native_type, native_type)

    @classmethod
    def parse_contract(cls, file_path: Path) -> List[FfiFunction]:
        try:
            data = json.loads(file_path.read_text(encoding='utf-8'))
        except Exception as e:
            print(f"Error reading {file_path}: {e}", file=sys.stderr)
            return []

        domain = data.get('domain', 'unknown')
        functions = []

        for fn in data.get('functions', []):
            name = fn['name'] # Short name e.g. "list_virtual_layouts"
            rust_name = fn.get('rust_name', f"keyrx_{domain}_{name}")
            desc = fn.get('description')

            # Parse Return
            ret_def = fn.get('returns', {})
            ret_type_str = ret_def.get('type', 'void')
            native_ret, ret_is_complex = cls.resolve_type(ret_type_str)

            # Parse Params
            params = []
            param_is_complex = False
            for p in fn.get('parameters', []):
                p_type = p['type']
                p_native, p_complex = cls.resolve_type(p_type)
                if p_complex:
                    param_is_complex = True
                params.append((p['name'], p_native))

            # Determine if error pointer needed
            # Rule: If return type is complex (string/json), check if it's NOT in base domain with explicit ptr return.
            # Actually, per analysis: If contract type is "string"/"json", it uses wrapper -> uses error ptr.
            # If contract type is explicit "*mut c_char", it is manual -> no error ptr (usually).

            has_error_ptr = ret_is_complex

            functions.append(FfiFunction(
                name=name,
                doc_comment=desc,
                rust_name=rust_name,
                return_type=native_ret,
                parameters=params,
                domain=domain,
                has_error_ptr=has_error_ptr
            ))

        return functions

class DartBindingsGenerator:
    HEADER = '''/// Auto-generated FFI bindings for KeyRx Core.
///
/// This file is generated by scripts/generate_dart_bindings.py
/// DO NOT EDIT MANUALLY - changes will be overwritten.
///
/// To regenerate: python3 scripts/generate_dart_bindings.py
library;

import 'dart:ffi';
import 'package:ffi/ffi.dart';

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
        # Convert rust_name (keyrx_config_list...) from snake to Pascal for Typedef
        # e.g. KeyrxConfigList...
        # Or follow existing pattern: ConfigListVirtualLayouts

        # Determine strict name for typedef
        # Existing: ConfigListVirtualLayouts
        # Derived from: domain + Pascal(name) type?

        # Helper to pascal case
        def to_pascal(s: str) -> str:
            return ''.join(word.capitalize() for word in s.split('_'))

        # Helper to camel case
        def to_camel(s: str) -> str:
            parts = s.split('_')
            return parts[0].lower() + ''.join(word.capitalize() for word in parts[1:])

        # Rust name usually starts with keyrx_
        # If it matches keyrx_{domain}_{name}, we can simplify

        clean_name = func.rust_name
        if clean_name.startswith('keyrx_'):
            clean_name = clean_name[6:]

        typedef_base = to_pascal(clean_name)
        typedef_native = f"{typedef_base}Native"
        typedef_dart = typedef_base

        # Build params
        native_params = []
        dart_params = []

        for pname, ptype in func.parameters:
            native_params.append(ptype)
            dart_params.append(ContractReflector.get_dart_type(ptype))

        if func.has_error_ptr:
            native_params.append('Pointer<Pointer<Utf8>>') # Error pointer
            # Dart typedef usually mirrors native for simple calls,
            # OR we can expose it.
            # Existing pattern exposed it.
            dart_params.append('Pointer<Pointer<Utf8>>')

        ret_native = func.return_type
        ret_dart = ContractReflector.get_dart_type(func.return_type)

        # Generate Typedefs
        self.typedef_section.append(f"\n// {func.rust_name}")
        if func.doc_comment:
            self.typedef_section.append(f"/// {func.doc_comment}")

        native_sig = ", ".join(native_params)
        dart_sig = ", ".join(dart_params)

        self.typedef_section.append(f"typedef {typedef_native} = {ret_native} Function({native_sig});")
        self.typedef_section.append(f"typedef {typedef_dart} = {ret_dart} Function({dart_sig});")

        # Generate Lookup
        # We need a proper member name.
        # e.g. configListVirtualLayouts (camel case of clean_name)
        member_name = to_camel(clean_name)

        self.lookup_section.append(f"  late final {member_name} = _lookup<NativeFunction<{typedef_native}>>('{func.rust_name}').asFunction<{typedef_dart}>();")

    def generate(self) -> str:
        out = self.HEADER
        out += "\n// Bindings Definitions\n"
        out += "\n".join(self.typedef_section)

        out += "\n\nclass KeyrxBindingsGenerated {\n"
        out += "  final DynamicLibrary _lib;\n"
        out += "  KeyrxBindingsGenerated(this._lib);\n"
        out += "  Pointer<T> _lookup<T extends NativeType>(String name) => _lib.lookup<T>(name);\n\n"
        out += "\n".join(self.lookup_section)
        out += "\n}\n"
        return out

def main():
    root = Path(__file__).parent.parent
    contracts_dir = root / 'core' / 'src' / 'ffi' / 'contracts'

    if not contracts_dir.exists():
        print(f"Contracts dir not found: {contracts_dir}", file=sys.stderr)
        sys.exit(1)

    functions = []
    for f in contracts_dir.glob('*.ffi-contract.json'):
        print(f"Parsing {f.name}...")
        functions.extend(ContractReflector.parse_contract(f))

    print(f"Found {len(functions)} functions.")

    gen = DartBindingsGenerator()
    for fn in functions:
        gen.add_function(fn)

    out_code = gen.generate()

    out_path = root / 'ui' / 'lib' / 'ffi' / 'generated' / 'bindings_generated.dart'
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(out_code, encoding='utf-8')
    print(f"Written to {out_path}")

if __name__ == '__main__':
    main()
