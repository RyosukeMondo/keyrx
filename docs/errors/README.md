# Error Documentation

This directory contains auto-generated documentation for all KeyRx error codes.

## Files

- `index.md` - Overview of all error categories
- `config.md` - Configuration errors (KRX-C1xx)
- `runtime.md` - Runtime errors (KRX-R2xx)
- `driver.md` - Driver errors (KRX-D3xx)
- `validation.md` - Validation errors (KRX-V4xx)
- `ffi.md` - FFI errors (KRX-F5xx)
- `internal.md` - Internal errors (KRX-I6xx)

## Regenerating Documentation

**Important**: These files are automatically generated. Do not edit them manually!

### Automatic Regeneration

Documentation is automatically regenerated when:
- Running `just check` (includes all quality checks)
- Running `just build` (before building release binaries)
- Running the `docs-errors` recipe: `just docs-errors`

### Manual Regeneration

If you don't have `just` installed, you can regenerate docs manually:

```bash
# Using the shell script
./scripts/generate-error-docs.sh

# Or directly with cargo
cd core && cargo run --bin generate_error_docs
```

### Build Integration

The `core/build.rs` script tracks all error source files. When any error definition changes, the build system will notify you to regenerate docs. The documentation generation is integrated into the standard build and check workflows to ensure docs stay in sync with code.

## Error Code Format

All error codes follow the format: `KRX-XNNN`
- `KRX` - KeyRx prefix
- `X` - Category letter (C/R/D/V/F/I)
- `NNN` - 3-digit error number

Example: `KRX-C101` (Config category, error 101)
