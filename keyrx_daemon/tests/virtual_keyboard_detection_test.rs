#[cfg(test)]
mod test_virtual_keyboard_detection {
    use keyrx_daemon::device_manager::enumerate_keyboards;
    use keyrx_daemon::test_utils::VirtualKeyboard;
    use std::thread;
    use std::time::Duration;

    #[test]
    #[ignore]
    fn test_virtual_keyboard_is_detectable() {
        if !keyrx_daemon::test_utils::can_access_uinput() {
            eprintln!("SKIPPED: uinput not accessible");
            return;
        }

        // Create virtual keyboard
        let keyboard =
            VirtualKeyboard::create("test-detection").expect("Failed to create keyboard");
        let name = keyboard.name().to_string();
        println!("Created virtual keyboard: {}", name);

        // Give kernel time to register
        thread::sleep(Duration::from_millis(500));

        // Try to enumerate keyboards
        let keyboards = enumerate_keyboards().expect("Failed to enumerate");

        println!("Found {} keyboards:", keyboards.len());
        for kb in &keyboards {
            println!("  - {} ({})", kb.name, kb.path.display());
        }

        // Check if our virtual keyboard is in the list
        let found = keyboards.iter().any(|kb| kb.name == name);

        if !found {
            panic!(
                "Virtual keyboard '{}' was NOT found by enumerate_keyboards()",
                name
            );
        }

        println!("SUCCESS: Virtual keyboard was detected");
    }
}
