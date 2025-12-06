use super::input::WindowsInput;
use crate::{
    bail_keyrx,
    drivers::KeyInjector,
    engine::{InputEvent, OutputAction},
    errors::{driver::*, KeyrxError},
    traits::InputSource,
};
use async_trait::async_trait;
use std::sync::atomic::Ordering;

#[async_trait]
impl<I: KeyInjector + 'static> InputSource for WindowsInput<I> {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        self.fail_if_hook_panicked()?;
        if self.is_inactive() {
            self.log_poll_when_inactive();
            bail_keyrx!(
                DRIVER_DEVICE_DISCONNECTED,
                device = "keyboard hook (stopped by emergency exit or manual stop)"
            );
        }

        #[cfg(feature = "otel-tracing")]
        let device_identity = self
            .device_identity
            .as_ref()
            .map(|d| d.to_key())
            .unwrap_or_else(|| "unknown".to_string());
        #[cfg(feature = "otel-tracing")]
        let poll_span = tracing::trace_span!(
            "driver.poll_events",
            driver = "windows",
            device = %device_identity,
            running = self.running.load(Ordering::Relaxed)
        );
        #[cfg(feature = "otel-tracing")]
        let _poll_guard = poll_span.enter();

        let mut events = Vec::new();
        while let Some(event) = self.next_event()? {
            #[cfg(feature = "otel-tracing")]
            let event_span = tracing::trace_span!(
                "driver.input_event",
                driver = "windows",
                key = ?event.key,
                pressed = event.pressed,
                timestamp_us = event.timestamp_us,
                device_id = event.device_id.as_deref().unwrap_or(""),
                is_repeat = event.is_repeat,
                is_synthetic = event.is_synthetic,
                scan_code = event.scan_code as u64,
            );
            #[cfg(feature = "otel-tracing")]
            let _event_guard = event_span.enter();
            events.push(event);
        }

        self.log_polled_events(events.len());
        Ok(events)
    }
    async fn send_output(&mut self, action: OutputAction) -> Result<(), KeyrxError> {
        if !self.running.load(Ordering::Relaxed) {
            self.log_inactive_send();
            return Ok(());
        }

        #[cfg(feature = "otel-tracing")]
        let action_label = format!("{:?}", &action);
        #[cfg(feature = "otel-tracing")]
        let send_span = tracing::trace_span!(
            "driver.send_output",
            driver = "windows",
            device = %self
                .device_identity
                .as_ref()
                .map(|d| d.to_key())
                .unwrap_or_else(|| "unknown".to_string()),
            action = %action_label
        );
        #[cfg(feature = "otel-tracing")]
        let _send_guard = send_span.enter();
        match action {
            OutputAction::KeyDown(key) => self.inject_key_action(key, true, "windows_key_down")?,
            OutputAction::KeyUp(key) => self.inject_key_action(key, false, "windows_key_up")?,
            OutputAction::KeyTap(key) => self.tap_key_action(key)?,
            OutputAction::Block => self.log_block_action(),
            OutputAction::PassThrough => self.log_passthrough_action(),
        }
        Ok(())
    }
    async fn start(&mut self) -> Result<(), KeyrxError> {
        if self.running.load(Ordering::Relaxed) {
            self.log_start_skipped();
            return Ok(());
        }

        self.log_starting();
        self.prepare_start();
        self.spawn_hook_thread();
        self.wait_for_hook_start()?;
        self.log_started();
        Ok(())
    }
    async fn stop(&mut self) -> Result<(), KeyrxError> {
        if !self.running.load(Ordering::Relaxed) {
            self.log_stop_skipped();
            return Ok(());
        }
        self.running.store(false, Ordering::Relaxed);
        self.post_quit_for_stop();
        self.join_hook_thread_for_stop();
        self.drain_events();

        // Invalidate cache entries for Windows device
        self.invalidate_cache("windows");
        tracing::debug!(
            service = "keyrx",
            event = "windows_cache_invalidated",
            component = "windows_input",
            "Invalidated cache entries"
        );

        self.log_stopped();
        Ok(())
    }
}
