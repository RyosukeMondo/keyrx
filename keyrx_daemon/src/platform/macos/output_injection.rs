//! macOS keyboard output injection using enigo.
//!
//! This module provides keyboard event injection using the enigo crate,
//! which uses CGEventPost for synthetic keyboard events.

use enigo::{Direction, Enigo, Keyboard, Settings};
use keyrx_core::runtime::KeyEvent;

use crate::platform::{DeviceError, OutputDevice};
use super::keycode_map::keyrx_to_enigo_key;

/// macOS keyboard output injector.
///
/// Injects keyboard events using enigo::Enigo.
pub struct MacosOutputInjector {
    enigo: Enigo,
}

impl MacosOutputInjector {
    /// Creates a new output injector instance.
    ///
    /// # Returns
    ///
    /// A new `MacosOutputInjector` with an initialized Enigo instance.
    ///
    /// # Errors
    ///
    /// Returns `DeviceError::PermissionDenied` if Accessibility permissions
    /// are not granted or Enigo initialization fails.
    pub fn new() -> Result<Self, DeviceError> {
        let settings = Settings::default();
        let enigo = Enigo::new(&settings).map_err(|e| {
            DeviceError::PermissionDenied(format!(
                "Failed to create Enigo instance (check Accessibility permissions): {}",
                e
            ))
        })?;
        Ok(Self { enigo })
    }
}

impl Default for MacosOutputInjector {
    fn default() -> Self {
        Self::new().expect("Failed to create default MacosOutputInjector")
    }
}

impl OutputDevice for MacosOutputInjector {
    fn inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError> {
        // Convert keyrx KeyCode to enigo::Key
        let enigo_key = keyrx_to_enigo_key(event.keycode()).ok_or_else(|| {
            DeviceError::InjectionFailed(format!(
                "Unsupported key for output injection: {:?}",
                event.keycode()
            ))
        })?;

        // Determine direction (Press or Release)
        let direction = if event.is_press() {
            Direction::Press
        } else if event.is_release() {
            Direction::Release
        } else {
            return Err(DeviceError::InjectionFailed(format!(
                "Unknown event type for key {:?}",
                event.keycode()
            )));
        };

        log::trace!(
            "Injecting key event: {:?} -> {:?} ({:?})",
            event.keycode(),
            enigo_key,
            direction
        );

        // Inject the event
        self.enigo.key(enigo_key, direction).map_err(|e| {
            DeviceError::InjectionFailed(format!(
                "Failed to inject key {:?}: {}",
                event.keycode(),
                e
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::config::keys::KeyCode;

    /// Helper to create an injector for testing.
    /// In CI/test environments without Accessibility permissions, this will return None.
    fn try_create_injector() -> Option<MacosOutputInjector> {
        MacosOutputInjector::new().ok()
    }

    #[test]
    fn test_macos_output_injector_creation() {
        // Attempt to create an injector
        let result = MacosOutputInjector::new();

        // In test environments without Accessibility permissions, expect an error
        // On systems with permissions, this should succeed
        match result {
            Ok(_) => {
                // Success - we have permissions
            }
            Err(DeviceError::PermissionDenied(_)) => {
                // Expected in CI/test environments without Accessibility
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[test]
    fn test_keycode_to_enigo_mapping() {
        // Test that keyrx_to_enigo_key works correctly without needing Enigo
        use super::keyrx_to_enigo_key;

        // Test some basic mappings
        assert!(keyrx_to_enigo_key(KeyCode::A).is_some());
        assert!(keyrx_to_enigo_key(KeyCode::Enter).is_some());
        assert!(keyrx_to_enigo_key(KeyCode::LShift).is_some());

        // Test unsupported key
        assert!(keyrx_to_enigo_key(KeyCode::Insert).is_none());
    }

    #[test]
    fn test_inject_letter_press() {
        if let Some(mut injector) = try_create_injector() {
            let event = KeyEvent::press(KeyCode::A);
            assert!(injector.inject_event(event).is_ok());
        }
    }

    #[test]
    fn test_inject_letter_release() {
        if let Some(mut injector) = try_create_injector() {
            let event = KeyEvent::release(KeyCode::A);
            assert!(injector.inject_event(event).is_ok());
        }
    }

    #[test]
    fn test_inject_number_key() {
        if let Some(mut injector) = try_create_injector() {
            assert!(injector.inject_event(KeyEvent::press(KeyCode::Num1)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::Num1)).is_ok());
        }
    }

    #[test]
    fn test_inject_modifier_key() {
        if let Some(mut injector) = try_create_injector() {
            assert!(injector.inject_event(KeyEvent::press(KeyCode::LShift)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::LShift)).is_ok());
        }
    }

    #[test]
    fn test_inject_special_key() {
        if let Some(mut injector) = try_create_injector() {
            assert!(injector.inject_event(KeyEvent::press(KeyCode::Enter)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::Enter)).is_ok());
        }
    }

    #[test]
    fn test_inject_function_key() {
        if let Some(mut injector) = try_create_injector() {
            assert!(injector.inject_event(KeyEvent::press(KeyCode::F1)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::F1)).is_ok());
        }
    }

    #[test]
    fn test_inject_navigation_key() {
        if let Some(mut injector) = try_create_injector() {
            let keys = vec![KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right];
            for key in keys {
                assert!(injector.inject_event(KeyEvent::press(key)).is_ok());
                assert!(injector.inject_event(KeyEvent::release(key)).is_ok());
            }
        }
    }

    #[test]
    fn test_inject_numpad_key() {
        if let Some(mut injector) = try_create_injector() {
            assert!(injector.inject_event(KeyEvent::press(KeyCode::Numpad5)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::Numpad5)).is_ok());
        }
    }

    #[test]
    fn test_inject_punctuation_key() {
        if let Some(mut injector) = try_create_injector() {
            let keys = vec![
                KeyCode::Comma,
                KeyCode::Period,
                KeyCode::Semicolon,
                KeyCode::Quote,
            ];
            for key in keys {
                assert!(injector.inject_event(KeyEvent::press(key)).is_ok());
            }
        }
    }

    #[test]
    fn test_inject_unsupported_key_returns_error() {
        if let Some(mut injector) = try_create_injector() {
            // Insert key is not supported by enigo (returns None in keyrx_to_enigo_key)
            let event = KeyEvent::press(KeyCode::Insert);
            let result = injector.inject_event(event);

            assert!(result.is_err());
            match result {
                Err(DeviceError::InjectionFailed(msg)) => {
                    assert!(msg.contains("Unsupported key"));
                }
                _ => panic!("Expected InjectionFailed error"),
            }
        }
    }

    #[test]
    fn test_multiple_sequential_injections() {
        if let Some(mut injector) = try_create_injector() {
            // Simulate typing "AB"
            assert!(injector.inject_event(KeyEvent::press(KeyCode::A)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::A)).is_ok());
            assert!(injector.inject_event(KeyEvent::press(KeyCode::B)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::B)).is_ok());
        }
    }

    #[test]
    fn test_modifier_with_letter() {
        if let Some(mut injector) = try_create_injector() {
            // Simulate Shift+A
            assert!(injector.inject_event(KeyEvent::press(KeyCode::LShift)).is_ok());
            assert!(injector.inject_event(KeyEvent::press(KeyCode::A)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::A)).is_ok());
            assert!(injector.inject_event(KeyEvent::release(KeyCode::LShift)).is_ok());
        }
    }

    #[test]
    fn test_all_supported_letters() {
        if let Some(mut injector) = try_create_injector() {
            let letters = vec![
                KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E,
                KeyCode::F, KeyCode::G, KeyCode::H, KeyCode::I, KeyCode::J,
                KeyCode::K, KeyCode::L, KeyCode::M, KeyCode::N, KeyCode::O,
                KeyCode::P, KeyCode::Q, KeyCode::R, KeyCode::S, KeyCode::T,
                KeyCode::U, KeyCode::V, KeyCode::W, KeyCode::X, KeyCode::Y,
                KeyCode::Z,
            ];

            for letter in letters {
                assert!(injector.inject_event(KeyEvent::press(letter)).is_ok());
            }
        }
    }

    #[test]
    fn test_all_modifiers() {
        if let Some(mut injector) = try_create_injector() {
            let modifiers = vec![
                KeyCode::LShift, KeyCode::RShift,
                KeyCode::LCtrl, KeyCode::RCtrl,
                KeyCode::LAlt, KeyCode::RAlt,
                KeyCode::LMeta, KeyCode::RMeta,
            ];

            for modifier in modifiers {
                assert!(injector.inject_event(KeyEvent::press(modifier)).is_ok());
            }
        }
    }

    #[test]
    fn test_media_keys() {
        if let Some(mut injector) = try_create_injector() {
            let media_keys = vec![
                KeyCode::VolumeUp,
                KeyCode::VolumeDown,
                KeyCode::Mute,
                KeyCode::MediaPlayPause,
                KeyCode::MediaNext,
                KeyCode::MediaPrevious,
            ];

            for key in media_keys {
                let result = injector.inject_event(KeyEvent::press(key));
                if key != KeyCode::MediaStop {
                    assert!(result.is_ok());
                }
            }
        }
    }
}
