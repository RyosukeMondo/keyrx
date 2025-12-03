// Golden Session Script: Basic Typing
//
// This script tests basic key remapping - CapsLock to Escape.
// Used as a golden session to verify regression-free behavior.

fn on_init() {
    // Basic CapsLock -> Escape remap
    remap("CapsLock", "Escape");

    print_debug("Basic typing config initialized");
}
