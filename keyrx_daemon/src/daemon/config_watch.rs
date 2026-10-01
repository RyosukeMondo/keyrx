//! Hot reload on file edit: when the loaded profile's `.rhai` source changes
//! on disk (an editor save, `git checkout`, a script), recompile it and ask
//! the daemon to reload - no `profiles activate`, no restart.
//!
//! The decision of WHICH config is live stays in [`super::live_config`]; the
//! watcher only recompiles the loaded profile's source and raises the same
//! reload request SIGHUP and the web UI raise. A source that fails to compile
//! keeps the previous `.krx` (and so the running config), logs the compiler's
//! `file:line` error and reports it as `config_error` in status/doctor on
//! every transport until a later load succeeds (the daemon's reload clears it).
//!
//! Plain polling of one file's mtime (every [`POLL_INTERVAL`]) on purpose: no
//! extra dependency, works on every platform and over network file systems.
//! A change must be seen on two consecutive polls before it is compiled, so
//! an editor's half-written file is not compiled.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use log::{error, info, warn};

use super::DaemonSharedState;
use crate::config::ProfileManager;

/// How often the loaded profile's source is checked.
pub const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// What one [`ProfileWatcher::tick`] did.
#[derive(Debug, PartialEq, Eq)]
pub enum WatchEvent {
    /// The source was recompiled and a daemon reload was requested.
    Reloaded { profile: String, compile_ms: u64 },
    /// Something else (web UI save, `profiles activate`) already compiled it.
    AlreadyCompiled { profile: String },
    /// The edited source does not compile; the running config is unchanged.
    CompileFailed { profile: String, error: String },
}

/// Polls the loaded profile's `.rhai` and reloads the daemon when it changes.
pub struct ProfileWatcher {
    manager: Arc<ProfileManager>,
    shared: Arc<DaemonSharedState>,
    /// The profile and `.rhai` mtime last acted on (the baseline).
    seen: Option<(String, SystemTime)>,
    /// An mtime seen once and waiting to be seen again (settled).
    candidate: Option<SystemTime>,
}

impl ProfileWatcher {
    /// A watcher over the profile `shared` reports as loaded.
    pub fn new(manager: Arc<ProfileManager>, shared: Arc<DaemonSharedState>) -> Self {
        Self {
            manager,
            shared,
            seen: None,
            candidate: None,
        }
    }

    /// One poll. Returns what it did, if anything.
    pub fn tick(&mut self) -> Option<WatchEvent> {
        // A pinned `--config FILE` has no profile: nothing to watch.
        let profile = self.shared.get_active_profile()?;
        let source = self.manager.profiles_dir().join(format!("{profile}.rhai"));
        let mtime = modified(&source)?;

        let baseline = self.seen.as_ref().filter(|(name, _)| *name == profile);
        let Some((_, last)) = baseline else {
            // First sight of this profile (startup or a switch): just remember.
            self.seen = Some((profile, mtime));
            self.candidate = None;
            return None;
        };
        if *last == mtime {
            self.candidate = None;
            return None;
        }
        if self.candidate != Some(mtime) {
            self.candidate = Some(mtime);
            return None;
        }
        self.candidate = None;
        self.seen = Some((profile.clone(), mtime));
        let event = self.reload(profile, &source, mtime);
        if let WatchEvent::CompileFailed { error, profile } = &event {
            self.shared.report_config_error(format!(
                "profile '{profile}' changed on disk but does not compile, keeping the \
                 running configuration: {error}"
            ));
        }
        Some(event)
    }

    fn reload(&self, profile: String, source: &Path, source_mtime: SystemTime) -> WatchEvent {
        let compiled = self.manager.profiles_dir().join(format!("{profile}.krx"));
        if modified(&compiled).is_some_and(|krx| krx >= source_mtime) {
            return WatchEvent::AlreadyCompiled { profile };
        }
        match self.manager.recompile(&profile) {
            Ok(compile_ms) => {
                self.shared.request_reload();
                WatchEvent::Reloaded {
                    profile,
                    compile_ms,
                }
            }
            Err(e) => WatchEvent::CompileFailed {
                error: format!("{}: {e}", source.display()),
                profile,
            },
        }
    }
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Runs a [`ProfileWatcher`] on its own thread until `running` is cleared.
pub fn spawn(
    manager: Arc<ProfileManager>,
    shared: Arc<DaemonSharedState>,
    running: Arc<AtomicBool>,
    interval: Duration,
) -> JoinHandle<()> {
    let dir: PathBuf = manager.profiles_dir();
    info!(
        "Watching the loaded profile's source in {} for edits (disable with --no-watch)",
        dir.display()
    );
    std::thread::spawn(move || {
        let mut watcher = ProfileWatcher::new(manager, shared);
        while running.load(Ordering::SeqCst) {
            match watcher.tick() {
                Some(WatchEvent::Reloaded {
                    profile,
                    compile_ms,
                }) => info!(
                    "Profile '{profile}' changed on disk: recompiled in {compile_ms}ms, reloading"
                ),
                Some(WatchEvent::AlreadyCompiled { profile }) => {
                    info!("Profile '{profile}' changed on disk (already compiled)");
                }
                Some(WatchEvent::CompileFailed { profile, error }) => error!(
                    "Profile '{profile}' changed on disk but does not compile; keeping the \
                     running configuration. {error}"
                ),
                None => {}
            }
            std::thread::sleep(interval);
        }
        warn!("Profile watcher stopped");
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProfileTemplate;
    use tempfile::TempDir;

    const REMAP_A_TO_B: &str = "device_start(\"*\");\n  map(\"VK_A\", \"VK_B\");\ndevice_end();\n";

    struct Fixture {
        dir: TempDir,
        manager: Arc<ProfileManager>,
        shared: Arc<DaemonSharedState>,
        watcher: ProfileWatcher,
    }

    /// A manager with profile "p" compiled and "loaded" by a fake daemon.
    fn fixture() -> Fixture {
        let dir = TempDir::new().unwrap();
        let manager = Arc::new(ProfileManager::new(dir.path().to_path_buf()).unwrap());
        manager.create("p", ProfileTemplate::Blank).unwrap();
        manager.set_config("p", REMAP_A_TO_B).unwrap();
        assert!(manager.activate("p").unwrap().success);
        let shared = Arc::new(DaemonSharedState::new(
            Arc::new(AtomicBool::new(true)),
            Some("p".to_string()),
            PathBuf::new(),
            0,
        ));
        let watcher = ProfileWatcher::new(Arc::clone(&manager), Arc::clone(&shared));
        Fixture {
            dir,
            manager,
            shared,
            watcher,
        }
    }

    fn edit_source(f: &Fixture, text: &str) {
        std::thread::sleep(Duration::from_millis(20)); // distinct mtime
        std::fs::write(f.dir.path().join("profiles/p.rhai"), text).unwrap();
    }

    #[test]
    fn an_untouched_source_never_reloads() {
        let mut f = fixture();
        assert_eq!(f.watcher.tick(), None); // baseline
        assert_eq!(f.watcher.tick(), None);
        assert!(!f.shared.take_reload_request());
    }

    #[test]
    fn an_edit_is_compiled_once_it_settles_and_reloads_the_daemon() {
        let mut f = fixture();
        f.watcher.tick(); // baseline
        edit_source(
            &f,
            "device_start(\"*\");\n  map(\"VK_A\", \"VK_C\");\ndevice_end();\n",
        );
        assert_eq!(f.watcher.tick(), None, "first sighting only waits");
        match f.watcher.tick() {
            Some(WatchEvent::Reloaded { profile, .. }) => assert_eq!(profile, "p"),
            other => panic!("expected a reload, got {other:?}"),
        }
        assert!(f.shared.take_reload_request());
        // The same edit is not acted on twice.
        assert_eq!(f.watcher.tick(), None);
        assert!(f.manager.get("p").is_some());
    }

    #[test]
    fn a_broken_edit_keeps_the_old_config_and_reports_the_error() {
        let mut f = fixture();
        f.watcher.tick();
        let krx = f.dir.path().join("profiles/p.krx");
        let before = std::fs::read(&krx).unwrap();
        edit_source(&f, "device_start(\"*\"\n  map(\"VK_A\" \"VK_C\");\n");
        f.watcher.tick();
        match f.watcher.tick() {
            Some(WatchEvent::CompileFailed { error, .. }) => {
                assert!(error.contains("p.rhai"), "error names the file: {error}");
            }
            other => panic!("expected a compile failure, got {other:?}"),
        }
        let reported = f.shared.get_config_error().expect("config_error is set");
        assert!(
            reported.contains("p.rhai:2"),
            "carries file:line: {reported}"
        );
        assert!(reported.contains("p.rhai"), "{reported}");
        assert!(
            reported.contains("keeping the running configuration"),
            "{reported}"
        );
        assert!(
            reported.contains(" (at 20"),
            "carries a timestamp: {reported}"
        );
        assert_eq!(
            std::fs::read(&krx).unwrap(),
            before,
            ".krx must be untouched"
        );
        assert!(!f.shared.take_reload_request());
    }

    #[test]
    fn switching_profiles_resets_the_baseline_instead_of_reloading() {
        let mut f = fixture();
        f.watcher.tick();
        f.manager.create("q", ProfileTemplate::Blank).unwrap();
        f.shared
            .set_active_config(Some("q".to_string()), PathBuf::new());
        assert_eq!(f.watcher.tick(), None);
        assert_eq!(f.watcher.tick(), None);
    }

    #[test]
    fn a_pinned_config_file_is_not_watched() {
        let mut f = fixture();
        f.shared.set_active_config(None, PathBuf::from("/x.krx"));
        assert_eq!(f.watcher.tick(), None);
    }
}
