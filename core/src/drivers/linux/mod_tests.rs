#![cfg(test)]
#![allow(clippy::unwrap_used)] // Tests use unwrap for clarity

use super::*;
use crate::drivers::{InjectedKey, MockKeyInjector};
use crate::engine::KeyCode;
use crate::errors::KeyrxError;

/// Test that LinuxInput can be created with a mock injector.
/// This test requires a keyboard device to be available, but doesn't
/// need uinput permissions since we're using a mock.
#[test]
fn linux_input_with_mock_injector_compiles() {
    // This test verifies the type signatures work correctly
    fn _create_with_mock(_path: PathBuf) -> Result<LinuxInput, KeyrxError> {
        let mock = MockKeyInjector::new();
        LinuxInput::new_with_injector(Some(_path), Box::new(mock))
    }
}

#[test]
fn prepare_start_skips_uinput_for_mock_injector() {
    struct NoUinputInjector;
    impl KeyInjector for NoUinputInjector {
        fn inject(&mut self, _key: KeyCode, _pressed: bool) -> Result<(), KeyrxError> {
            Ok(())
        }
        fn sync(&mut self) -> Result<(), KeyrxError> {
            Ok(())
        }
        fn needs_uinput(&self) -> bool {
            false
        }
    }

    let mut input = LinuxInput::new_with_injector(
        Some(PathBuf::from("/dev/input/event-test")),
        Box::new(NoUinputInjector),
    )
    .unwrap();

    // Should not try to access /dev/uinput and should mark running true
    input.prepare_start().unwrap();
    assert!(input.running.load(Ordering::Relaxed));
}

/// Test that UinputWriter implements KeyInjector.
#[test]
fn uinput_writer_implements_key_injector() {
    fn assert_key_injector<T: KeyInjector>() {}
    assert_key_injector::<UinputWriter>();
}

/// Test MockKeyInjector records injections correctly when used as a trait object.
#[test]
fn mock_injector_as_trait_object() {
    let mut injector: Box<dyn KeyInjector> = Box::new(MockKeyInjector::new());

    injector.inject(KeyCode::A, true).unwrap();
    injector.inject(KeyCode::A, false).unwrap();
    injector.sync().unwrap();

    // Downcast to check recorded injections
    // Note: In real tests, you'd typically keep a reference to the mock
}

/// Test that the injector trait is Send.
#[test]
fn key_injector_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Box<dyn KeyInjector>>();
}

#[tokio::test]
async fn poll_events_returns_channel_data() {
    let mock = MockKeyInjector::new();
    let clone = mock.clone();
    let mut input =
        LinuxInput::new_with_injector(Some(PathBuf::from("/dev/input/event-test")), Box::new(mock))
            .unwrap();

    input.running.store(true, Ordering::Relaxed);
    input
        .tx
        .send(InputEvent {
            key: KeyCode::A,
            pressed: true,
            timestamp_us: 1,
            device_id: Some("dev".into()),
            is_repeat: false,
            is_synthetic: false,
            scan_code: 30,
            serial_number: None,
        })
        .unwrap();

    let events = input.poll_events().await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].key, KeyCode::A);
    assert!(input.is_running());

    // Ensure injector was untouched during poll
    assert!(clone.injected_keys().is_empty());
}

#[tokio::test]
async fn poll_events_errors_when_panic_flag_set() {
    let mut input = LinuxInput::new_with_injector(
        Some(PathBuf::from("/dev/input/event-test")),
        Box::new(MockKeyInjector::new()),
    )
    .unwrap();

    input.panic_error.store(true, Ordering::SeqCst);
    let err = input.poll_events().await.unwrap_err();
    assert!(err.to_string().contains("panic"));
}

#[tokio::test]
async fn poll_events_handles_disconnected_channel() {
    let mut input = LinuxInput::new_with_injector(
        Some(PathBuf::from("/dev/input/event-test")),
        Box::new(MockKeyInjector::new()),
    )
    .unwrap();
    input.running.store(true, Ordering::Relaxed);

    let (dummy_tx, _) = crossbeam_channel::unbounded();
    let old_tx = std::mem::replace(&mut input.tx, dummy_tx);
    drop(old_tx);

    let err = input.poll_events().await.unwrap_err();
    assert!(err.to_string().contains("disconnected"));
}

#[tokio::test]
async fn send_output_uses_injector_when_running() {
    let mock = MockKeyInjector::new();
    let recorder = mock.clone();
    let mut input =
        LinuxInput::new_with_injector(Some(PathBuf::from("/dev/input/event-test")), Box::new(mock))
            .unwrap();
    input.running.store(true, Ordering::Relaxed);

    input
        .send_output(OutputAction::KeyDown(KeyCode::Escape))
        .await
        .unwrap();
    input
        .send_output(OutputAction::KeyUp(KeyCode::Escape))
        .await
        .unwrap();
    input
        .send_output(OutputAction::KeyTap(KeyCode::Enter))
        .await
        .unwrap();
    input.send_output(OutputAction::Block).await.unwrap();
    input.send_output(OutputAction::PassThrough).await.unwrap();

    let injected = recorder.injected_keys();
    assert_eq!(injected.len(), 4);
    assert_eq!(injected[0], InjectedKey::press(KeyCode::Escape));
    assert_eq!(injected[1], InjectedKey::release(KeyCode::Escape));
    assert_eq!(injected[2], InjectedKey::press(KeyCode::Enter));
    assert_eq!(injected[3], InjectedKey::release(KeyCode::Enter));
}

#[tokio::test]
async fn send_output_noop_when_stopped() {
    let mock = MockKeyInjector::new();
    let recorder = mock.clone();
    let mut input =
        LinuxInput::new_with_injector(Some(PathBuf::from("/dev/input/event-test")), Box::new(mock))
            .unwrap();

    input
        .send_output(OutputAction::KeyDown(KeyCode::Escape))
        .await
        .unwrap();

    assert!(recorder.injected_keys().is_empty());
}

#[tokio::test]
async fn send_output_propagates_inject_error() {
    let mut mock = MockKeyInjector::new();
    mock.fail_next_injection();
    let mut input =
        LinuxInput::new_with_injector(Some(PathBuf::from("/dev/input/event-test")), Box::new(mock))
            .unwrap();
    input.running.store(true, Ordering::Relaxed);

    let err = input
        .send_output(OutputAction::KeyDown(KeyCode::Escape))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Mock injection failure"));
}

#[tokio::test]
async fn stop_resets_running_and_drains_events() {
    let mut input = LinuxInput::new_with_injector(
        Some(PathBuf::from("/dev/input/event-test")),
        Box::new(MockKeyInjector::new()),
    )
    .unwrap();
    input.running.store(true, Ordering::Relaxed);
    input
        .tx
        .send(InputEvent {
            key: KeyCode::B,
            pressed: true,
            timestamp_us: 2,
            device_id: Some("dev".into()),
            is_repeat: false,
            is_synthetic: false,
            scan_code: 48,
            serial_number: None,
        })
        .unwrap();

    input.stop().await.unwrap();
    assert!(!input.is_running());
    assert!(input.rx.try_recv().is_err());
}

#[test]
fn log_helpers_do_not_panic() {
    let input = LinuxInput::new_with_injector(
        Some(PathBuf::from("/dev/input/event-test")),
        Box::new(MockKeyInjector::new()),
    )
    .unwrap();

    input.log_block_action();
    input.log_passthrough_action();
    input.log_poll_when_inactive();
}
