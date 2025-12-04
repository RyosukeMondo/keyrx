// Test script to see row/column detection
// Device profile: USB Keyboard (099a:0638)
// 6 rows, varying columns per row

fn on_init() {
    print_debug("Row/Column Test Script Loaded");
    print_debug("Your keyboard has 6 rows");
    print_debug("Try simulating keys to see their positions");

    // Example: Key 'A' is at row 1, col 3
    // In your profile: scan_code 4 → r1_c3

    // Map a few keys by their row/column position
    // These are just examples - your profile shows all mappings

    // Row 0 keys (top row - Esc, F keys, etc.)
    print_debug("Row 0: Escape, F1-F12, etc.");

    // Row 1 keys (number row)
    print_debug("Row 1: Number row (1234567890)");

    // Row 2 keys (QWERTY row)
    print_debug("Row 2: QWERTY row");

    // Row 3 keys (ASDF row)
    print_debug("Row 3: ASDF row (home row)");

    // Row 4 keys (ZXCV row)
    print_debug("Row 4: ZXCV row");

    // Row 5 keys (bottom modifiers)
    print_debug("Row 5: Space, Alt, Ctrl, etc.");
}
