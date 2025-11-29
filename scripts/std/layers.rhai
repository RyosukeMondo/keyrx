// KeyRx Standard Library: Layer Management
//
// Provides utilities for creating and managing keyboard layers.

/// Create a new layer with the given name.
fn new_layer(name) {
    #{
        name: name,
        active: false,
        priority: 0
    }
}

/// Create a layer with a specific priority.
fn new_layer_with_priority(name, priority) {
    #{
        name: name,
        active: false,
        priority: priority
    }
}

/// Activate a layer by name.
fn activate(layer) {
    layer.active = true;
    layer
}

/// Deactivate a layer by name.
fn deactivate(layer) {
    layer.active = false;
    layer
}

/// Toggle a layer's active state.
fn toggle(layer) {
    layer.active = !layer.active;
    layer
}

/// Check if a layer is active.
fn is_active(layer) {
    layer.active
}
