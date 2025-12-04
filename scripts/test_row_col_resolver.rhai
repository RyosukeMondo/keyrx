// Integration test for row-col API
// Tests that resolver can resolve positions from device profile

fn on_init() {
    // Test remap_rc with home row positions
    // r3_c0 (CapsLock position) → Escape
    remap_rc(3, 0, "Escape");

    // r3_c1 (A position) → B
    remap_rc(3, 1, "B");

    // Test tap_hold_rc
    // r3_c2 (S position): tap=S, hold=LeftAlt
    tap_hold_rc(3, 2, "S", "LeftAlt");

    // Test block_rc
    // r1_c13 (Insert position)
    block_rc(1, 13);

    print("Row-col API integration test completed successfully!");
}
