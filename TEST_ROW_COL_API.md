# Row-Column API Test Plan

## Quick Test (Recommended)

Run the minimal test script with your keyboard:

```bash
sudo ./target/debug/keyrx run --script scripts/test_minimal_rowcol.rhai
```

### Expected Output:
```
[OK] Starting KeyRx engine...
[OK] Using Linux input driver
[OK] Loaded device profile for 099a:0638 from ~/.config/keyrx/devices/099a_0638.json
[OK] Loading script: scripts/test_minimal_rowcol.rhai
Testing row-col API with device profile...
✓ Test 1: remap_rc(3, 0, 'Escape')
✓ Test 2: tap_hold_rc(3, 1, 'A', 'LeftCtrl')
✓ Test 3: block_rc(1, 13)

🎉 All row-col functions executed successfully!
📍 Device profile loaded and positions resolved to KeyCodes

Press Ctrl+C to exit
[OK] Engine started. Press Ctrl+C to stop.
```

### What to Test:
1. **CapsLock → Escape**: Press CapsLock, it should send Escape
2. **A key tap-hold**:
   - Quick press A → types 'a'
   - Hold A → acts as LeftCtrl
3. **Insert blocked**: Press Insert → nothing happens

Press `Ctrl+C` to stop when done.

---

## Full Demo Test

Test all 5 row-col functions with the comprehensive demo:

```bash
sudo ./target/debug/keyrx run --script scripts/examples/row_col_full_demo.rhai
```

This tests:
- ✅ `remap_rc()` - Basic remapping
- ✅ `tap_hold_rc()` - Home row modifiers
- ✅ `block_rc()` - Disable keys
- ✅ `combo_rc()` - Multi-key shortcuts
- ✅ `layer_map_rc()` - Layer-based navigation

---

## Troubleshooting

### Error: "No device profile loaded"
- **Cause**: Device profile not found for your keyboard
- **Solution**: Run `keyrx discover` first to create profile

### Error: "Position rX_cY not found"
- **Cause**: Position doesn't exist in device profile
- **Solution**: Run `./scripts/show_key_position.sh` to see valid positions

### Warning: "Row-col API unavailable"
- **Cause**: Device profile loading failed (non-fatal)
- **Impact**: Row-col functions will error, but script continues
- **Solution**: Check device profile exists and is valid JSON

---

## Manual Verification

If you want to verify the device profile loading manually:

```bash
# 1. Check device profile exists
ls -la ~/.config/keyrx/devices/099a_0638.json

# 2. View available positions
./scripts/show_key_position.sh | head -20

# 3. Check positions used in test:
# r3_c0 = scan_code 58 (CapsLock)
# r3_c1 = scan_code 30 (A)
# r1_c13 = scan_code ? (Insert)
jq '.keymap | to_entries | map(select(.value.row == 3 and .value.col == 0)) | .[0]' \
  ~/.config/keyrx/devices/099a_0638.json
```

---

## Success Criteria

✅ Script loads without errors
✅ Device profile loaded message appears
✅ All test functions execute (✓ marks appear)
✅ CapsLock sends Escape when pressed
✅ A key tap-hold works (tap=a, hold=Ctrl)
✅ Insert key is blocked (no output)

---

## Next Steps After Testing

Once testing confirms everything works:
1. Update README with row-col API documentation
2. Commit implementation with test results
3. Add row-col examples to script library
