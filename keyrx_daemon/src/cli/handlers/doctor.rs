//! `doctor` subcommand: diagnoses the common reasons keyrx does not work on
//! this machine (device access, uinput, udev rules, config, whether the
//! daemon is running) and prints the exact fix for each - one command
//! instead of five, and no silent "0 keyboards" left unexplained.

use crate::cli::dispatcher::exit_codes;

/// One diagnostic result: a name, whether it passed, and a human message
/// (the fix, when it did not).
struct Check {
    name: &'static str,
    ok: bool,
    detail: String,
}

fn check(name: &'static str, ok: bool, detail: impl Into<String>) -> Check {
    Check {
        name,
        ok,
        detail: detail.into(),
    }
}

/// Handles the `doctor` subcommand.
pub fn handle_doctor(json: bool) -> Result<(), (i32, String)> {
    let checks = run_checks();
    if json {
        print_json(&checks);
    } else {
        print_human(&checks);
    }
    if checks.iter().all(|c| c.ok) {
        Ok(())
    } else {
        // Findings are already printed; the exit code alone signals "check
        // failed" to scripts.
        Err((exit_codes::PERMISSION_ERROR, String::new()))
    }
}

#[cfg(target_os = "linux")]
fn run_checks() -> Vec<Check> {
    vec![
        check_group_membership("input", "reading /dev/input/event*"),
        check_uinput_device(),
        check_udev_rule(),
        check_keyboards(),
        check_emergency_stop(),
        check_config(),
        check_daemon_running(),
        check_web_port(),
    ]
}

#[cfg(not(target_os = "linux"))]
fn run_checks() -> Vec<Check> {
    vec![check(
        "platform",
        false,
        "`doctor` only has Linux-specific checks implemented so far",
    )]
}

/// A group both configured (`/etc/group`) and effective in this session
/// (`getgroups()`) - the gap between the two is exactly the "added to the
/// group but haven't logged out and back in yet" trap.
#[cfg(target_os = "linux")]
fn check_group_membership(group: &'static str, needed_for: &str) -> Check {
    use nix::unistd::{getgroups, Group};

    let Ok(Some(g)) = Group::from_name(group) else {
        return check(
            group_check_name(group),
            false,
            format!("group '{group}' does not exist - reinstall udev rules: scripts/install.sh"),
        );
    };
    let me = whoami();
    let in_member_list = g.mem.contains(&me);
    let effective = getgroups()
        .map(|gids| gids.contains(&g.gid))
        .unwrap_or(false);

    if effective {
        check(group_check_name(group), true, "active in this session")
    } else if in_member_list {
        check(
            group_check_name(group),
            false,
            format!(
                "configured in /etc/group but NOT active in this session (needed for \
                 {needed_for}) - log out and back in, or run: newgrp {group}"
            ),
        )
    } else {
        check(
            group_check_name(group),
            false,
            format!(
                "not a member (needed for {needed_for}) - run: sudo usermod -aG {group} $USER, \
                 then log out and back in"
            ),
        )
    }
}

#[cfg(target_os = "linux")]
fn group_check_name(group: &str) -> &'static str {
    match group {
        "input" => "input group",
        "uinput" => "uinput group",
        _ => "group",
    }
}

/// The current user's login name, for matching against a group's member list.
#[cfg(target_os = "linux")]
fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_default()
}

/// Always passes: it is here so the escape hatch is visible while diagnosing,
/// before anything has gone wrong with the keyboard.
#[cfg(target_os = "linux")]
fn check_emergency_stop() -> Check {
    let describe = crate::cli::config_dir::get_config_dir()
        .map_err(|e| e.to_string())
        .and_then(|dir| {
            crate::daemon::options::RuntimeOptions::from_environment(&dir, &Default::default())
        });
    match describe {
        Ok(options) => check(
            "emergency stop",
            true,
            format!(
                "{}. Either releases every keyboard and stops the daemon",
                options.emergency.describe()
            ),
        ),
        Err(e) => check("emergency stop", false, format!("invalid setting: {e}")),
    }
}

#[cfg(target_os = "linux")]
fn check_uinput_device() -> Check {
    use nix::unistd::{access, AccessFlags};
    use std::path::Path;

    let path = Path::new("/dev/uinput");
    if !path.exists() {
        return check(
            "/dev/uinput",
            false,
            "does not exist - is the uinput kernel module loaded? sudo modprobe uinput",
        );
    }
    match access(path, AccessFlags::R_OK | AccessFlags::W_OK) {
        Ok(()) => check("/dev/uinput", true, "read/write access OK"),
        Err(e) => check(
            "/dev/uinput",
            false,
            format!(
                "not accessible ({e}) - the udev rule must give your group (input or \
                 uinput) rw access; check `ls -l /dev/uinput` and the groups above"
            ),
        ),
    }
}

#[cfg(target_os = "linux")]
fn check_udev_rule() -> Check {
    use std::path::Path;

    let path = Path::new("/etc/udev/rules.d/99-keyrx.rules");
    if path.exists() {
        check("udev rule", true, path.display().to_string())
    } else {
        check(
            "udev rule",
            false,
            "not installed at /etc/udev/rules.d/99-keyrx.rules - run scripts/install.sh, or: \
             sudo install -m 644 keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/",
        )
    }
}

/// How many enumerated keyboards this process can actually open, without
/// paying the ~15ms-per-device open+close cost `list-devices` avoids (see
/// `device_manager::linux_enum`) - a cheap `access()` per device instead.
#[cfg(target_os = "linux")]
fn check_keyboards() -> Check {
    use crate::device_manager::enumerate_keyboards;
    use nix::unistd::{access, AccessFlags};

    let keyboards = match enumerate_keyboards() {
        Ok(k) => k,
        Err(e) => return check("keyboards", false, format!("failed to enumerate: {e}")),
    };
    if keyboards.is_empty() {
        return check(
            "keyboards",
            false,
            "no keyboard-like devices found on this system",
        );
    }
    let unreadable: Vec<String> = keyboards
        .iter()
        .filter(|k| access(&k.path, AccessFlags::R_OK).is_err())
        .map(|k| format!("{} ({})", k.path.display(), k.name))
        .collect();
    if unreadable.is_empty() {
        check("keyboards", true, format!("{} readable", keyboards.len()))
    } else {
        check(
            "keyboards",
            false,
            format!(
                "{}/{} not readable: {} - see the input group check above",
                unreadable.len(),
                keyboards.len(),
                unreadable.join(", ")
            ),
        )
    }
}

#[cfg(target_os = "linux")]
fn check_config() -> Check {
    let config_dir = match crate::cli::config_dir::get_config_dir() {
        Ok(d) => d,
        Err(e) => return check("config dir", false, format!("cannot resolve: {e}")),
    };
    if !config_dir.exists() {
        return check(
            "config dir",
            false,
            format!(
                "{} does not exist yet - run `keyrx_daemon run` once to create it",
                config_dir.display()
            ),
        );
    }
    let manager = match crate::config::ProfileManager::new(config_dir.clone()) {
        Ok(m) => m,
        Err(e) => return check("config dir", false, format!("cannot read profiles: {e}")),
    };
    match manager.get_active() {
        Ok(Some(name)) => check(
            "config dir",
            true,
            format!("{} (active profile: {name})", config_dir.display()),
        ),
        Ok(None) => check(
            "config dir",
            true,
            format!(
                "{} (no active profile - no keyboard is grabbed until one is activated)",
                config_dir.display()
            ),
        ),
        Err(e) => check(
            "config dir",
            false,
            format!("active profile is broken: {e} - fix or reactivate a profile"),
        ),
    }
}

#[cfg(target_os = "linux")]
fn check_daemon_running() -> Check {
    use crate::ipc::client::IpcClient;
    use crate::ipc::{DaemonIpc, IpcEndpoint, IpcRequest, IpcResponse};

    let mut ipc = IpcClient::new(IpcEndpoint::from_cli(None));
    match ipc.send_request(&IpcRequest::GetStatus) {
        Ok(IpcResponse::Status {
            running,
            device_count,
            config_error: Some(error),
            ..
        }) if running => check(
            "daemon",
            false,
            format!(
                "running, but the last config change did not load ({error}); {device_count} device(s) \
                 grabbed (the previous config stays live; with none, no keyboard is grabbed) - \
                 fix the profile or activate another"
            ),
        ),
        Ok(IpcResponse::Status {
            running,
            active_profile,
            device_count,
            ..
        }) if running => check(
            "daemon",
            true,
            format!(
                "running (profile: {}, {device_count} device(s) grabbed)",
                active_profile.as_deref().unwrap_or("-")
            ),
        ),
        _ => check(
            "daemon",
            false,
            "not running (or not reachable over IPC) - start it: keyrx_daemon run, or \
             systemctl --user start keyrx",
        ),
    }
}

#[cfg(target_os = "linux")]
fn check_web_port() -> Check {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let config = match crate::daemon_config::DaemonConfig::from_env() {
        Ok(c) => c,
        Err(e) => return check("web UI", false, format!("cannot resolve config: {e}")),
    };
    let addr: SocketAddr = match format!("127.0.0.1:{}", config.port).parse() {
        Ok(a) => a,
        Err(e) => return check("web UI", false, format!("bad address: {e}")),
    };
    match TcpStream::connect_timeout(&addr, Duration::from_millis(300)) {
        Ok(_) => check("web UI", true, config.web_url()),
        Err(_) => check(
            "web UI",
            false,
            format!(
                "not listening on {} - daemon not running, or KEYRX_PORT overridden",
                config.port
            ),
        ),
    }
}

fn print_human(checks: &[Check]) {
    println!("keyrx doctor");
    println!();
    for c in checks {
        let glyph = if c.ok { "[OK]  " } else { "[FAIL]" };
        println!("{glyph} {:<16} {}", c.name, c.detail);
    }
    println!();
    let failed = checks.iter().filter(|c| !c.ok).count();
    if failed == 0 {
        println!("All checks passed.");
    } else {
        println!("{failed} check(s) failed - see the fixes above.");
    }
}

fn print_json(checks: &[Check]) {
    let items: Vec<serde_json::Value> = checks
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "ok": c.ok,
                "detail": c.detail,
            })
        })
        .collect();
    let out = serde_json::json!({
        "ok": checks.iter().all(|c| c.ok),
        "checks": items,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ok_prints_summary_without_panicking() {
        let checks = vec![check("a", true, "fine"), check("b", true, "also fine")];
        print_human(&checks);
        print_json(&checks);
    }

    #[test]
    fn a_failure_is_reported_with_its_fix() {
        let checks = vec![check("a", false, "broken - do X")];
        assert!(!checks.iter().all(|c| c.ok));
        print_human(&checks);
    }
}
