//! Tests for transition logging functionality.

#[cfg(test)]
mod entry_tests {
    use crate::engine::state::snapshot::{PressedKey, StateSnapshot};
    use crate::engine::transitions::log::TransitionEntry;
    use crate::engine::transitions::transition::{
        DecisionKind, StateTransition, TransitionCategory,
    };
    use crate::engine::KeyCode;

    #[test]
    fn test_new_entry() {
        let transition = StateTransition::KeyPressed {
            key: KeyCode::A,
            timestamp: 1000,
        };
        let state_before = StateSnapshot::empty();
        let state_after = StateSnapshot::empty();

        let entry = TransitionEntry::new(
            transition,
            state_before.clone(),
            state_after.clone(),
            1000000,
            5000,
        );

        assert_eq!(entry.wall_time_us, 1000000);
        assert_eq!(entry.duration_ns, 5000);
        assert_eq!(entry.name(), "KeyPressed");
        assert_eq!(entry.event_timestamp(), Some(1000));
    }

    #[test]
    fn test_category() {
        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );

        assert_eq!(entry.category(), TransitionCategory::Engine);
    }

    #[test]
    fn test_version_tracking() {
        let mut state_before = StateSnapshot::empty();
        state_before.version = 5;

        let mut state_after = StateSnapshot::empty();
        state_after.version = 6;

        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        assert_eq!(entry.version_before(), 5);
        assert_eq!(entry.version_after(), 6);
        assert!(entry.changed_version());
    }

    #[test]
    fn test_no_version_change() {
        let mut state = StateSnapshot::empty();
        state.version = 5;

        let entry = TransitionEntry::new(
            StateTransition::EngineReset,
            state.clone(),
            state,
            1000000,
            5000,
        );

        assert_eq!(entry.version_before(), 5);
        assert_eq!(entry.version_after(), 5);
        assert!(!entry.changed_version());
    }

    #[test]
    fn test_state_diff_summary_no_changes() {
        let state = StateSnapshot::empty();

        let entry = TransitionEntry::new(
            StateTransition::ConfigReloaded,
            state.clone(),
            state,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 0);
        assert_eq!(layers, 0);
        assert!(!mods);
        assert!(!pending);
    }

    #[test]
    fn test_state_diff_summary_key_changes() {
        let state_before = StateSnapshot::empty();
        let state_after = StateSnapshot::with_keys(vec![PressedKey {
            key: KeyCode::A,
            pressed_at: 1000,
        }]);

        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 1);
        assert_eq!(layers, 0);
        assert!(!mods);
        assert!(!pending);
    }

    #[test]
    fn test_state_diff_summary_layer_changes() {
        let state_before = StateSnapshot::with_layers(vec![0]);
        let state_after = StateSnapshot::with_layers(vec![0, 1]);

        let entry = TransitionEntry::new(
            StateTransition::LayerPushed { layer: 1 },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 0);
        assert_eq!(layers, 1);
        assert!(!mods);
        assert!(!pending);
    }

    #[test]
    fn test_state_diff_summary_pending_changes() {
        let state_before = StateSnapshot::empty();
        let mut state_after = StateSnapshot::empty();
        state_after.pending_count = 1;

        let entry = TransitionEntry::new(
            StateTransition::DecisionQueued {
                id: 1,
                kind: DecisionKind::TapHold,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, layers, mods, pending) = entry.state_diff_summary();
        assert_eq!(keys, 0);
        assert_eq!(layers, 0);
        assert!(!mods);
        assert!(pending);
    }

    #[test]
    fn test_serialization() {
        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );

        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(json.contains("\"transition\""));
        assert!(json.contains("\"state_before\""));
        assert!(json.contains("\"state_after\""));
        assert!(json.contains("\"wall_time_us\":1000000"));
        assert!(json.contains("\"duration_ns\":5000"));

        let deserialized: TransitionEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name(), entry.name());
        assert_eq!(deserialized.wall_time_us, entry.wall_time_us);
        assert_eq!(deserialized.duration_ns, entry.duration_ns);
    }

    #[test]
    fn test_event_timestamp_none() {
        let entry = TransitionEntry::new(
            StateTransition::ConfigReloaded,
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );

        assert_eq!(entry.event_timestamp(), None);
    }

    #[test]
    fn test_multiple_key_changes() {
        let state_before = StateSnapshot::with_keys(vec![
            PressedKey {
                key: KeyCode::A,
                pressed_at: 1000,
            },
            PressedKey {
                key: KeyCode::B,
                pressed_at: 1100,
            },
        ]);

        let state_after = StateSnapshot::with_keys(vec![
            PressedKey {
                key: KeyCode::B,
                pressed_at: 1100,
            },
            PressedKey {
                key: KeyCode::C,
                pressed_at: 1200,
            },
        ]);

        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::C,
                timestamp: 1200,
            },
            state_before,
            state_after,
            1000000,
            5000,
        );

        let (keys, _, _, _) = entry.state_diff_summary();
        // A was removed, C was added = 2 changes
        assert_eq!(keys, 2);
    }
}

#[cfg(test)]
mod feature_tests {
    use crate::engine::state::snapshot::StateSnapshot;
    use crate::engine::transitions::log::{TransitionEntry, TransitionLog};
    use crate::engine::transitions::transition::{StateTransition, TransitionCategory};
    use crate::engine::KeyCode;

    #[test]
    #[cfg(feature = "transition-logging")]
    fn test_transition_log_has_storage_when_enabled() {
        // When feature is enabled, TransitionLog should have actual storage
        let log = TransitionLog::new(100);
        assert_eq!(log.capacity(), 100);
        assert_eq!(log.len(), 0);
        assert!(std::mem::size_of_val(&log) > 0);
    }

    #[test]
    #[cfg(not(feature = "transition-logging"))]
    fn test_transition_log_is_zero_sized_when_disabled() {
        // When feature is disabled, TransitionLog should be zero-sized
        let log = TransitionLog::new(100);
        assert_eq!(log.capacity(), 0);
        assert_eq!(log.len(), 0);
        // PhantomData is zero-sized
        assert_eq!(std::mem::size_of::<TransitionLog>(), 0);
    }

    #[test]
    #[cfg(not(feature = "transition-logging"))]
    fn test_stub_methods_are_no_ops() {
        let mut log = TransitionLog::new(100);

        // Push should be a no-op
        let entry = TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 1000,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            1000000,
            5000,
        );
        log.push(entry);

        // Log should remain empty
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert!(log.last().is_none());
        assert_eq!(log.total_count(), 0);

        // Search methods should return empty
        assert!(log
            .search_by_category(TransitionCategory::Engine)
            .is_empty());
        assert!(log.search_by_name("KeyPressed").is_empty());
        assert!(log.search_by_time_range(0, 1000000).is_empty());
        assert!(log.search_version_changes().is_empty());

        // Export should return empty array
        assert_eq!(log.export_json().unwrap(), "[]");
        assert_eq!(log.export_json_pretty().unwrap(), "[]");

        // Statistics should be all zeros
        let (total, unique, total_dur, avg_dur) = log.statistics();
        assert_eq!(total, 0);
        assert_eq!(unique, 0);
        assert_eq!(total_dur, 0);
        assert_eq!(avg_dur, 0);
    }
}

#[cfg(all(test, feature = "transition-logging"))]
mod transition_log_tests {
    use crate::engine::state::snapshot::StateSnapshot;
    use crate::engine::transitions::log::{TransitionEntry, TransitionLog};
    use crate::engine::transitions::transition::{StateTransition, TransitionCategory};
    use crate::engine::KeyCode;

    fn create_test_entry(key: KeyCode, timestamp: u64, wall_time_us: u64) -> TransitionEntry {
        TransitionEntry::new(
            StateTransition::KeyPressed { key, timestamp },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            wall_time_us,
            5000,
        )
    }

    #[test]
    fn test_new_log() {
        let log = TransitionLog::new(100);
        assert_eq!(log.capacity(), 100);
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
        assert_eq!(log.total_count(), 0);
        assert!(!log.has_wrapped());
    }

    #[test]
    #[should_panic(expected = "TransitionLog capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        TransitionLog::new(0);
    }

    #[test]
    fn test_push_single_entry() {
        let mut log = TransitionLog::new(10);
        let entry = create_test_entry(KeyCode::A, 1000, 1000000);

        log.push(entry);

        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
        assert_eq!(log.total_count(), 1);
        assert!(!log.has_wrapped());
    }

    #[test]
    fn test_push_multiple_entries() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        assert_eq!(log.len(), 5);
        assert_eq!(log.total_count(), 5);
        assert!(!log.has_wrapped());
    }

    #[test]
    fn test_ring_buffer_wrap() {
        let mut log = TransitionLog::new(3);

        // Add 5 entries to a capacity-3 log
        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        assert_eq!(log.len(), 3); // Still only 3 entries
        assert_eq!(log.total_count(), 5); // But 5 total added
        assert!(log.has_wrapped());
    }

    #[test]
    fn test_iter_order_before_wrap() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries.len(), 5);

        // Check chronological order
        for i in 0..5 {
            assert_eq!(entries[i].wall_time_us, i as u64 * 100000);
        }
    }

    #[test]
    fn test_iter_order_after_wrap() {
        let mut log = TransitionLog::new(3);

        // Add 5 entries (indices 0-4)
        for i in 0..5 {
            let entry = create_test_entry(KeyCode::A, i * 100, i * 100000);
            log.push(entry);
        }

        // Log should contain entries 2, 3, 4 in chronological order
        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].wall_time_us, 200000); // entry 2
        assert_eq!(entries[1].wall_time_us, 300000); // entry 3
        assert_eq!(entries[2].wall_time_us, 400000); // entry 4
    }

    #[test]
    fn test_last_entry() {
        let mut log = TransitionLog::new(10);

        assert!(log.last().is_none());

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        assert_eq!(log.last().unwrap().wall_time_us, 100000);

        log.push(create_test_entry(KeyCode::B, 200, 200000));
        assert_eq!(log.last().unwrap().wall_time_us, 200000);
    }

    #[test]
    fn test_last_entry_after_wrap() {
        let mut log = TransitionLog::new(3);

        for i in 0..5 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        // Last entry should be entry 4
        assert_eq!(log.last().unwrap().wall_time_us, 400000);
    }

    #[test]
    fn test_clear() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        log.clear();

        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
        assert!(log.last().is_none());
        // Note: total_count is NOT reset by clear
    }

    #[test]
    fn test_search_by_category() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(TransitionEntry::new(
            StateTransition::ConfigReloaded,
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            5000,
        ));
        log.push(create_test_entry(KeyCode::B, 300, 300000));

        let engine_entries = log.search_by_category(TransitionCategory::Engine);
        assert_eq!(engine_entries.len(), 2); // 2 KeyPressed events

        let system_entries = log.search_by_category(TransitionCategory::System);
        assert_eq!(system_entries.len(), 1); // 1 ConfigReloaded event
    }

    #[test]
    fn test_search_by_name() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(TransitionEntry::new(
            StateTransition::ConfigReloaded,
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            5000,
        ));
        log.push(create_test_entry(KeyCode::B, 300, 300000));

        let key_pressed = log.search_by_name("KeyPressed");
        assert_eq!(key_pressed.len(), 2);

        let config_reloaded = log.search_by_name("ConfigReloaded");
        assert_eq!(config_reloaded.len(), 1);
    }

    #[test]
    fn test_search_by_time_range() {
        let mut log = TransitionLog::new(10);

        for i in 0..5 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        // Search for entries between 150000 and 350000
        let results = log.search_by_time_range(150000, 350000);
        assert_eq!(results.len(), 2); // entries at 200000 and 300000
        assert_eq!(results[0].wall_time_us, 200000);
        assert_eq!(results[1].wall_time_us, 300000);
    }

    #[test]
    fn test_search_version_changes() {
        let mut log = TransitionLog::new(10);

        // Entry with version change
        let mut state_before1 = StateSnapshot::empty();
        state_before1.version = 1;
        let mut state_after1 = StateSnapshot::empty();
        state_after1.version = 2;

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 100,
            },
            state_before1,
            state_after1,
            100000,
            5000,
        ));

        // Entry without version change
        let state = StateSnapshot::empty();
        log.push(TransitionEntry::new(
            StateTransition::EngineReset,
            state.clone(),
            state,
            200000,
            5000,
        ));

        let version_changes = log.search_version_changes();
        assert_eq!(version_changes.len(), 1);
        assert_eq!(version_changes[0].wall_time_us, 100000);
    }

    #[test]
    fn test_search_custom_predicate() {
        let mut log = TransitionLog::new(10);

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 100,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            100000,
            500_000, // 0.5ms
        ));

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::B,
                timestamp: 200,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            2_000_000, // 2ms (slow)
        ));

        // Find slow transitions (> 1ms)
        let slow = log.search(|entry| entry.duration_ns > 1_000_000);
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].wall_time_us, 200000);
    }

    #[test]
    fn test_export_json() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(create_test_entry(KeyCode::B, 200, 200000));

        let json = log.export_json().expect("export failed");
        assert!(json.contains("KeyPressed"));
        assert!(json.contains("100000"));
        assert!(json.contains("200000"));

        // Verify it's valid JSON
        let parsed: Vec<TransitionEntry> = serde_json::from_str(&json).expect("invalid JSON");
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_export_json_pretty() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));

        let json = log.export_json_pretty().expect("export failed");
        assert!(json.contains('\n')); // Pretty printing adds newlines
        assert!(json.contains("  ")); // Pretty printing adds indentation
    }

    #[test]
    fn test_export_entries_json() {
        let mut log = TransitionLog::new(10);

        log.push(create_test_entry(KeyCode::A, 100, 100000));
        log.push(create_test_entry(KeyCode::B, 200, 200000));
        log.push(create_test_entry(KeyCode::C, 300, 300000));

        // Search for specific entries
        let filtered = log.search_by_time_range(150000, 300000);
        assert_eq!(filtered.len(), 2);

        // Export only filtered entries
        let json = TransitionLog::export_entries_json(&filtered).expect("export failed");

        let parsed: Vec<TransitionEntry> = serde_json::from_str(&json).expect("invalid JSON");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].wall_time_us, 200000);
        assert_eq!(parsed[1].wall_time_us, 300000);
    }

    #[test]
    fn test_statistics_empty() {
        let log = TransitionLog::new(10);

        let (total, unique_names, total_duration, avg_duration) = log.statistics();
        assert_eq!(total, 0);
        assert_eq!(unique_names, 0);
        assert_eq!(total_duration, 0);
        assert_eq!(avg_duration, 0);
    }

    #[test]
    fn test_statistics_with_data() {
        let mut log = TransitionLog::new(10);

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::A,
                timestamp: 100,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            100000,
            1000, // 1000ns
        ));

        log.push(TransitionEntry::new(
            StateTransition::KeyPressed {
                key: KeyCode::B,
                timestamp: 200,
            },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            200000,
            3000, // 3000ns
        ));

        log.push(TransitionEntry::new(
            StateTransition::LayerPushed { layer: 1 },
            StateSnapshot::empty(),
            StateSnapshot::empty(),
            300000,
            2000, // 2000ns
        ));

        let (total, unique_names, total_duration, avg_duration) = log.statistics();
        assert_eq!(total, 3);
        assert_eq!(unique_names, 2); // KeyPressed and LayerPushed
        assert_eq!(total_duration, 6000); // 1000 + 3000 + 2000
        assert_eq!(avg_duration, 2000); // 6000 / 3
    }

    #[test]
    fn test_default() {
        let log = TransitionLog::default();
        assert_eq!(log.capacity(), 10_000);
        assert!(log.is_empty());
    }

    #[test]
    fn test_ring_buffer_full_cycle() {
        let mut log = TransitionLog::new(3);

        // Fill the buffer completely
        for i in 0..3 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        assert_eq!(log.len(), 3);
        assert_eq!(log.total_count(), 3);
        assert!(!log.has_wrapped());

        // Add one more to trigger wrap
        log.push(create_test_entry(KeyCode::B, 300, 300000));

        assert_eq!(log.len(), 3);
        assert_eq!(log.total_count(), 4);
        assert!(log.has_wrapped());

        // Verify oldest entry was overwritten
        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries[0].wall_time_us, 100000); // entry 1 (entry 0 was overwritten)
        assert_eq!(entries[1].wall_time_us, 200000); // entry 2
        assert_eq!(entries[2].wall_time_us, 300000); // entry 3
    }

    #[test]
    fn test_multiple_wraps() {
        let mut log = TransitionLog::new(3);

        // Add many entries (multiple wraps)
        for i in 0..10 {
            log.push(create_test_entry(KeyCode::A, i * 100, i * 100000));
        }

        assert_eq!(log.len(), 3);
        assert_eq!(log.total_count(), 10);
        assert!(log.has_wrapped());

        // Should contain the last 3 entries (7, 8, 9)
        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].wall_time_us, 700000);
        assert_eq!(entries[1].wall_time_us, 800000);
        assert_eq!(entries[2].wall_time_us, 900000);
    }
}
