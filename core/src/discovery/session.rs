//! Discovery session state machine.
//!
//! Guides users through pressing keys in a defined row/col order, detects
//! duplicates, and produces serializable progress/summary snapshots that can
//! be consumed by CLI/FFI without side effects.

use crate::discovery::types::{
    default_schema_version, DeviceId, DeviceProfile, PhysicalKey, ProfileSource,
};
use crate::engine::InputEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler,
)]
#[ffi(strategy = "json")]
pub struct ExpectedPosition {
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct DiscoveryProgress {
    pub captured: usize,
    pub total: usize,
    pub current: Option<ExpectedPosition>,
    pub next: Option<ExpectedPosition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct DuplicateWarning {
    pub scan_code: u16,
    pub existing: ExpectedPosition,
    pub attempted: ExpectedPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub enum SessionStatus {
    InProgress,
    Completed,
    Cancelled,
    Bypassed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub struct DiscoverySummary {
    pub device_id: DeviceId,
    pub status: SessionStatus,
    pub message: Option<String>,
    pub rows: u8,
    pub cols_per_row: Vec<u8>,
    pub captured: usize,
    pub total: usize,
    pub next: Option<ExpectedPosition>,
    pub unmapped: Vec<ExpectedPosition>,
    pub duplicates: Vec<DuplicateWarning>,
    pub keymap: HashMap<u16, PhysicalKey>,
    pub aliases: HashMap<String, u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, keyrx_ffi_macros::FfiMarshaler)]
#[ffi(strategy = "json")]
pub enum SessionUpdate {
    Ignored,
    Progress(DiscoveryProgress),
    Duplicate(DuplicateWarning),
    Finished(DiscoverySummary),
}

type SessionUpdateSink = dyn Fn(&SessionUpdate) + Send + Sync + 'static;

fn update_sink_slot() -> &'static Mutex<Option<Arc<SessionUpdateSink>>> {
    static SINK: OnceLock<Mutex<Option<Arc<SessionUpdateSink>>>> = OnceLock::new();
    SINK.get_or_init(|| Mutex::new(None))
}

pub(crate) fn set_session_update_sink(sink: Option<Arc<SessionUpdateSink>>) {
    if let Ok(mut guard) = update_sink_slot().lock() {
        *guard = sink;
    }
}

pub(crate) fn publish_session_update(update: &SessionUpdate) {
    let sink = update_sink_slot()
        .lock()
        .ok()
        .and_then(|guard| (*guard).clone());

    if let Some(callback) = sink {
        callback(update);
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SessionError {
    #[error("rows ({rows}) must match cols_per_row length ({cols_len}) and be non-zero")]
    InvalidLayout { rows: u8, cols_len: usize },
    #[error("each row must have at least one column")]
    EmptyRow,
}

type EmergencyDetector = dyn Fn(&InputEvent) -> bool + Send + Sync;

pub struct DiscoverySession {
    device_id: DeviceId,
    rows: u8,
    cols_per_row: Vec<u8>,
    expected_positions: Vec<ExpectedPosition>,
    cursor: usize,
    keymap: HashMap<u16, PhysicalKey>,
    aliases: HashMap<String, u16>,
    duplicates: Vec<DuplicateWarning>,
    status: SessionStatus,
    message: Option<String>,
    target_device_id: Option<String>,
    emergency_detector: Option<Box<EmergencyDetector>>,
}

impl fmt::Debug for DiscoverySession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscoverySession")
            .field("device_id", &self.device_id)
            .field("rows", &self.rows)
            .field("cols_per_row", &self.cols_per_row)
            .field("cursor", &self.cursor)
            .field("keymap_len", &self.keymap.len())
            .field("duplicates_len", &self.duplicates.len())
            .field("status", &self.status)
            .field("message", &self.message)
            .field("target_device_id", &self.target_device_id)
            .field(
                "has_emergency_detector",
                &self.emergency_detector.as_ref().map(|_| true),
            )
            .finish()
    }
}

impl DiscoverySession {
    pub fn new(device_id: DeviceId, rows: u8, cols_per_row: Vec<u8>) -> Result<Self, SessionError> {
        if rows == 0 || rows as usize != cols_per_row.len() {
            return Err(SessionError::InvalidLayout {
                rows,
                cols_len: cols_per_row.len(),
            });
        }
        if cols_per_row.contains(&0) {
            return Err(SessionError::EmptyRow);
        }

        let mut expected_positions = Vec::new();
        for (row_idx, cols) in cols_per_row.iter().enumerate() {
            for col in 0..*cols {
                expected_positions.push(ExpectedPosition {
                    row: row_idx as u8,
                    col,
                });
            }
        }

        Ok(Self {
            device_id,
            rows,
            cols_per_row,
            expected_positions,
            cursor: 0,
            keymap: HashMap::new(),
            aliases: HashMap::new(),
            duplicates: Vec::new(),
            status: SessionStatus::InProgress,
            message: None,
            target_device_id: None,
            emergency_detector: None,
        })
    }

    pub fn with_target_device_id(mut self, target: impl Into<String>) -> Self {
        self.target_device_id = Some(target.into());
        self
    }

    pub fn with_emergency_exit_detector<F>(mut self, detector: F) -> Self
    where
        F: Fn(&InputEvent) -> bool + Send + Sync + 'static,
    {
        self.emergency_detector = Some(Box::new(detector));
        self
    }

    pub fn handle_event(&mut self, event: &InputEvent) -> SessionUpdate {
        if self.status != SessionStatus::InProgress {
            let update = SessionUpdate::Finished(self.summary());
            publish_session_update(&update);
            return update;
        }

        if let Some(target) = &self.target_device_id {
            if event.device_id.as_ref() != Some(target) {
                let update = SessionUpdate::Ignored;
                publish_session_update(&update);
                return update;
            }
        }

        if let Some(detector) = &self.emergency_detector {
            if detector(event) {
                self.status = SessionStatus::Bypassed;
                self.message = Some("emergency-exit triggered".to_string());
                let update = SessionUpdate::Finished(self.summary());
                publish_session_update(&update);
                return update;
            }
        }

        if !event.pressed {
            let update = SessionUpdate::Ignored;
            publish_session_update(&update);
            return update;
        }

        if self.cursor >= self.expected_positions.len() {
            self.status = SessionStatus::Completed;
            let update = SessionUpdate::Finished(self.summary());
            publish_session_update(&update);
            return update;
        }

        let position = self.expected_positions[self.cursor];

        if let Some(existing) = self.keymap.get(&event.scan_code) {
            // Feature: Undo on double press
            // If the duplicate matches the immediately preceding key, undo it.
            if self.cursor > 0 {
                let last_pos = self.expected_positions[self.cursor - 1];
                if existing.row == last_pos.row && existing.col == last_pos.col {
                    // Undo mapping
                    self.keymap.remove(&event.scan_code);
                    let alias = format!("r{}_c{}", last_pos.row, last_pos.col);
                    self.aliases.remove(&alias);
                    self.cursor -= 1;

                    // Clear the duplicate warning for this specific interaction if needed,
                    // but simply returning updated progress is sufficient.
                    let update = SessionUpdate::Progress(self.progress());
                    publish_session_update(&update);
                    return update;
                }
            }

            let duplicate = DuplicateWarning {
                scan_code: event.scan_code,
                existing: ExpectedPosition {
                    row: existing.row,
                    col: existing.col,
                },
                attempted: position,
            };
            self.duplicates.push(duplicate.clone());
            let update = SessionUpdate::Duplicate(duplicate);
            publish_session_update(&update);
            return update;
        }

        let alias = format!("r{}_c{}", position.row, position.col);
        let mut key = PhysicalKey::new(event.scan_code, position.row, position.col);
        key.alias = Some(alias.clone());

        self.keymap.insert(event.scan_code, key);
        self.aliases.insert(alias, event.scan_code);
        self.cursor += 1;

        let update = if self.cursor == self.expected_positions.len() {
            self.status = SessionStatus::Completed;
            SessionUpdate::Finished(self.summary())
        } else {
            SessionUpdate::Progress(self.progress())
        };

        publish_session_update(&update);
        update
    }

    pub fn skip_current(&mut self) -> SessionUpdate {
        if self.status != SessionStatus::InProgress {
            return SessionUpdate::Ignored;
        }

        if self.cursor >= self.expected_positions.len() {
            return SessionUpdate::Ignored;
        }

        // Just advance the cursor without adding to keymap
        self.cursor += 1;

        let update = if self.cursor >= self.expected_positions.len() {
            self.status = SessionStatus::Completed;
            SessionUpdate::Finished(self.summary())
        } else {
            SessionUpdate::Progress(self.progress())
        };

        publish_session_update(&update);
        update
    }

    pub fn undo_last(&mut self) -> SessionUpdate {
        if self.status != SessionStatus::InProgress && self.status != SessionStatus::Completed {
            // Can't undo if cancelled or bypassed, but allow undoing from Completed state (to re-open session)
            if self.status != SessionStatus::Completed {
                return SessionUpdate::Ignored;
            }
        }

        if self.cursor == 0 {
            return SessionUpdate::Ignored;
        }

        // Move cursor back
        self.cursor -= 1;
        self.status = SessionStatus::InProgress; // Always back to InProgress if we undo

        // Remove mapping if it exists for this position
        // We need to find the key that maps to the expected position at self.cursor
        let position = self.expected_positions[self.cursor];

        // Inefficient search but acceptable for discovery session sizes
        let scan_code_to_remove = self
            .keymap
            .iter()
            .find(|(_, k)| k.row == position.row && k.col == position.col)
            .map(|(s, _)| *s);

        if let Some(scan_code) = scan_code_to_remove {
            self.keymap.remove(&scan_code);
            let alias = format!("r{}_c{}", position.row, position.col);
            self.aliases.remove(&alias);
        }

        let update = SessionUpdate::Progress(self.progress());
        publish_session_update(&update);
        update
    }

    pub fn cancel(&mut self, reason: impl Into<String>) -> DiscoverySummary {
        self.status = SessionStatus::Cancelled;
        self.message = Some(reason.into());
        self.summary()
    }

    pub fn progress(&self) -> DiscoveryProgress {
        DiscoveryProgress {
            captured: self.cursor,
            total: self.expected_positions.len(),
            current: if self.cursor > 0 {
                self.expected_positions.get(self.cursor - 1).copied()
            } else {
                None
            },
            next: self.expected_positions.get(self.cursor).copied(),
        }
    }

    pub fn summary(&self) -> DiscoverySummary {
        DiscoverySummary {
            device_id: self.device_id,
            status: self.status.clone(),
            message: self.message.clone(),
            rows: self.rows,
            cols_per_row: self.cols_per_row.clone(),
            captured: self.cursor,
            total: self.expected_positions.len(),
            next: self.expected_positions.get(self.cursor).copied(),
            unmapped: self.expected_positions[self.cursor..].to_vec(),
            duplicates: self.duplicates.clone(),
            keymap: self.keymap.clone(),
            aliases: self.aliases.clone(),
        }
    }

    pub fn into_profile(self, name: Option<String>, discovered_at: DateTime<Utc>) -> DeviceProfile {
        DeviceProfile {
            schema_version: default_schema_version(),
            vendor_id: self.device_id.vendor_id,
            product_id: self.device_id.product_id,
            name,
            discovered_at,
            rows: self.rows,
            cols_per_row: self.cols_per_row,
            keymap: self.keymap,
            aliases: self.aliases,
            source: ProfileSource::Discovered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn event_for(
        scan_code: u16,
        target: Option<&str>,
        pressed: bool,
        timestamp: u64,
    ) -> InputEvent {
        InputEvent::with_metadata(
            KeyCode::Unknown(scan_code),
            pressed,
            timestamp,
            target.map(String::from),
            false,
            false,
            scan_code,
            None,
        )
    }

    #[test]
    fn rejects_invalid_layout() {
        let err = DiscoverySession::new(DeviceId::new(1, 2), 2, vec![3]).unwrap_err();
        assert_eq!(
            err,
            SessionError::InvalidLayout {
                rows: 2,
                cols_len: 1
            }
        );

        let err = DiscoverySession::new(DeviceId::new(1, 2), 1, vec![0]).unwrap_err();
        assert_eq!(err, SessionError::EmptyRow);
    }

    #[test]
    fn progresses_and_completes() {
        let device_id = DeviceId::new(0x1, 0x2);
        let mut session = DiscoverySession::new(device_id, 1, vec![2])
            .unwrap()
            .with_target_device_id("kb0");

        assert_eq!(
            session.progress(),
            DiscoveryProgress {
                captured: 0,
                total: 2,
                current: None,
                next: Some(ExpectedPosition { row: 0, col: 0 })
            }
        );

        assert_eq!(
            session.handle_event(&event_for(30, Some("other"), true, 1)),
            SessionUpdate::Ignored
        );

        let update = session.handle_event(&event_for(30, Some("kb0"), true, 2));
        assert_eq!(
            update,
            SessionUpdate::Progress(DiscoveryProgress {
                captured: 1,
                total: 2,
                current: Some(ExpectedPosition { row: 0, col: 0 }),
                next: Some(ExpectedPosition { row: 0, col: 1 })
            })
        );

        let finished = session.handle_event(&event_for(31, Some("kb0"), true, 3));
        let summary = match finished {
            SessionUpdate::Finished(summary) => summary,
            other => {
                unreachable!("expected Finished update, got {:?}", other)
            }
        };
        assert_eq!(summary.status, SessionStatus::Completed);
        assert_eq!(summary.captured, 2);
        assert!(summary.unmapped.is_empty());
        assert_eq!(summary.keymap.len(), 2);

        let profile = session.into_profile(Some("Test".to_string()), Utc::now());
        assert_eq!(profile.vendor_id, device_id.vendor_id);
        assert_eq!(profile.keymap.len(), 2);
        assert_eq!(profile.aliases.len(), 2);
        assert_eq!(profile.source, ProfileSource::Discovered);
    }

    #[test]
    fn detects_duplicates_and_recovers() {
        let mut session =
            DiscoverySession::new(DeviceId::new(0xA, 0xB), 1, vec![3]).expect("valid layout");

        // First key press
        let first = session.handle_event(&event_for(10, None, true, 1));
        assert!(matches!(first, SessionUpdate::Progress(_)));

        // Second key press (different scan code)
        let second = session.handle_event(&event_for(11, None, true, 2));
        assert!(matches!(second, SessionUpdate::Progress(_)));

        // Third press with scan code 10 again - should be duplicate since it's not the immediately preceding position
        let duplicate = session.handle_event(&event_for(10, None, true, 3));
        let dup = match duplicate {
            SessionUpdate::Duplicate(dup) => dup,
            other => {
                unreachable!("expected Duplicate update, got {:?}", other)
            }
        };
        assert_eq!(dup.scan_code, 10);
        assert_eq!(dup.existing, ExpectedPosition { row: 0, col: 0 });
        assert_eq!(dup.attempted, ExpectedPosition { row: 0, col: 2 });

        // Fourth key (new scan code) should complete the session
        let finished = session.handle_event(&event_for(12, None, true, 4));
        assert!(matches!(
            finished,
            SessionUpdate::Finished(DiscoverySummary {
                status: SessionStatus::Completed,
                ..
            })
        ));
        assert_eq!(session.duplicates.len(), 1);
        assert_eq!(session.keymap.len(), 3);
    }

    #[test]
    fn cancel_and_bypass() {
        let mut session = DiscoverySession::new(DeviceId::new(0xC, 0xD), 1, vec![1]).unwrap();
        session.handle_event(&event_for(50, None, true, 1));

        let summary = session.cancel("user cancelled");
        assert_eq!(summary.status, SessionStatus::Cancelled);
        assert_eq!(summary.message.as_deref(), Some("user cancelled"));

        let after_cancel = session.handle_event(&event_for(51, None, true, 2));
        assert!(matches!(
            after_cancel,
            SessionUpdate::Finished(DiscoverySummary {
                status: SessionStatus::Cancelled,
                ..
            })
        ));

        let mut session = DiscoverySession::new(DeviceId::new(0xE, 0xF), 1, vec![1])
            .unwrap()
            .with_emergency_exit_detector(|event| event.scan_code == 99);

        let bypass = session.handle_event(&event_for(99, None, true, 1));
        let summary = match bypass {
            SessionUpdate::Finished(summary) => summary,
            other => {
                unreachable!("expected Finished (bypass) update, got {:?}", other)
            }
        };
        assert_eq!(summary.status, SessionStatus::Bypassed);
        assert_eq!(summary.message.as_deref(), Some("emergency-exit triggered"));
    }

    #[test]
    fn publishes_updates_through_sink() {
        let counter = Arc::new(AtomicUsize::new(0));
        let completed = Arc::new(Mutex::new(Vec::new()));
        let counter_clone = counter.clone();
        let completed_clone = completed.clone();

        set_session_update_sink(Some(Arc::new(move |update| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
            if let SessionUpdate::Finished(summary) = update {
                completed_clone.lock().unwrap().push(summary.status.clone());
            }
        })));

        let mut session = DiscoverySession::new(DeviceId::new(0xAA, 0xBB), 1, vec![2]).unwrap();
        session.handle_event(&event_for(42, None, true, 1));
        session.handle_event(&event_for(43, None, true, 2));

        assert!(counter.load(Ordering::Relaxed) >= 2);
        let statuses = completed.lock().unwrap();
        assert!(statuses
            .iter()
            .any(|status| status == &SessionStatus::Completed));

        set_session_update_sink(None);
    }
}
