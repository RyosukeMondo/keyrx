// Golden Session Script: Layer Switching
//
// This script tests layer-based key remapping with Vim-style navigation.
// Used as a golden session to verify layer functionality remains consistent.

fn on_init() {
    // Define layers: base (opaque) and nav (transparent)
    layer_define("base", false);
    layer_define("nav", true);

    // CapsLock toggles nav layer on/off
    layer_map("base", "CapsLock", "layer_toggle:nav");

    // Nav layer: HJKL for Vim-style navigation
    layer_map("nav", "H", "Left");
    layer_map("nav", "J", "Down");
    layer_map("nav", "K", "Up");
    layer_map("nav", "L", "Right");

    // Escape exits nav layer
    layer_map("nav", "Escape", "layer_toggle:nav");

    print_debug("Layer switch config initialized");
}
