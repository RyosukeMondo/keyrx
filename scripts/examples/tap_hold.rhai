// KeyRx Tap-Hold Examples
// Run with: keyrx run --script scripts/examples/tap_hold.rhai
//
// Demonstrates tap_hold() and tap_hold_mod() with timing tuning.
// Each binding taps when released quickly and holds on timeout.

fn on_init() {
    // Timing tuned for responsive taps; tweak to your preference (1-5000 ms).
    set_tap_timeout(180);
    set_hold_delay(200);
    set_permissive_hold(true);
    set_eager_tap(true);
    set_retro_tap(false);

    // Tap Escape on CapsLock, hold for Control (popular Vim-friendly swap).
    tap_hold("CapsLock", "Escape", "LeftCtrl");

    // Space bar: tap for space, hold for Shift (one-handed Shift).
    tap_hold("Space", "Space", "LeftShift");

    // Enter: quick press sends Enter, hold for OS/meta (Command/Windows).
    tap_hold("Enter", "Enter", "LeftMeta");

    // Virtual modifier demo: hold F to engage a custom "nav" modifier.
    let nav_mod = define_modifier("nav");
    tap_hold_mod("F", "F", nav_mod);

    print_debug("Tap-hold bindings ready");
}
