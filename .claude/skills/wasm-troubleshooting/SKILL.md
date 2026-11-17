---
name: wasm-troubleshooting
description: Diagnose and fix keyrx WASM build/loading problems (WASM not available, hash mismatch, wasm-pack missing, tiny .wasm, UI badge red). Use when the UI simulator/validation says WASM is unavailable or a build fails on WASM.
---

# keyrx WASM troubleshooting

**Quick health check:**
```bash
./scripts/wasm-health.sh            # Comprehensive diagnostics
```

**Common errors:**

| Error | Fix |
|-------|-----|
| "WASM not available" | `cd keyrx_ui && npm run build:wasm` |
| "wasm-pack not found" | `cargo install wasm-pack` |
| "wasm32-unknown-unknown not installed" | `rustup target add wasm32-unknown-unknown` |
| "Hash mismatch" | `cd keyrx_ui && npm run rebuild:wasm` |
| WASM file < 100KB | Check logs, run `npm run rebuild:wasm` |

**Build & verify:**
```bash
cd keyrx_ui
npm run build:wasm                  # Build
npm run rebuild:wasm                # Clean + build
npm run clean:wasm                  # Clean artifacts
../scripts/verify-wasm.sh           # Verify integrity
```

**Debug loading:**
1. Check browser console (F12)
2. Check WASM status badge in UI
3. Force rebuild: `rm -rf src/wasm/pkg && npm run build:wasm`
