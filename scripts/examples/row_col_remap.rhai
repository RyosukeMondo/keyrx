// Row-Column Based Remapping Example
// Device: USB Keyboard (099a:0638)
//
// This script demonstrates remapping keys based on their physical position
// rather than their logical key names. Useful for:
// - Physical layout-based configurations
// - Keyboard-agnostic scripts
// - Alternative layouts (Colemak, Dvorak)
//
// Run with: keyrx run --script scripts/examples/row_col_remap.rhai

// ============================================================================
// ROW-COLUMN REFERENCE (from device profile)
// ============================================================================
//
// Row 0 (top row - 17 keys):
//   r0_c0  = Escape        (scan 1)
//   r0_c1  = F1            (scan 59)
//   r0_c2  = F2            (scan 60)
//   ...
//
// Row 1 (number row - 15 keys):
//   r1_c0  = Backquote     (scan 41)   `~
//   r1_c1  = Digit1        (scan 2)    1!
//   r1_c2  = Digit2        (scan 3)    2@
//   r1_c3  = A             (scan 4)    3#   <- NOTE: This is '3', not 'A'!
//   r1_c4  = B             (scan 5)    4$
//   ...
//
// Row 2 (QWERTY row - 14 keys):
//   r2_c0  = Tab           (scan 15)
//   r2_c1  = Q             (scan 16)
//   r2_c2  = W             (scan 17)
//   r2_c3  = E             (scan 18)
//   ...
//
// Row 3 (home row - 13 keys):
//   r3_c0  = CapsLock      (scan 58)
//   r3_c1  = A             (scan 30)   <- This is the real 'A'!
//   r3_c2  = S             (scan 31)
//   r3_c3  = D             (scan 32)
//   r3_c4  = F             (scan 33)
//   r3_c5  = G             (scan 34)
//   r3_c6  = H             (scan 35)
//   r3_c7  = J             (scan 36)
//   r3_c8  = K             (scan 37)
//   r3_c9  = L             (scan 38)
//   r3_c10 = Semicolon     (scan 39)
//   r3_c11 = Quote         (scan 40)
//   r3_c12 = Backslash     (scan 43)
//
// Row 4 (ZXCV row - 14 keys):
//   r4_c0  = LeftShift     (scan 42)
//   r4_c1  = Z             (scan 44)
//   r4_c2  = X             (scan 45)
//   r4_c3  = C             (scan 46)
//   r4_c4  = V             (scan 47)
//   r4_c5  = B             (scan 48)
//   r4_c6  = N             (scan 49)
//   r4_c7  = M             (scan 50)
//   r4_c8  = Comma         (scan 51)
//   r4_c9  = Period        (scan 52)
//   r4_c10 = Slash         (scan 53)
//   r4_c13 = RightShift    (scan 54)
//
// Row 5 (bottom row - 12 keys):
//   r5_c0  = LeftCtrl      (scan 29)
//   r5_c1  = LeftSuper     (scan 125)
//   r5_c2  = LeftAlt       (scan 56)
//   r5_c4  = Space         (scan 57)
//   r5_c5  = RightCtrl     (scan 92)
//   r5_c6  = RightAlt      (scan 93)
//   ...

fn on_init() {
    print_debug("=== Row-Column Based Remapping ===");
    print_debug("Remapping keys based on physical position");

    // ========================================================================
    // EXAMPLE 1: Swap home row keys (position-based)
    // ========================================================================
    // Swap the leftmost home row key (CapsLock at r3_c0) with Escape (r0_c0)
    remap("CapsLock", "Escape");  // r3_c0 → Escape
    remap("Escape", "CapsLock");  // r0_c0 → CapsLock

    // ========================================================================
    // EXAMPLE 2: Home row modifiers (physical columns)
    // ========================================================================
    // Map home row positions to modifiers:
    //   r3_c1 (A) = Ctrl on hold
    //   r3_c2 (S) = Alt on hold
    //   r3_c3 (D) = Meta on hold
    //   r3_c4 (F) = Shift on hold

    tap_hold("A", "A", "LeftCtrl");   // r3_c1: tap=A, hold=Ctrl
    tap_hold("S", "S", "LeftAlt");    // r3_c2: tap=S, hold=Alt
    tap_hold("D", "D", "LeftMeta");   // r3_c3: tap=D, hold=Meta
    tap_hold("F", "F", "LeftShift");  // r3_c4: tap=F, hold=Shift

    // Right hand (symmetrical)
    tap_hold("J", "J", "RightShift"); // r3_c7: tap=J, hold=Shift
    tap_hold("K", "K", "RightMeta");  // r3_c8: tap=K, hold=Meta
    tap_hold("L", "L", "RightAlt");   // r3_c9: tap=L, hold=Alt
    tap_hold("Semicolon", "Semicolon", "RightCtrl"); // r3_c10

    // ========================================================================
    // EXAMPLE 3: Block keys by physical position
    // ========================================================================
    // Block the top-right corner keys (often accidentally pressed)
    block("Insert");      // r1_c13
    block("Delete");      // r0_c16
    block("Pause");       // r0_c15

    // ========================================================================
    // EXAMPLE 4: Create custom layout (Colemak-style)
    // ========================================================================
    // Remap QWERTY row 2 positions to different letters
    // (This is just an example - for real Colemak, use a complete mapping)

    // Uncomment to try:
    // remap("E", "F");  // r2_c3 → F
    // remap("R", "P");  // r2_c4 → P
    // remap("T", "G");  // r2_c5 → G

    print_debug("Row-column remapping active!");
    print_debug("Try: CapsLock (r3_c0) → Escape");
    print_debug("Try: Hold A/S/D/F (r3_c1-4) for modifiers");
}

// ============================================================================
// USAGE TIPS
// ============================================================================
//
// 1. Use ./scripts/show_key_position.sh to look up positions:
//    ./scripts/show_key_position.sh all | grep "(row 3,"
//
// 2. Test with simulate:
//    keyrx simulate --input "A,S,D,F" --script this_file.rhai
//
// 3. Create layout-independent configs:
//    - Reference keys by physical position (row/col)
//    - Makes configs portable across different keyboard layouts
//    - Easier to describe physical finger positions
//
// 4. Document your mappings with row-column comments!
//    Always include "// r3_c1" comments for clarity
