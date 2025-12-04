use super::hook::HookManager;
use crate::drivers::common::cache::LruKeymapCache;
use crate::drivers::common::extract_panic_message;
use crate::engine::InputEvent;
use crossbeam_channel::Sender;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use tracing::{debug, error, warn};

pub(crate) fn spawn_hook_thread(
    running: Arc<AtomicBool>,
    panic_error: Arc<AtomicBool>,
    thread_id_store: Arc<AtomicU32>,
    tx: Sender<InputEvent>,
    cache: Arc<LruKeymapCache>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || run_hook_thread(running, panic_error, thread_id_store, tx, cache))
}

fn run_hook_thread(
    running: Arc<AtomicBool>,
    panic_error: Arc<AtomicBool>,
    thread_id_store: Arc<AtomicU32>,
    tx: Sender<InputEvent>,
    cache: Arc<LruKeymapCache>,
) {
    debug!(
        service = "keyrx",
        event = "windows_hook_thread_started",
        component = "windows_input",
        "Hook thread started"
    );
    let mut hook_manager = HookManager::new(running.clone(), thread_id_store);
    if let Err(e) = hook_manager.install(tx, cache) {
        error!(
            service = "keyrx",
            event = "windows_keyboard_hook_install_failed",
            component = "windows_input",
            error = ?e,
            "Failed to install keyboard hook"
        );
        running.store(false, Ordering::Relaxed);
        return;
    }

    let result = panic::catch_unwind(AssertUnwindSafe(|| hook_manager.run_message_loop()));
    match result {
        Ok(()) => {
            hook_manager.uninstall();
            debug!(
                service = "keyrx",
                event = "windows_hook_thread_finished",
                component = "windows_input",
                "Hook thread finished"
            );
        }
        Err(panic_info) => handle_hook_panic(&mut hook_manager, panic_error, running, panic_info),
    }
}

fn handle_hook_panic(
    hook_manager: &mut HookManager,
    panic_error: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    panic_info: Box<dyn std::any::Any + Send>,
) {
    panic_error.store(true, Ordering::SeqCst);
    running.store(false, Ordering::Relaxed);
    let panic_msg = extract_panic_message(&*panic_info);
    error!(
        service = "keyrx",
        event = "windows_hook_thread_panic",
        component = "windows_input",
        error = panic_msg,
        "Hook thread panicked"
    );
    hook_manager.uninstall();
    debug!(
        service = "keyrx",
        event = "windows_hook_uninstalled_after_panic",
        component = "windows_input",
        "Hook uninstalled after panic"
    );
    debug!(
        service = "keyrx",
        event = "windows_hook_thread_exit_after_panic",
        component = "windows_input",
        "Hook thread exiting after panic"
    );
}
