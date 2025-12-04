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

        let mut events = Vec::new();
        while let Some(event) = self.next_event()? {
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
        self.log_stopped();
        Ok(())
    }
}
