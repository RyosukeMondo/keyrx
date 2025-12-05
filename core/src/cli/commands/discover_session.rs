//! Session management for device discovery.
//!
//! Handles the capture loop, emergency exit detection, and progress/result reporting.

use crate::cli::{OutputFormat, OutputWriter};
use crate::discovery::{
    default_schema_version, DeviceProfile, DeviceRegistry, DiscoveryProgress, DiscoverySummary,
    DuplicateWarning, ProfileSource, SessionStatus, SessionUpdate,
};
use crate::drivers::DeviceInfo;
use crate::engine::InputEvent;
use crate::traits::InputSource;
use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

use super::discover_validation::{confirm, DiscoverExit};

/// Tracks modifier key state for emergency exit detection.
///
/// Triggers emergency exit on Ctrl+Alt+Shift+Escape.
#[derive(Default)]
pub struct EmergencyTracker {
    ctrl: bool,
    alt: bool,
    shift: bool,
}

impl EmergencyTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update tracker state with an input event.
    /// Returns `true` if emergency exit sequence was triggered.
    pub fn update(&mut self, event: &InputEvent) -> bool {
        use crate::engine::KeyCode;

        match event.key {
            KeyCode::LeftCtrl | KeyCode::RightCtrl => self.ctrl = event.pressed,
            KeyCode::LeftAlt | KeyCode::RightAlt => self.alt = event.pressed,
            KeyCode::LeftShift | KeyCode::RightShift => self.shift = event.pressed,
            KeyCode::Escape => {
                if event.pressed && self.ctrl && self.alt && self.shift {
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

/// Create an emergency exit detector closure for use with DiscoverySession.
pub fn create_emergency_detector() -> (
    Arc<Mutex<EmergencyTracker>>,
    impl Fn(&InputEvent) -> bool + Clone + Send + 'static,
) {
    let tracker = Arc::new(Mutex::new(EmergencyTracker::new()));
    let tracker_clone = tracker.clone();
    let detector = move |event: &InputEvent| {
        tracker_clone
            .lock()
            .map(|mut t| t.update(event))
            .unwrap_or(false)
    };
    (tracker, detector)
}

/// Run the capture session loop, polling for events until discovery completes.
pub async fn capture_session(
    input: &mut Box<dyn InputSource>,
    mut session: crate::discovery::DiscoverySession,
    output: &OutputWriter,
) -> Result<DiscoverySummary> {
    loop {
        let events = input.poll_events().await?;
        if events.is_empty() {
            sleep(Duration::from_millis(25)).await;
            continue;
        }

        for event in events {
            match session.handle_event(&event) {
                SessionUpdate::Ignored => {}
                SessionUpdate::Progress(progress) => report_progress(output, &progress)?,
                SessionUpdate::Duplicate(dup) => report_duplicate(output, &dup)?,
                SessionUpdate::Finished(summary) => return Ok(summary),
            }
        }
    }
}

/// Report discovery progress to the output.
pub fn report_progress(output: &OutputWriter, progress: &DiscoveryProgress) -> Result<()> {
    match output.format() {
        OutputFormat::Json | OutputFormat::Yaml => {
            let payload = ProgressJson {
                status: "progress",
                captured: progress.captured,
                total: progress.total,
                next: progress
                    .next
                    .as_ref()
                    .and_then(|pos| serde_json::to_value(pos).ok()),
            };
            output.data(&payload)?;
        }
        _ => {
            if let Some(current) = progress.current {
                if let Some(next) = progress.next {
                    output.success(&format!(
                        "Captured key at row {}, col {} ({}/{}). Next: row {}, col {}",
                        current.row + 1,
                        current.col + 1,
                        progress.captured,
                        progress.total,
                        next.row + 1,
                        next.col + 1
                    ));
                } else {
                    output.success(&format!(
                        "Captured key at row {}, col {} ({}/{})",
                        current.row + 1,
                        current.col + 1,
                        progress.captured,
                        progress.total
                    ));
                }
            } else {
                output.success(&format!(
                    "Ready to capture keys (0/{}). Press key at row 1, col 1",
                    progress.total
                ));
            }
        }
    }
    Ok(())
}

/// Report a duplicate scan code warning.
pub fn report_duplicate(output: &OutputWriter, dup: &DuplicateWarning) -> Result<()> {
    match output.format() {
        OutputFormat::Json | OutputFormat::Yaml => {
            let payload = DuplicateJson {
                status: "duplicate",
                scan_code: dup.scan_code,
                existing: serde_json::json!(dup.existing),
                attempted: serde_json::json!(dup.attempted),
            };
            output.data(&payload)?;
        }
        _ => output.warning(&format!(
            "Duplicate scan code {} (existing r{},c{} attempted r{},c{})",
            dup.scan_code,
            dup.existing.row + 1,
            dup.existing.col + 1,
            dup.attempted.row + 1,
            dup.attempted.col + 1
        )),
    }
    Ok(())
}

/// Report the final discovery summary.
pub fn report_summary(output: &OutputWriter, summary: &DiscoverySummary) -> Result<()> {
    match output.format() {
        OutputFormat::Json | OutputFormat::Yaml => {
            let payload = SummaryJson {
                status: "summary",
                summary,
            };
            output.data(&payload)?;
        }
        _ => {
            let headline = match summary.status {
                SessionStatus::Completed => "Discovery completed",
                SessionStatus::Cancelled => "Discovery cancelled",
                SessionStatus::Bypassed => "Discovery bypassed (emergency exit)",
                SessionStatus::InProgress => "Discovery incomplete",
            };
            output.success(headline);
            output.success(&format!(
                "Captured {}/{} keys; duplicates: {}; unmapped: {}",
                summary.captured,
                summary.total,
                summary.duplicates.len(),
                summary.unmapped.len()
            ));
            if let Some(msg) = &summary.message {
                output.warning(msg);
            }
        }
    }
    Ok(())
}

/// Handle the discovery summary, saving the profile if completed.
pub fn handle_summary(
    registry: &mut DeviceRegistry,
    summary: DiscoverySummary,
    device: &DeviceInfo,
    output: &OutputWriter,
    assume_yes: bool,
) -> Result<()> {
    report_summary(output, &summary)?;

    match summary.status {
        SessionStatus::Completed => {
            if !assume_yes
                && !matches!(output.format(), OutputFormat::Json | OutputFormat::Yaml)
                && !confirm("Save discovered profile? [y/N]: ")?
            {
                return Err(DiscoverExit::Cancelled.into());
            }

            let profile = DeviceProfile {
                schema_version: default_schema_version(),
                vendor_id: summary.device_id.vendor_id,
                product_id: summary.device_id.product_id,
                name: Some(device.name.clone()),
                discovered_at: Utc::now(),
                rows: summary.rows,
                cols_per_row: summary.cols_per_row.clone(),
                keymap: summary.keymap.clone(),
                aliases: summary.aliases.clone(),
                source: ProfileSource::Discovered,
            };
            let path = registry.save_profile(profile)?;
            output.success(&format!("Saved profile to {}", path.display()));

            if matches!(output.format(), OutputFormat::Json | OutputFormat::Yaml) {
                let payload = DiscoverResultJson {
                    status: "saved",
                    path: path.display().to_string(),
                    cols_per_row: summary.cols_per_row.clone(),
                };
                output.data(&payload)?;
            }
            Ok(())
        }
        SessionStatus::Cancelled | SessionStatus::Bypassed => Err(DiscoverExit::Cancelled.into()),
        SessionStatus::InProgress => {
            Err(DiscoverExit::Validation("discovery did not complete".to_string()).into())
        }
    }
}

// JSON payload structs for structured output

#[derive(Serialize)]
pub struct ExistingProfileJson<'a> {
    pub status: &'a str,
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
}

#[derive(Serialize)]
struct ProgressJson<'a> {
    status: &'a str,
    captured: usize,
    total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct DuplicateJson<'a> {
    status: &'a str,
    scan_code: u16,
    existing: serde_json::Value,
    attempted: serde_json::Value,
}

#[derive(Serialize)]
struct SummaryJson<'a> {
    status: &'a str,
    summary: &'a DiscoverySummary,
}

#[derive(Serialize)]
struct DiscoverResultJson<'a> {
    status: &'a str,
    path: String,
    cols_per_row: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

    fn key_event(key: KeyCode, pressed: bool) -> InputEvent {
        if pressed {
            InputEvent::key_down(key, 0)
        } else {
            InputEvent::key_up(key, 0)
        }
    }

    #[test]
    fn emergency_tracker_triggers_on_combo() {
        let mut tracker = EmergencyTracker::new();

        // Press modifiers
        assert!(!tracker.update(&key_event(KeyCode::LeftCtrl, true)));
        assert!(!tracker.update(&key_event(KeyCode::LeftAlt, true)));
        assert!(!tracker.update(&key_event(KeyCode::LeftShift, true)));

        // Press Escape with all modifiers - should trigger
        assert!(tracker.update(&key_event(KeyCode::Escape, true)));
    }

    #[test]
    fn emergency_tracker_requires_all_modifiers() {
        let mut tracker = EmergencyTracker::new();

        // Only Ctrl + Escape - should not trigger
        tracker.update(&key_event(KeyCode::LeftCtrl, true));
        assert!(!tracker.update(&key_event(KeyCode::Escape, true)));
    }

    #[test]
    fn emergency_tracker_handles_release() {
        let mut tracker = EmergencyTracker::new();

        // Press all modifiers
        tracker.update(&key_event(KeyCode::LeftCtrl, true));
        tracker.update(&key_event(KeyCode::LeftAlt, true));
        tracker.update(&key_event(KeyCode::LeftShift, true));

        // Release one modifier
        tracker.update(&key_event(KeyCode::LeftCtrl, false));

        // Press Escape - should not trigger
        assert!(!tracker.update(&key_event(KeyCode::Escape, true)));
    }
}
