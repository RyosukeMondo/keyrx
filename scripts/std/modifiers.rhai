// KeyRx Standard Library: Modifier Definitions
//
// Provides standard modifier constants and utilities.

// Standard modifier IDs (0-15 reserved for system modifiers)
const MOD_SHIFT = 0;
const MOD_CTRL = 1;
const MOD_ALT = 2;
const MOD_WIN = 3;

// User-defined modifier range starts at 16
const MOD_USER_START = 16;

/// Create a custom modifier with the given ID.
fn custom_mod(id) {
    if id < MOD_USER_START {
        throw "Custom modifier ID must be >= " + MOD_USER_START;
    }
    #{
        id: id,
        name: "Custom_" + id
    }
}

/// Create a named custom modifier.
fn named_mod(id, name) {
    if id < MOD_USER_START {
        throw "Custom modifier ID must be >= " + MOD_USER_START;
    }
    #{
        id: id,
        name: name
    }
}

// Common custom modifiers
fn mod_thumb() { named_mod(16, "Thumb") }
fn mod_gaming() { named_mod(17, "Gaming") }
fn mod_nav() { named_mod(18, "Navigation") }
fn mod_symbols() { named_mod(19, "Symbols") }
fn mod_numpad() { named_mod(20, "Numpad") }
