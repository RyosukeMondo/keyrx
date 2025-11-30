// KeyRx Home Row Modifiers
// Run with: keyrx run --script scripts/examples/home_row_mods.rhai
//
// Classic home row mod layout: tap for letters, hold for modifiers.
// Adjust timing for your typing speed; values must be 1-5000 ms.

fn on_init() {
    set_tap_timeout(170);
    set_hold_delay(210);
    set_permissive_hold(true);
    set_eager_tap(true);

    // Left hand: Ctrl, Alt, Meta, Shift on A S D F.
    tap_hold("A", "A", "LeftCtrl");
    tap_hold("S", "S", "LeftAlt");
    tap_hold("D", "D", "LeftMeta");   // Command/Windows
    tap_hold("F", "F", "LeftShift");

    // Right hand: Shift, Meta, Alt, Ctrl on J K L ;.
    tap_hold("J", "J", "RightShift");
    tap_hold("K", "K", "RightMeta");
    tap_hold("L", "L", "RightAlt");
    tap_hold("Semicolon", "Semicolon", "RightCtrl");

    // Optional: space as a Hyper-style virtual modifier for layers/combos.
    let hyper = define_modifier("hyper");
    tap_hold_mod("Space", "Space", hyper);

    print_debug("Home row mods ready");
}
