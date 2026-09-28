# KeyRx web UI: personas and experience model

This document defines who the daemon web UI serves and the common workflow each
person should be able to complete without first learning KeyRx terminology.

## Product promise

KeyRx turns an ordinary keyboard, numpad, or spare USB keyboard into a
customizable input device. The UI should make a one-key change feel safe and
quick while keeping layers, macros, tap/hold behavior, and Rhai available to
advanced users.

## Primary personas

### The quick fixer

- Wants to swap or disable one to five keys on their everyday keyboard.
- May not know what a profile, layer, or device scope is.
- Needs a visible sequence: choose scope, select a physical key, assign an
  action, test, and save.
- Success: completes a Caps Lock/Escape-style remap without opening code.

### The gadget re-user

- Owns a cheap numpad, macro pad, or spare keyboard and wants Stream Deck-like
  shortcut buttons without buying new hardware.
- Needs device-specific mappings so the main keyboard is unchanged.
- Benefits from media keys, shortcuts, macros, and visible connection status.
- Success: selects the secondary device, maps several keys, tests the exact
  input/output events, and activates the profile.

### The layout enthusiast

- Wants a full remap, multiple layers, tap/hold behavior, or Vim-style
  navigation.
- Needs fast access to dense controls, editable Rhai, validation, diffs, and
  deterministic saving.
- Success: can move between visual and code editing without configuration drift
  and diagnose behavior from the live monitor.

### The troubleshooter

- Has a mapping that behaves differently than expected.
- Needs to know which profile, device, input, output, layer, and latency were
  involved.
- Success: reproduces a key event, pauses the feed, filters it, and exports the
  evidence.

## Capability and usability map

| User goal                     | Existing capability                                    | UX treatment                                                                              |
| ----------------------------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| Change a few keys             | Visual keyboard and mapping panel                      | “Quick fix” path selects global scope and moves to the keyboard                           |
| Reuse a numpad/spare keyboard | Device-scoped Rhai blocks, device manager, macros      | “Command pad” path selects a concrete device and never changes the main keyboard globally |
| Build a full layout           | Layers, tap/hold, macros, Rhai editor                  | “Power setup” exposes code while preserving the visual editor                             |
| Safely apply changes          | Validation, diff review, simulator, explicit save      | Map → Test workflow, persistent sync state, and one primary Save action                   |
| Explain unexpected behavior   | Live events, state snapshot, latency, filtering/export | Monitor navigation is described as “Inspect live events”                                  |

## Design rules

1. Lead with the intended outcome, then introduce KeyRx concepts in context.
2. Always show the active profile and daemon connection near the primary action.
3. Treat global versus device-specific scope as a safety decision, not an
   implementation detail.
4. Keep visual editing the default; code is an adjacent expert tool, never a
   prerequisite.
5. Testing comes before saving in the visible workflow.
6. Use human labels in navigation and helper text while preserving exact device
   IDs, key codes, and event data where analysis requires them.
