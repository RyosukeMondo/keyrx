//! Conditional functions for Rhai DSL.
//!
//! Provides when_start(), when_end(), when_not_start(), when_not_end(),
//! when_device_start(), when_device_end() functions.

use crate::config::{Condition, ConditionItem, KeyMapping};
use crate::parser::state::ParserState;
use crate::parser::validators::parse_condition_string;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use rhai::{Array, Engine, EvalAltResult};
use spin::Mutex;

/// Register conditional functions with the Rhai engine.
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
                    Condition::ImeActive => condition_items.push(ConditionItem::ImeActive),
                    Condition::InputLanguage(lang) => {
                        condition_items.push(ConditionItem::InputLanguage(lang))
                    }
                    _ => return Err("Only single conditions allowed in array".into()),
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

    // when_not_start() function
    let state_clone_not = Arc::clone(&state);
    engine.register_fn(
        "when_not_start",
        move |cond: &str| -> Result<(), Box<EvalAltResult>> {
            let condition =
                parse_condition_string(cond).map_err(|e| format!("Invalid condition: {}", e))?;
            let item = match condition {
                Condition::ModifierActive(id) => ConditionItem::ModifierActive(id),
                Condition::LockActive(id) => ConditionItem::LockActive(id),
                Condition::ImeActive => ConditionItem::ImeActive,
                Condition::InputLanguage(lang) => ConditionItem::InputLanguage(lang),
                _ => return Err("Only single conditions allowed in when_not".into()),
            };
            start_conditional_block(&state_clone_not, Condition::NotActive(alloc::vec![item]))
        },
    );

    // when_not_end() - finalize when_not block
    let state_clone_not_end = Arc::clone(&state);
    engine.register_fn("when_not_end", move || -> Result<(), Box<EvalAltResult>> {
        end_conditional_block(&state_clone_not_end)
    });

    // when_device_start(pattern) - start device-specific conditional block
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
    let mut state = state.lock();
    if state.current_device.is_none() {
        return Err("Conditional blocks must be called inside a device_start() block".into());
    }

    // Push (condition, empty mappings Vec) onto the stack
    state.conditional_stack.push((condition, Vec::new()));

    Ok(())
}

/// End a conditional block - pop the stack and create the Conditional mapping
fn end_conditional_block(state: &Arc<Mutex<ParserState>>) -> Result<(), Box<EvalAltResult>> {
    let mut state = state.lock();

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
        // Nested conditionals not supported
        Err("Nested conditional blocks are not supported".into())
    } else if let Some(ref mut device) = state.current_device {
        device.mappings.push(conditional_mapping);
        Ok(())
    } else {
        Err("Conditional blocks must be called inside a device_start() block".into())
    }
}
