//! Runtime API Usage Example
//!
//! This example demonstrates how to use keyrx's core runtime components:
//! - DeviceState: Track modifier and lock state
//! - KeyLookup: Build lookup tables from configuration
//! - KeyEvent: Represent keyboard events
//! - process_event(): Process events through the runtime
//! - EventProcessor: Complete event processing pipeline with mock devices
//!
//! Run with: cargo run --example runtime_example -p keyrx_daemon

use keyrx_core::config::{
    BaseKeyMapping, Condition, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};
use keyrx_core::runtime::event::{process_event, KeyEvent};
use keyrx_core::runtime::lookup::KeyLookup;
use keyrx_core::runtime::state::DeviceState;
use keyrx_daemon::platform::mock::{MockInput, MockOutput};
use keyrx_daemon::processor::EventProcessor;

fn main() {
    println!("=== KeyRx Runtime API Example ===\n");

    // Example 1: Basic state management
    println!("--- Example 1: DeviceState ---");
    example_device_state();

    // Example 2: Key lookup table
    println!("\n--- Example 2: KeyLookup ---");
    example_key_lookup();

    // Example 3: Event processing
    println!("\n--- Example 3: Event Processing ---");
    example_event_processing();

    // Example 4: Complete event processor pipeline
    println!("\n--- Example 4: EventProcessor Pipeline ---");
    example_event_processor();

    println!("\n=== All Examples Complete ===");
}

/// Example 1: DeviceState - Managing modifier and lock state
fn example_device_state() {
    let mut state = DeviceState::new();
    println!("Created fresh DeviceState (all bits cleared)");

    // Set a modifier (e.g., MD_00)
    state.set_modifier(0);
    println!("Set modifier MD_00");
    assert!(state.is_modifier_active(0));
    println!("  is_modifier_active(0) = {}", state.is_modifier_active(0));

    // Toggle a lock (e.g., LK_01)
    state.toggle_lock(1);
    println!("Toggle lock LK_01 (OFF → ON)");
    assert!(state.is_lock_active(1));
    println!("  is_lock_active(1) = {}", state.is_lock_active(1));

    // Toggle again to turn off
    state.toggle_lock(1);
    println!("Toggle lock LK_01 (ON → OFF)");
    assert!(!state.is_lock_active(1));
    println!("  is_lock_active(1) = {}", state.is_lock_active(1));

    // Evaluate a condition
    let condition = Condition::ModifierActive(0);
    let result = state.evaluate_condition(&condition);
    println!("Evaluate Condition::ModifierActive(0) = {}", result);
    assert!(result);

    // Clear the modifier
    state.clear_modifier(0);
    println!("Clear modifier MD_00");
    assert!(!state.is_modifier_active(0));
    println!("  is_modifier_active(0) = {}", state.is_modifier_active(0));
}

/// Example 2: KeyLookup - Building lookup tables from config
fn example_key_lookup() {
    // Create a simple device config with a few mappings
    let mappings = vec![
        // Simple: A → B
        KeyMapping::Base(BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::B,
        }),
        // Modifier: CapsLock → MD_00
        KeyMapping::Base(BaseKeyMapping::Modifier {
            from: KeyCode::CapsLock,
            modifier_id: 0,
        }),
        // Conditional: When MD_00 is active, H → Left
        KeyMapping::Conditional {
            condition: Condition::ModifierActive(0),
            mappings: vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        },
        // Unconditional fallback: H → H (passthrough)
        KeyMapping::Base(BaseKeyMapping::Simple {
            from: KeyCode::H,
            to: KeyCode::H,
        }),
    ];

    let config = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: ".*".to_string(),
        },
        mappings,
    };

    // Build the lookup table
    let lookup = KeyLookup::from_device_config(&config);
    println!("Built KeyLookup from DeviceConfig with 4 mappings");

    // Test finding mappings
    let mut state = DeviceState::new();

    // Find simple mapping (A → B)
    let mapping = lookup.find_mapping(KeyCode::A, &state);
    println!("find_mapping(A, state) = {:?}", mapping.is_some());
    assert!(mapping.is_some());

    // Find conditional mapping with condition FALSE (H → H fallback)
    let mapping = lookup.find_mapping(KeyCode::H, &state);
    println!("find_mapping(H, state[MD_00=OFF]) = Simple(H→H) fallback");
    if let Some(BaseKeyMapping::Simple { from, to }) = mapping {
        println!("  from={:?}, to={:?}", from, to);
        assert_eq!(*from, KeyCode::H);
        assert_eq!(*to, KeyCode::H);
    }

    // Find conditional mapping with condition TRUE (H → Left)
    state.set_modifier(0);
    let mapping = lookup.find_mapping(KeyCode::H, &state);
    println!("find_mapping(H, state[MD_00=ON]) = Simple(H→Left) conditional");
    if let Some(BaseKeyMapping::Simple { from, to }) = mapping {
        println!("  from={:?}, to={:?}", from, to);
        assert_eq!(*from, KeyCode::H);
        assert_eq!(*to, KeyCode::Left);
    }
}

/// Example 3: Event Processing - Using process_event()
fn example_event_processing() {
    // Create config and lookup table
    let mappings = vec![
        KeyMapping::Base(BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::B,
        }),
        KeyMapping::Base(BaseKeyMapping::Modifier {
            from: KeyCode::CapsLock,
            modifier_id: 0,
        }),
        KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
            from: KeyCode::Num1,
            to: KeyCode::Num1,
            shift: true,
            ctrl: false,
            alt: false,
            win: false,
        }),
    ];

    let config = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: ".*".to_string(),
        },
        mappings,
    };

    let lookup = KeyLookup::from_device_config(&config);
    let mut state = DeviceState::new();

    // Test 1: Simple mapping (A → B)
    println!("Processing Press(A) with simple mapping:");
    let input = KeyEvent::Press(KeyCode::A);
    let output = process_event(input.clone(), &lookup, &mut state);
    println!("  Input: {:?}", input);
    println!("  Output: {:?}", output);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::B));

    // Test 2: Modifier mapping (CapsLock → MD_00, no output)
    println!("Processing Press(CapsLock) with modifier mapping:");
    let input = KeyEvent::Press(KeyCode::CapsLock);
    let output = process_event(input.clone(), &lookup, &mut state);
    println!("  Input: {:?}", input);
    println!("  Output: {:?} (no output, state updated)", output);
    assert_eq!(output.len(), 0);
    assert!(state.is_modifier_active(0));

    // Test 3: ModifiedOutput (Shift+1)
    println!("Processing Press(Num1) with ModifiedOutput (Shift+1):");
    let input = KeyEvent::Press(KeyCode::Num1);
    let output = process_event(input.clone(), &lookup, &mut state);
    println!("  Input: {:?}", input);
    println!("  Output: {:?}", output);
    assert_eq!(output.len(), 2);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::LShift));
    assert_eq!(output[1], KeyEvent::Press(KeyCode::Num1));

    // Test 4: Passthrough (unmapped key)
    println!("Processing Press(Z) with no mapping (passthrough):");
    let input = KeyEvent::Press(KeyCode::Z);
    let output = process_event(input.clone(), &lookup, &mut state);
    println!("  Input: {:?}", input);
    println!("  Output: {:?}", output);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], KeyEvent::Press(KeyCode::Z));
}

/// Example 4: EventProcessor - Complete pipeline with mock devices
fn example_event_processor() {
    // Create a Vim navigation layer config
    // CapsLock → MD_00
    // When MD_00: H→Left, J→Down, K→Up, L→Right
    let mappings = vec![
        KeyMapping::Base(BaseKeyMapping::Modifier {
            from: KeyCode::CapsLock,
            modifier_id: 0,
        }),
        KeyMapping::Conditional {
            condition: Condition::ModifierActive(0),
            mappings: vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        },
        KeyMapping::Conditional {
            condition: Condition::ModifierActive(0),
            mappings: vec![BaseKeyMapping::Simple {
                from: KeyCode::J,
                to: KeyCode::Down,
            }],
        },
        KeyMapping::Conditional {
            condition: Condition::ModifierActive(0),
            mappings: vec![BaseKeyMapping::Simple {
                from: KeyCode::K,
                to: KeyCode::Up,
            }],
        },
        KeyMapping::Conditional {
            condition: Condition::ModifierActive(0),
            mappings: vec![BaseKeyMapping::Simple {
                from: KeyCode::L,
                to: KeyCode::Right,
            }],
        },
    ];

    let config = DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: ".*".to_string(),
        },
        mappings,
    };

    // Create mock input with test events
    let input_events = vec![
        KeyEvent::Press(KeyCode::CapsLock),   // Activate MD_00
        KeyEvent::Press(KeyCode::H),          // Should output Left
        KeyEvent::Release(KeyCode::H),        // Should output Left release
        KeyEvent::Press(KeyCode::J),          // Should output Down
        KeyEvent::Release(KeyCode::J),        // Should output Down release
        KeyEvent::Release(KeyCode::CapsLock), // Deactivate MD_00
    ];
    let input = MockInput::new(input_events);
    let output = MockOutput::new();

    // Create event processor
    let mut processor = EventProcessor::new(&config, input, output);
    println!("Created EventProcessor with Vim navigation config");

    // Process all events
    println!("Processing input events...");
    processor.run().expect("Failed to run processor");

    // Verify output
    let output_events = processor.output().events();
    println!("Output events: {} events generated", output_events.len());
    for (i, event) in output_events.iter().enumerate() {
        println!("  [{}] {:?}", i, event);
    }

    // Expected output:
    // CapsLock press/release → no output (modifier)
    // H press → Left press
    // H release → Left release
    // J press → Down press
    // J release → Down release
    assert_eq!(output_events.len(), 4);
    assert_eq!(output_events[0], KeyEvent::Press(KeyCode::Left));
    assert_eq!(output_events[1], KeyEvent::Release(KeyCode::Left));
    assert_eq!(output_events[2], KeyEvent::Press(KeyCode::Down));
    assert_eq!(output_events[3], KeyEvent::Release(KeyCode::Down));

    println!("✓ Vim navigation layer working correctly!");
}
