# Verification Scripts

Scripts that **verify correctness** and **fail if something is wrong** (exit code != 0).

## Pipeline Layers

These scripts verify different transformation layers (NOT duplicates!):

```
1. Rust #[ffi_export]
        ↓
2. JSON Contracts      ← contracts.sh verifies this
        ↓
3. Dart Bindings       ← bindings.py verifies this
        ↓
4. SSOT Rules          ← ssot.sh verifies this
```

## Scripts

| Script | Verifies | Fails If |
|--------|----------|----------|
| `bindings.py` | Dart bindings match Rust exports | Bindings out of sync |
| `contracts.sh` | JSON contracts match #[ffi_export] | Missing/orphaned contracts |
| `registry.sh` | CLI persistence works | Config not persisted |
| `ssot.sh` | No SSOT violations | Duplicate definitions found |
| `breaking-changes.sh` | No FFI breaking changes | BC detected vs base ref |
| `all.sh` | All verifications pass | Any verification fails |

## Usage

```bash
# Verify individual layers
./scripts/verify/bindings.py
./scripts/verify/contracts.sh

# Verify everything
./scripts/verify/all.sh

# Via justfile
just verify-bindings
just verify-contracts
just verify-all
```

## Exit Codes

- `0` - Verification passed ✅
- `1` - Verification failed ❌
