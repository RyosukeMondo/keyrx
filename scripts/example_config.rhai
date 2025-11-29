// KeyRx Example Configuration
//
// This is an example user configuration script demonstrating
// common remapping patterns.

// Import standard library
// import "std/layouts/ansi" as keys;
// import "std/layers" as layers;
// import "std/modifiers" as mods;

// Define a simple CapsLock <-> Ctrl swap
fn init() {
    print("KeyRx Example Config Loaded");

    // Example remaps (placeholder until engine bindings are ready)
    // input.remap(keys::CAPSLOCK, keys::LCTRL);
    // input.remap(keys::LCTRL, keys::CAPSLOCK);
}

// React to window focus changes
fn on_window_change(app_name) {
    print("Window changed to: " + app_name);

    // Example: Activate Vim layer when vim is focused
    // if app_name == "vim" || app_name == "nvim" {
    //     layers.activate("vim");
    // } else {
    //     layers.deactivate("vim");
    // }
}

// React to key events
fn on_key(event) {
    // Example: Log key events in debug mode
    // print("Key: " + event.key + " pressed: " + event.pressed);
}
