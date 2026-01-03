use keyrx_core::config::{Condition, ConditionItem, KeyMapping};
use rhai::{Array, Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;
use crate::parser::validators::parse_condition_string;

pub fn register_when_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    // when_start() for single condition string
    let state_clone_single = Arc::clone(&state);
    engine.register_fn(
        "when_start",
        move |cond: &str| -> Result<(), Box<EvalAltResult>> {
            let condition =
                parse_condition_string(cond).map_err(|e| format!("Invalid condition: {}", e))?;
            start_conditional_block(&state_clone_single, condition)
        },
    );

    // when_start() for array of conditions (AllActive)
    let state_clone_multi = Arc::clone(&state);
    engine.register_fn(
        "when_start",
        move |conds: Array| -> Result<(), Box<EvalAltResult>> {
            let mut condition_items = Vec::new();
            for cond_dyn in conds {
                let cond_str = cond_dyn
                    .into_string()
                    .map_err(|_| "Condition must be a string")?;
                let cond = parse_condition_string(&cond_str)
                    .map_err(|e| format!("Invalid condition: {}", e))?;
                match cond {
                    Condition::ModifierActive(id) => {
                        condition_items.push(ConditionItem::ModifierActive(id))
                    }
                    Condition::LockActive(id) => {
                        condition_items.push(ConditionItem::LockActive(id))
                    }
                    _ => return Err("Only single modifiers/locks allowed in array".into()),
                }
            }
            start_conditional_block(&state_clone_multi, Condition::AllActive(condition_items))
        },
    );

    // when_end() - finalize conditional block
    let state_clone_end = Arc::clone(&state);
    engine.register_fn("when_end", move || -> Result<(), Box<EvalAltResult>> {
        end_conditional_block(&state_clone_end)
    });

    // when_not() function
    let state_clone_not = Arc::clone(&state);
    engine.register_fn(
        "when_not_start",
        move |cond: &str| -> Result<(), Box<EvalAltResult>> {
            let condition =
                parse_condition_string(cond).map_err(|e| format!("Invalid condition: {}", e))?;
            let item = match condition {
                Condition::ModifierActive(id) => ConditionItem::ModifierActive(id),
                Condition::LockActive(id) => ConditionItem::LockActive(id),
                _ => return Err("Only single modifiers/locks allowed in when_not".into()),
            };
            start_conditional_block(&state_clone_not, Condition::NotActive(vec![item]))
        },
    );

    // when_not_end() - finalize when_not block
    let state_clone_not_end = Arc::clone(&state);
    engine.register_fn("when_not_end", move || -> Result<(), Box<EvalAltResult>> {
        end_conditional_block(&state_clone_not_end)
    });

    // when_device_start(pattern) - start device-specific conditional block
    //
    // Creates a conditional block that only applies to devices matching the pattern.
    // The pattern supports glob-style matching with * wildcard:
    // - Exact match: "usb-numpad-123"
    // - Prefix match: "usb-*"
    // - Suffix match: "*-keyboard"
    // - Contains: "*numpad*"
    //
    // Example:
    //   when_device_start("*numpad*");
    //   map(Numpad1, VK_F13);  // Only applies to numpad devices
    //   when_device_end();
    let state_clone_device = Arc::clone(&state);
    engine.register_fn(
        "when_device_start",
        move |pattern: &str| -> Result<(), Box<EvalAltResult>> {
            if pattern.is_empty() {
                return Err("Device pattern cannot be empty".into());
            }
            start_conditional_block(
                &state_clone_device,
                Condition::DeviceMatches(pattern.to_string()),
            )
        },
    );

    // when_device_end() - finalize device conditional block
    let state_clone_device_end = Arc::clone(&state);
    engine.register_fn(
        "when_device_end",
        move || -> Result<(), Box<EvalAltResult>> {
            end_conditional_block(&state_clone_device_end)
        },
    );
}

/// Start a conditional block - push a new conditional stack entry
fn start_conditional_block(
    state: &Arc<Mutex<ParserState>>,
    condition: Condition,
) -> Result<(), Box<EvalAltResult>> {
    // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
    #[allow(clippy::unwrap_used)]
    let mut state = state.lock().unwrap();
    if state.current_device.is_none() {
        return Err("Conditional blocks must be called inside a device() block".into());
    }

    // Push (condition, empty mappings Vec) onto the stack
    state.conditional_stack.push((condition, Vec::new()));

    Ok(())
}

/// End a conditional block - pop the stack and create the Conditional mapping
fn end_conditional_block(state: &Arc<Mutex<ParserState>>) -> Result<(), Box<EvalAltResult>> {
    // SAFETY: Mutex cannot be poisoned - no panic paths while lock is held
    #[allow(clippy::unwrap_used)]
    let mut state = state.lock().unwrap();

    let (condition, mappings) = state
        .conditional_stack
        .pop()
        .ok_or("when_end() called without matching when_start()")?;

    // Create the Conditional mapping and add it to the current device or outer conditional
    let conditional_mapping = KeyMapping::Conditional {
        condition,
        mappings,
    };

    // If there's an outer conditional block, add to it; otherwise add to current device
    if state.conditional_stack.last().is_some() {
        // We're inside a nested conditional - but wait, Conditional holds Vec<BaseKeyMapping>
        // not Vec<KeyMapping>, so we can't nest Conditionals inside Conditionals
        // This is a limitation of the current design
        Err("Nested conditional blocks are not supported".into())
    } else if let Some(ref mut device) = state.current_device {
        device.mappings.push(conditional_mapping);
        Ok(())
    } else {
        Err("Conditional blocks must be called inside a device() block".into())
    }
}
