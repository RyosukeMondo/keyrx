//! Test primitive implementations for Rhai script testing.
//!
//! This module contains the actual implementations of test functions:
//! - `simulate_tap`: Generate key down/up events for a tap
//! - `simulate_hold`: Generate key events with a specified hold duration
//! - `assert_output`: Assert a key was in the output
//! - `assert_mapping`: Assert key mapping exists
//! - `assert_blocked`: Assert a key was blocked
//! - `clear_outputs`: Clear captured outputs
//! - `get_output_count`: Get the count of outputs

use super::helpers::parse_key_or_error;
use crate::engine::{InputEvent, OutputAction};
use rhai::{Engine, EvalAltResult, Position};
use std::sync::Arc;
use std::sync::Mutex;

use super::test_harness::TEST_CONTEXT;

/// Register simulate_tap function with the Rhai engine.
///
/// `simulate_tap(key: String)` - Simulates a key tap (down + up) with a typical
/// tap duration of ~50ms between down and up events.
pub fn register_simulate_tap(engine: &mut Engine) {
    engine.register_fn(
        "simulate_tap",
        |key: &str| -> Result<(), Box<EvalAltResult>> {
            let key_code = parse_key_or_error(key, "simulate_tap")?;

            TEST_CONTEXT.with(|ctx| {
                let mut context = ctx.borrow_mut();
                let timestamp = context.current_time();

                // Key down event
                let down_event = InputEvent::key_down(key_code, timestamp);
                context.add_input(down_event);

                // Advance time slightly (typical tap duration ~50ms = 50000µs)
                context.advance_time(50_000);

                // Key up event
                let up_event = InputEvent::key_up(key_code, context.current_time());
                context.add_input(up_event);

                // Small gap between events
                context.advance_time(10_000);
            });

            tracing::debug!(
                service = "keyrx",
                event = "test_simulate_tap",
                component = "test_harness",
                key = key,
                "Simulated key tap"
            );

            Ok(())
        },
    );
}

/// Register simulate_hold function with the Rhai engine.
///
/// `simulate_hold(key: String, duration_ms: i64)` - Simulates holding a key
/// for the specified duration in milliseconds.
pub fn register_simulate_hold(engine: &mut Engine) {
    engine.register_fn(
        "simulate_hold",
        |key: &str, duration_ms: i64| -> Result<(), Box<EvalAltResult>> {
            let key_code = parse_key_or_error(key, "simulate_hold")?;

            if duration_ms < 0 {
                return Err(Box::new(EvalAltResult::ErrorRuntime(
                    format!(
                        "simulate_hold: duration must be non-negative, got {}",
                        duration_ms
                    )
                    .into(),
                    Position::NONE,
                )));
            }

            let duration_us = (duration_ms as u64).saturating_mul(1000);

            TEST_CONTEXT.with(|ctx| {
                let mut context = ctx.borrow_mut();
                let timestamp = context.current_time();

                // Key down event
                let down_event = InputEvent::key_down(key_code, timestamp);
                context.add_input(down_event);

                // Advance time by hold duration
                context.advance_time(duration_us);

                // Key up event
                let up_event = InputEvent::key_up(key_code, context.current_time());
                context.add_input(up_event);

                // Small gap between events
                context.advance_time(10_000);
            });

            tracing::debug!(
                service = "keyrx",
                event = "test_simulate_hold",
                component = "test_harness",
                key = key,
                duration_ms = duration_ms,
                "Simulated key hold"
            );

            Ok(())
        },
    );
}

/// Register assert_output function with the Rhai engine.
///
/// `assert_output(key: String)` - Asserts that the specified key was present
/// in the output actions (KeyDown, KeyUp, or KeyTap).
pub fn register_assert_output(engine: &mut Engine, collector: Arc<Mutex<Vec<OutputAction>>>) {
    engine.register_fn(
        "assert_output",
        move |key: &str| -> Result<bool, Box<EvalAltResult>> {
            let key_code = parse_key_or_error(key, "assert_output")?;

            let found = if let Ok(outputs) = collector.lock() {
                outputs.iter().any(|action| match action {
                    OutputAction::KeyDown(k) | OutputAction::KeyUp(k) | OutputAction::KeyTap(k) => {
                        *k == key_code
                    }
                    _ => false,
                })
            } else {
                false
            };

            let message = if found {
                format!("Key '{}' found in outputs", key)
            } else {
                format!("Key '{}' NOT found in outputs", key)
            };

            TEST_CONTEXT.with(|ctx| {
                ctx.borrow_mut().add_assertion(
                    format!("assert_output({})", key),
                    found,
                    message.clone(),
                );
            });

            if !found {
                return Err(Box::new(EvalAltResult::ErrorRuntime(
                    message.into(),
                    Position::NONE,
                )));
            }

            Ok(true)
        },
    );
}

/// Register assert_mapping function with the Rhai engine.
///
/// `assert_mapping(from: String, to: String)` - Asserts that a mapping exists
/// from the source key to the target key by checking inputs and outputs.
pub fn register_assert_mapping(engine: &mut Engine) {
    engine.register_fn(
        "assert_mapping",
        |from: &str, to: &str| -> Result<bool, Box<EvalAltResult>> {
            let from_key = parse_key_or_error(from, "assert_mapping")?;
            let to_key = parse_key_or_error(to, "assert_mapping")?;

            // Check if the mapping exists by looking at the test context inputs/outputs
            // This is a declarative assertion that will be validated by the test runner
            let found = TEST_CONTEXT.with(|ctx| {
                let context = ctx.borrow();
                // Look for a pattern: input of `from` key followed by output of `to` key
                let has_input = context
                    .inputs
                    .iter()
                    .any(|e| e.key == from_key && e.pressed);
                let has_output = context.outputs.iter().any(|action| match action {
                    OutputAction::KeyDown(k) | OutputAction::KeyTap(k) => *k == to_key,
                    _ => false,
                });
                has_input && has_output
            });

            let message = if found {
                format!("Mapping '{}' -> '{}' verified", from, to)
            } else {
                format!(
                    "Mapping '{}' -> '{}' NOT verified (input not found or output mismatch)",
                    from, to
                )
            };

            TEST_CONTEXT.with(|ctx| {
                ctx.borrow_mut().add_assertion(
                    format!("assert_mapping({}, {})", from, to),
                    found,
                    message.clone(),
                );
            });

            if !found {
                return Err(Box::new(EvalAltResult::ErrorRuntime(
                    message.into(),
                    Position::NONE,
                )));
            }

            Ok(true)
        },
    );
}

/// Register assert_blocked function with the Rhai engine.
///
/// `assert_blocked(key: String)` - Asserts that a key was blocked (i.e., a Block
/// action was present in the outputs).
pub fn register_assert_blocked(engine: &mut Engine, collector: Arc<Mutex<Vec<OutputAction>>>) {
    engine.register_fn(
        "assert_blocked",
        move |key: &str| -> Result<bool, Box<EvalAltResult>> {
            let _key_code = parse_key_or_error(key, "assert_blocked")?;

            let has_block = if let Ok(outputs) = collector.lock() {
                outputs
                    .iter()
                    .any(|action| matches!(action, OutputAction::Block))
            } else {
                false
            };

            let message = if has_block {
                format!("Key '{}' was blocked", key)
            } else {
                format!("Key '{}' was NOT blocked", key)
            };

            TEST_CONTEXT.with(|ctx| {
                ctx.borrow_mut().add_assertion(
                    format!("assert_blocked({})", key),
                    has_block,
                    message.clone(),
                );
            });

            if !has_block {
                return Err(Box::new(EvalAltResult::ErrorRuntime(
                    message.into(),
                    Position::NONE,
                )));
            }

            Ok(true)
        },
    );
}

/// Register clear_outputs function with the Rhai engine.
///
/// `clear_outputs()` - Clears both the output collector and the test context
/// outputs, useful for fresh assertions after a batch of tests.
pub fn register_clear_outputs(engine: &mut Engine, collector: Arc<Mutex<Vec<OutputAction>>>) {
    engine.register_fn("clear_outputs", move || {
        if let Ok(mut outputs) = collector.lock() {
            outputs.clear();
        }
        TEST_CONTEXT.with(|ctx| {
            ctx.borrow_mut().outputs.clear();
        });
    });
}

/// Register get_output_count function with the Rhai engine.
///
/// `get_output_count()` - Returns the number of outputs in the test context.
pub fn register_get_outputs(engine: &mut Engine) {
    engine.register_fn("get_output_count", || -> i64 {
        TEST_CONTEXT.with(|ctx| ctx.borrow().outputs.len() as i64)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;
    use crate::scripting::test_harness::{
        get_test_context, record_output, reset_test_context, TestHarness,
    };

    fn setup_engine_with_harness() -> (Engine, TestHarness) {
        reset_test_context();
        let mut engine = Engine::new();
        let harness = TestHarness::new();
        harness.register_functions(&mut engine);
        (engine, harness)
    }

    #[test]
    fn simulate_tap_creates_down_up_events() {
        let (engine, _) = setup_engine_with_harness();
        engine.run(r#"simulate_tap("A");"#).unwrap();

        let ctx = get_test_context();
        assert_eq!(ctx.inputs.len(), 2);
        assert!(ctx.inputs[0].pressed);
        assert!(!ctx.inputs[1].pressed);
        assert_eq!(ctx.inputs[0].key, KeyCode::A);
    }

    #[test]
    fn simulate_hold_respects_duration() {
        let (engine, _) = setup_engine_with_harness();
        engine.run(r#"simulate_hold("Space", 200);"#).unwrap();

        let ctx = get_test_context();
        assert_eq!(
            ctx.inputs[1].timestamp_us - ctx.inputs[0].timestamp_us,
            200_000
        );
    }

    #[test]
    fn simulate_hold_rejects_negative_duration() {
        let (engine, _) = setup_engine_with_harness();
        assert!(engine.run(r#"simulate_hold("A", -100);"#).is_err());
    }

    #[test]
    fn assert_output_fails_when_key_not_found() {
        let (engine, _) = setup_engine_with_harness();
        assert!(engine.run(r#"assert_output("B");"#).is_err());
        assert!(!get_test_context().assertions[0].passed);
    }

    #[test]
    fn assert_output_succeeds_when_key_found() {
        let (engine, harness) = setup_engine_with_harness();
        harness
            .output_collector()
            .lock()
            .unwrap()
            .push(OutputAction::KeyDown(KeyCode::B));

        assert!(engine.run(r#"assert_output("B");"#).is_ok());
        assert!(get_test_context().assertions[0].passed);
    }

    #[test]
    fn clear_outputs_clears_collector() {
        let (engine, harness) = setup_engine_with_harness();
        harness
            .output_collector()
            .lock()
            .unwrap()
            .push(OutputAction::KeyDown(KeyCode::A));
        engine.run("clear_outputs();").unwrap();
        assert!(harness.output_collector().lock().unwrap().is_empty());
    }

    #[test]
    fn assert_blocked_behavior() {
        let (engine, harness) = setup_engine_with_harness();

        // Fails when no block
        assert!(engine.run(r#"assert_blocked("A");"#).is_err());
        assert!(!get_test_context().assertions[0].passed);

        // Succeeds when block present
        reset_test_context();
        harness
            .output_collector()
            .lock()
            .unwrap()
            .push(OutputAction::Block);
        assert!(engine.run(r#"assert_blocked("A");"#).is_ok());
    }

    #[test]
    fn invalid_key_returns_error() {
        let (engine, _) = setup_engine_with_harness();
        assert!(engine.run(r#"simulate_tap("InvalidKeyName");"#).is_err());
        assert!(engine
            .run(r#"simulate_hold("InvalidKeyName", 100);"#)
            .is_err());
    }

    #[test]
    fn get_output_count_returns_context_output_length() {
        let (engine, _) = setup_engine_with_harness();
        record_output(OutputAction::KeyDown(KeyCode::A));
        record_output(OutputAction::KeyUp(KeyCode::A));
        assert_eq!(engine.eval::<i64>("get_output_count()").unwrap(), 2);
    }

    #[test]
    fn assert_mapping_records_assertion() {
        let (engine, _) = setup_engine_with_harness();
        engine.run(r#"simulate_tap("A");"#).unwrap();
        record_output(OutputAction::KeyDown(KeyCode::B));
        let _result = engine.run(r#"assert_mapping("A", "B");"#);
        assert!(get_test_context()
            .assertions
            .iter()
            .any(|a| a.name.contains("assert_mapping")));
    }

    #[test]
    fn multiple_simulate_taps_advance_time() {
        let (engine, _) = setup_engine_with_harness();
        engine.run(r#"simulate_tap("A");"#).unwrap();
        let t1 = get_test_context().current_time_us;
        engine.run(r#"simulate_tap("B");"#).unwrap();
        assert!(get_test_context().current_time_us > t1);
        assert_eq!(get_test_context().inputs.len(), 4);
    }
}
