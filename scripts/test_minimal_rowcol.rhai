// Minimal row-col API test
// Tests basic position-based remapping with your device profile

fn on_init() {
    print("Testing row-col API with device profile...");

    // Test 1: Simple remap - CapsLock (r3_c0) → Escape
    remap_rc(3, 0, "Escape");
    print("✓ Test 1: remap_rc(3, 0, 'Escape')");

    // Test 2: Tap-hold - A key (r3_c1)
    tap_hold_rc(3, 1, "A", "LeftCtrl");
    print("✓ Test 2: tap_hold_rc(3, 1, 'A', 'LeftCtrl')");

    // Test 3: Block - Insert key (r1_c13)
    block_rc(1, 13);
    print("✓ Test 3: block_rc(1, 13)");

    print("\n🎉 All row-col functions executed successfully!");
    print("📍 Device profile loaded and positions resolved to KeyCodes");
    print("\nPress Ctrl+C to exit");
}

fn on_key_event(key, pressed_at) {
    // Echo remapped keys for verification
    debug("Key event: " + key);
}
