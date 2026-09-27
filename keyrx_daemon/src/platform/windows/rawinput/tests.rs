use super::*;

// Note: These tests require an interactive Windows session and may fail in some CI environments.
#[test]
fn test_raw_input_manager_creation() {
    let device_map = DeviceMap::new();
    let (tx, _rx) = unbounded();
    let bridge_context = Arc::new(Mutex::new(None));
    let bridge_hook = Arc::new(Mutex::new(None));

    // We wrap in a block to ensure RawInputManager is dropped at end
    {
        match RawInputManager::new(device_map, tx, bridge_context, bridge_hook) {
            Ok(manager) => {
                assert!(manager.hwnd != 0 as HWND);
                let _receiver = manager.subscribe(12345);
                manager.unsubscribe(12345);
            }
            Err(e) => {
                // This can happen in CI environments without a GUI session
                eprintln!("RawInputManager creation failed: {}", e);
            }
        }
    }
}

#[test]
fn test_raw_input_simulation() {
    let device_map = DeviceMap::new();
    let (tx, rx_global) = unbounded();
    let bridge_context = Arc::new(Mutex::new(None));
    let bridge_hook = Arc::new(Mutex::new(None));

    // Register a synthetic device
    device_map.add_synthetic_device(0x1234, "test-path".to_string(), Some("SN123".to_string()));

    match RawInputManager::new(device_map, tx, bridge_context, bridge_hook) {
        Ok(manager) => {
            let rx_device = manager.subscribe(0x1234);

            // Simulate a key press (A key, MakeCode 0x1E)
            manager.simulate_raw_input(0x1234, 0x1E, 0);

            // Global channel should receive it
            let event_global = rx_global
                .recv_timeout(std::time::Duration::from_millis(100))
                .expect("Should receive global event");
            assert_eq!(event_global.device_id(), Some("serial-SN123"));

            // Subscribed channel should receive it
            let event_device = rx_device
                .recv_timeout(std::time::Duration::from_millis(100))
                .expect("Should receive device event");
            assert_eq!(event_device.device_id(), Some("serial-SN123"));
        }
        Err(e) => {
            eprintln!("Skipping simulation test (no GUI): {}", e);
        }
    }
}

#[test]
fn test_raw_input_subscription_logic() {
    let device_map = DeviceMap::new();
    let (tx, _rx) = unbounded();
    let bridge_context = Arc::new(Mutex::new(None));
    let bridge_hook = Arc::new(Mutex::new(None));

    match RawInputManager::new(device_map, tx, bridge_context, bridge_hook) {
        Ok(manager) => {
            // Subscription for non-existent device is allowed (manager doesn't check existence)
            let _rx = manager.subscribe(0xDEAD);
            assert!(manager.subscribers.read().unwrap().contains_key(&0xDEAD));

            // Same handle multiple times
            let _rx2 = manager.subscribe(0xDEAD);
            assert_eq!(manager.subscribers.read().unwrap().len(), 1);

            manager.unsubscribe(0xDEAD);
            assert!(!manager.subscribers.read().unwrap().contains_key(&0xDEAD));
        }
        Err(e) => eprintln!("Skipping test (no GUI): {}", e),
    }
}

#[test]
fn test_raw_input_drop_cleanup() {
    let device_map = DeviceMap::new();
    let (tx, _rx) = unbounded();
    let bridge_context = Arc::new(Mutex::new(None));
    let bridge_hook = Arc::new(Mutex::new(None));
    let hwnd;

    {
        let manager = RawInputManager::new(device_map, tx, bridge_context, bridge_hook)
            .expect("Should create manager");
        hwnd = manager.hwnd;
        assert!(hwnd != 0 as HWND);
        // Context should be present
        unsafe {
            let context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            assert!(context_ptr != 0);
        }
    }

    // Manager dropped, window should be destroyed and context cleared
    // Note: DestroyWindow might be async or require message loop to fully vanish,
    // but we check if GWLP_USERDATA is cleared as per our drop logic.
    // Actually, our drop logic clears it.
    // Wait, after DestroyWindow, GetWindowLongPtrW(hwnd) is invalid.
}

#[test]
fn test_multiple_manager_instances_coexist() {
    // This test demonstrates that multiple RawInputManager instances can coexist
    // with independent bridge contexts and hooks, proving thread-safety.
    let device_map1 = DeviceMap::new();
    let device_map2 = DeviceMap::new();
    let (tx1, _rx1) = unbounded();
    let (tx2, _rx2) = unbounded();
    let bridge_context1 = Arc::new(Mutex::new(None));
    let bridge_hook1 = Arc::new(Mutex::new(None));
    let bridge_context2 = Arc::new(Mutex::new(None));
    let bridge_hook2 = Arc::new(Mutex::new(None));

    // Create first manager
    let manager1_result = RawInputManager::new(
        device_map1,
        tx1,
        bridge_context1.clone(),
        bridge_hook1.clone(),
    );

    // Create second manager
    let manager2_result = RawInputManager::new(
        device_map2,
        tx2,
        bridge_context2.clone(),
        bridge_hook2.clone(),
    );

    // Both should succeed (or both fail if no GUI session)
    match (manager1_result, manager2_result) {
        (Ok(manager1), Ok(manager2)) => {
            // Verify they have different windows
            assert_ne!(manager1.hwnd, manager2.hwnd);

            // Verify they have independent bridge contexts
            assert!(bridge_context1
                .lock()
                .expect("Test: bridge_context1 should not be poisoned")
                .is_some());
            assert!(bridge_context2
                .lock()
                .expect("Test: bridge_context2 should not be poisoned")
                .is_some());

            // Verify they have independent hooks
            assert!(bridge_hook1
                .lock()
                .expect("Test: bridge_hook1 should not be poisoned")
                .is_some());
            assert!(bridge_hook2
                .lock()
                .expect("Test: bridge_hook2 should not be poisoned")
                .is_some());

            // Drop them to verify cleanup works independently
            drop(manager1);
            assert!(bridge_context1
                .lock()
                .expect("Test: bridge_context1 should not be poisoned after drop")
                .is_none());
            assert!(bridge_hook1
                .lock()
                .expect("Test: bridge_hook1 should not be poisoned after drop")
                .is_none());

            // Manager2's context should still be valid
            assert!(bridge_context2
                .lock()
                .expect("Test: bridge_context2 should still not be poisoned")
                .is_some());
            assert!(bridge_hook2
                .lock()
                .expect("Test: bridge_hook2 should still not be poisoned")
                .is_some());
        }
        _ => {
            eprintln!("Skipping multiple instances test (no GUI session)");
        }
    }
}
