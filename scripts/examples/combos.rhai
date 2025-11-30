// KeyRx Combo Examples
// Run with: keyrx run --script scripts/examples/combos.rhai
//
// Combos are order-independent multi-key chords. Use 2-4 keys per combo.

fn on_init() {
    // Keep the combo window tight so accidental rolls don't trigger combos.
    set_combo_timeout(200);

    // Quick homerow combo for Escape.
    combo(["J", "K"], "Escape");

    // Three-key chord for Tab (press Q+W+E together).
    combo(["Q", "W", "E"], "Tab");

    // Modifier + Space chord for Enter (press LeftAlt + Space together).
    combo(["LeftAlt", "Space"], "Enter");

    // Arrow cluster on ZX combo.
    combo(["Z", "X"], "Left");

    print_debug("Combo bindings registered");
}
