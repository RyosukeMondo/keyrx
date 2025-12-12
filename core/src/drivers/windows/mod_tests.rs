use super::*;
use crate::drivers::{InjectedKey, KeyInjector, MockKeyInjector};
use crate::engine::{KeyCode, OutputAction};
use crate::traits::InputSource;

#[test]
fn windows_input_default() {
    let input = WindowsInput::default();
    assert!(!input.is_running());
}

#[test]
fn windows_input_new() {
    let input = WindowsInput::new().unwrap();
    assert!(!input.is_running());
}

#[test]
fn windows_input_has_receiver() {
    let input = WindowsInput::new().unwrap();
    // Verify we can access the receiver (channel is empty initially)
    assert!(input.receiver().try_recv().is_err());
}

#[test]
fn windows_input_with_mock_injector() {
    let mock = MockKeyInjector::new();
    let input = WindowsInput::new_with_injector(mock).unwrap();
    assert!(!input.is_running());
}

#[test]
fn windows_input_mock_injector_captures_keys() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    // Directly test the inject_key helper
    input.inject_key(KeyCode::A, true).unwrap();
    input.inject_key(KeyCode::A, false).unwrap();
    input.inject_key(KeyCode::Escape, true).unwrap();

    // Verify injections were captured
    let injected = input.injector().injected_keys();
    assert_eq!(injected.len(), 3);
    assert_eq!(injected[0], InjectedKey::press(KeyCode::A));
    assert_eq!(injected[1], InjectedKey::release(KeyCode::A));
    assert_eq!(injected[2], InjectedKey::press(KeyCode::Escape));
}

#[test]
fn windows_input_mock_injector_sync() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    // Sync is a no-op for mock, but verify it doesn't panic
    input.injector_mut().sync().unwrap();
    assert_eq!(input.injector().sync_count(), 1);
}

#[test]
fn windows_input_mock_injector_was_pressed() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    assert!(!input.injector().was_pressed(KeyCode::B));

    input.inject_key(KeyCode::B, true).unwrap();
    assert!(input.injector().was_pressed(KeyCode::B));
    assert!(!input.injector().was_pressed(KeyCode::C));
}

#[test]
fn windows_input_mock_injector_was_tapped() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    // Press and release = tap
    input.inject_key(KeyCode::Space, true).unwrap();
    assert!(!input.injector().was_tapped(KeyCode::Space)); // Not yet
    input.inject_key(KeyCode::Space, false).unwrap();
    assert!(input.injector().was_tapped(KeyCode::Space)); // Now it's a tap
}

#[test]
fn windows_input_mock_injector_fail_next() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    // Set up failure
    input.injector_mut().fail_next_injection();

    // This should fail
    let result = input.inject_key(KeyCode::A, true);
    assert!(result.is_err());

    // Next one should succeed
    let result = input.inject_key(KeyCode::A, true);
    assert!(result.is_ok());
}

#[test]
fn windows_input_mock_injector_clear() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    input.inject_key(KeyCode::A, true).unwrap();
    input.injector_mut().sync().unwrap();

    assert_eq!(input.injector().injected_keys().len(), 1);
    assert_eq!(input.injector().sync_count(), 1);

    input.injector_mut().clear();

    assert!(input.injector().injected_keys().is_empty());
    assert_eq!(input.injector().sync_count(), 0);
}

#[tokio::test]
async fn windows_input_send_output_with_mock() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    // Simulate running state for send_output to work
    input.prepare_start();

    // Test KeyDown
    input
        .send_output(OutputAction::KeyDown(KeyCode::A))
        .await
        .unwrap();
    assert!(input.injector().was_pressed(KeyCode::A));

    // Test KeyUp
    input
        .send_output(OutputAction::KeyUp(KeyCode::B))
        .await
        .unwrap();
    assert!(input.injector().was_released(KeyCode::B));

    // Test KeyTap
    input
        .send_output(OutputAction::KeyTap(KeyCode::C))
        .await
        .unwrap();
    assert!(input.injector().was_tapped(KeyCode::C));

    // Verify total injections
    let injected = input.injector().injected_keys();
    assert_eq!(injected.len(), 4); // A down, B up, C down, C up
}

#[tokio::test]
async fn windows_input_send_output_block_passthrough() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();
    input.prepare_start();

    // Block and PassThrough should not inject any keys
    input.send_output(OutputAction::Block).await.unwrap();
    input.send_output(OutputAction::PassThrough).await.unwrap();

    assert!(input.injector().injected_keys().is_empty());
}

#[tokio::test]
async fn windows_input_send_output_not_running() {
    let mock = MockKeyInjector::new();
    let mut input = WindowsInput::new_with_injector(mock).unwrap();

    // Not running - send_output should be a no-op
    assert!(!input.is_running());
    input
        .send_output(OutputAction::KeyDown(KeyCode::A))
        .await
        .unwrap();

    // Nothing should be injected
    assert!(input.injector().injected_keys().is_empty());
}
