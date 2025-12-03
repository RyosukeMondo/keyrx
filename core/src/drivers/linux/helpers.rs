use super::{EvdevReader, LinuxInput};
use crate::engine::{InputEvent, KeyCode};
use crate::errors::{driver::*, runtime::*, KeyrxError};
use crate::{bail_keyrx, keyrx_err};
use crossbeam_channel::TryRecvError;
use std::sync::atomic::Ordering;
use tracing::{debug, error, trace, warn};

impl LinuxInput {
    pub(super) fn fail_if_reader_panicked(&mut self) -> Result<(), KeyrxError> {
        if self.panic_error.load(Ordering::SeqCst) {
            error!(
                service = "keyrx",
                event = "linux_reader_panic_detected",
                component = "linux_input",
                "poll_events called after reader thread panic"
            );
            self.running.store(false, Ordering::Relaxed);
            bail_keyrx!(THREAD_PANIC, thread = "input reader");
        }
        Ok(())
    }

    pub(super) fn is_inactive(&self) -> bool {
        !self.running.load(Ordering::Relaxed)
    }

    pub(super) fn log_poll_when_inactive(&self) {
        trace!(
            service = "keyrx",
            event = "linux_poll_events_inactive",
            component = "linux_input",
            "poll_events called while not running"
        );
    }

    pub(super) fn next_event(&mut self) -> Result<Option<InputEvent>, KeyrxError> {
        match self.rx.try_recv() {
            Ok(event) => {
                trace!(
                    service = "keyrx",
                    event = "linux_input_event_received",
                    component = "linux_input",
                    key = ?event.key,
                    pressed = event.pressed,
                    "Received input event"
                );
                Ok(Some(event))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => self.handle_disconnected_channel(),
        }
    }

    fn handle_disconnected_channel(&mut self) -> Result<Option<InputEvent>, KeyrxError> {
        if self.panic_error.load(Ordering::SeqCst) {
            error!(
                service = "keyrx",
                event = "linux_channel_disconnected",
                component = "linux_input",
                reason = "reader_panic",
                "Event channel disconnected due to reader thread panic"
            );
            self.running.store(false, Ordering::Relaxed);
            bail_keyrx!(THREAD_PANIC, thread = "input reader");
        }
        error!(
            service = "keyrx",
            event = "linux_channel_disconnected",
            component = "linux_input",
            reason = "unexpected_disconnect",
            "Event channel disconnected - reader thread may have crashed"
        );
        self.running.store(false, Ordering::Relaxed);
        bail_keyrx!(DRIVER_DEVICE_DISCONNECTED, device = "input reader channel");
    }

    pub(super) fn log_polled_events(&self, count: usize) {
        if count > 0 {
            debug!(
                service = "keyrx",
                event = "linux_poll_events",
                component = "linux_input",
                count = count,
                "Returning polled events"
            );
        }
    }

    pub(super) fn log_inactive_send(&self) {
        trace!(
            service = "keyrx",
            event = "linux_send_output_inactive",
            component = "linux_input",
            "send_output called while not running"
        );
    }

    pub(super) fn inject_key_action(
        &mut self,
        key: KeyCode,
        pressed: bool,
        event: &'static str,
    ) -> Result<(), KeyrxError> {
        debug!(
            service = "keyrx",
            event = event,
            component = "linux_input",
            key = ?key,
            pressed = pressed,
            "Sending key action"
        );
        self.injector.inject(key, pressed)
    }

    pub(super) fn tap_key(&mut self, key: KeyCode) -> Result<(), KeyrxError> {
        debug!(
            service = "keyrx",
            event = "linux_key_tap",
            component = "linux_input",
            key = ?key,
            "Sending key tap"
        );
        self.injector.inject(key, true)?;
        self.injector.inject(key, false)
    }

    pub(super) fn log_block_action(&self) {
        trace!(
            service = "keyrx",
            event = "linux_block_action",
            component = "linux_input",
            "Blocking key (no action needed)"
        );
    }

    pub(super) fn log_passthrough_action(&self) {
        trace!(
            service = "keyrx",
            event = "linux_passthrough_action",
            component = "linux_input",
            "PassThrough (no action needed)"
        );
    }

    pub(super) fn prepare_start(&mut self) -> Result<(), KeyrxError> {
        if self.injector.needs_uinput() {
            LinuxInput::check_uinput_accessible()?;
        }
        self.panic_error.store(false, Ordering::SeqCst);
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub(super) fn build_reader(&self) -> Result<EvdevReader, KeyrxError> {
        EvdevReader::new(
            self.device_path.clone(),
            self.tx.clone(),
            self.running.clone(),
            self.panic_error.clone(),
        )
        .map_err(|e| keyrx_err!(DRIVER_INIT_FAILED, reason = e.to_string()))
    }

    pub(super) fn spawn_reader(&mut self, reader: EvdevReader) {
        let handle = reader.spawn();
        self.reader_handle = Some(handle);
    }

    pub(super) fn log_started(&self) {
        debug!(
            service = "keyrx",
            event = "linux_started",
            component = "linux_input",
            path = %self.device_path.display(),
            "LinuxInput started successfully"
        );
    }

    pub(super) fn log_start_skipped(&self) {
        warn!(
            service = "keyrx",
            event = "linux_start_skipped",
            component = "linux_input",
            reason = "already_running",
            "LinuxInput already running"
        );
    }

    pub(super) fn log_stop_skipped(&self) {
        debug!(
            service = "keyrx",
            event = "linux_stop_skipped",
            component = "linux_input",
            reason = "already_stopped",
            "LinuxInput already stopped"
        );
    }

    pub(super) fn join_reader_thread(&mut self) {
        if let Some(handle) = self.reader_handle.take() {
            debug!(
                service = "keyrx",
                event = "linux_join_reader",
                component = "linux_input",
                "Waiting for reader thread to finish"
            );
            match handle.join() {
                Ok(()) => {
                    debug!(
                        service = "keyrx",
                        event = "linux_reader_stopped",
                        component = "linux_input",
                        status = "clean",
                        "Reader thread finished cleanly"
                    );
                }
                Err(e) => {
                    error!(
                        service = "keyrx",
                        event = "linux_reader_panic",
                        component = "linux_input",
                        error = ?e,
                        "Reader thread panicked"
                    );
                }
            }
        }
    }

    pub(super) fn drain_events(&mut self) {
        while self.rx.try_recv().is_ok() {}
    }

    pub(super) fn log_stopped(&self) {
        debug!(
            service = "keyrx",
            event = "linux_stopped",
            component = "linux_input",
            "LinuxInput stopped successfully"
        );
    }
}
