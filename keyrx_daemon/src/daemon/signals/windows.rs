use crate::daemon::state::ReloadState;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug)]
pub struct SignalHandler {
    reload_state: ReloadState,
}

impl SignalHandler {
    pub fn new(reload_state: ReloadState) -> Self {
        Self { reload_state }
    }

    pub fn check_reload(&self) -> bool {
        self.reload_state.check_and_clear()
    }

    pub fn reload_state(&self) -> &ReloadState {
        &self.reload_state
    }
}

pub fn install_signal_handlers(_running: Arc<AtomicBool>) -> std::io::Result<SignalHandler> {
    // Windows doesn't use the same signals as Linux for daemon control.
    // Instead, we use the tray icon and the Win32 message loop.
    let reload_state = ReloadState::new();
    Ok(SignalHandler::new(reload_state))
}
