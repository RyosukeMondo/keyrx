// KeyRx Layer Examples
// Run with: keyrx run --script scripts/examples/layers.rhai
//
// Demonstrates defining layers, mapping keys inside layers, and using push/pop/toggle.

fn on_init() {
    // Define layers: base (opaque) plus two transparent layers that fall through.
    layer_define("base", false);
    layer_define("nav", true);
    layer_define("symbols", true);

    // Entry points into layers from the base layer.
    layer_map("base", "CapsLock", "layer_toggle:nav");   // tap to toggle nav on/off
    layer_map("base", "Space", "layer_push:symbols");    // momentarily push symbols

    // Base layer tap-hold: Escape tap, Control hold.
    layer_map("base", "Q", "tap_hold:Escape:LeftCtrl");

    // Nav layer: Vim-style arrows; Space pops back to prior layer.
    layer_map("nav", "H", "Left");
    layer_map("nav", "J", "Down");
    layer_map("nav", "K", "Up");
    layer_map("nav", "L", "Right");
    layer_map("nav", "Space", "layer_pop");
    layer_map("nav", "Escape", "layer_toggle:nav"); // hard toggle off

    // Symbols layer: brackets and number row; Space pops to previous layer.
    layer_map("symbols", "J", "LeftBracket");
    layer_map("symbols", "K", "RightBracket");
    layer_map("symbols", "U", "Minus");
    layer_map("symbols", "I", "Equal");
    layer_map("symbols", "O", "Backslash");
    layer_map("symbols", "Space", "layer_pop");

    print_debug("Layers configured (base/nav/symbols)");
}
