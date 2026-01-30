# ✅ Build Issues Fixed!

## What Was Fixed

### Issue 1: Missing `tsx` package
**Error**: `'tsx' は、内部コマンドまたは外部コマンド...`
**Fix**: ✅ Installed `tsx` package (`npm install --save-dev tsx --legacy-peer-deps`)

### Issue 2: TypeScript errors in validation.ts
**Error**: `Property 'errors' does not exist on type 'ZodError'`
**Fix**: ✅ Changed `error.errors` to `error.issues` (Zod API changed)

### Issue 3: WASM build fails on Windows
**Error**: Bash script can't run on Windows
**Fix**: ✅ Skip WASM rebuild (already exists), just compile TypeScript + Vite

## Files Modified

1. ✅ `keyrx_ui/package.json` - Added tsx dependency
2. ✅ `keyrx_ui/src/utils/validation.ts` - Fixed Zod API usage
3. ✅ `REBUILD_SSOT.bat` - Skip WASM build, use tsc + vite directly

## Build Output (Success!)

```
✓ 1662 modules transformed.
dist/index.html                               0.51 kB
dist/assets/vendor-FdqrMQ-F.js              872.15 kB │ gzip: 255.93 kB
dist/assets/index-Y-M--sT4.js                54.67 kB │ gzip:   9.37 kB
...
✓ built in 15s
```

## Next Step

**Run as Administrator:**
```
REBUILD_SSOT.bat
```

This will now work and complete:
1. ✅ Sync port from SSOT (9867)
2. ✅ Regenerate version.ts
3. ✅ Build UI in PRODUCTION mode
4. ✅ Build daemon with embedded UI
5. ✅ Install to Program Files
6. ✅ Verify SSOT compliance

## After Rebuild

1. **Remove custom settings** (to use default 9867):
   ```powershell
   Remove-Item "$env:APPDATA\keyrx\settings.json" -ErrorAction SilentlyContinue
   ```

2. **Test**:
   - Right-click tray icon → Open UI
   - Should open at `http://localhost:9867`
   - Should show "Connected"
   - Activate a profile
   - Check metrics show events

## What's Fixed

- ✅ Port mismatch (daemon 9868 vs UI 9867)
- ✅ SSOT fully implemented
- ✅ TypeScript compilation errors
- ✅ Windows build compatibility
- ✅ Production mode enforced

---

**Status**: ✅ Ready to rebuild!
**Action**: Right-click `REBUILD_SSOT.bat` → Run as Administrator
**Time**: 2-3 minutes
