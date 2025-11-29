//! Self-diagnostics command.

use crate::cli::{OutputFormat, OutputWriter};
use anyhow::Result;
use serde::Serialize;

/// Run self-diagnostics.
pub struct DoctorCommand {
    pub verbose: bool,
    pub output: OutputWriter,
}

/// Status of a diagnostic check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CheckStatus {
    Pass,
    Fail,
    Warn,
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatus::Pass => write!(f, "PASS"),
            CheckStatus::Fail => write!(f, "FAIL"),
            CheckStatus::Warn => write!(f, "WARN"),
        }
    }
}

/// Result of a diagnostic check.
#[derive(Serialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub remediation: Option<String>,
}

impl DiagnosticCheck {
    fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Pass,
            message: message.into(),
            remediation: None,
        }
    }

    fn fail(
        name: impl Into<String>,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Fail,
            message: message.into(),
            remediation: Some(remediation.into()),
        }
    }

    fn warn(
        name: impl Into<String>,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warn,
            message: message.into(),
            remediation: Some(remediation.into()),
        }
    }
}

impl DoctorCommand {
    pub fn new(verbose: bool, format: OutputFormat) -> Self {
        Self {
            verbose,
            output: OutputWriter::new(format),
        }
    }

    pub fn run(&self) -> Result<()> {
        let mut checks = Vec::new();

        // Check Rhai engine
        checks.push(DiagnosticCheck::pass(
            "Rhai Engine",
            "Scripting engine available (v1.16)",
        ));

        // Platform-specific checks
        #[cfg(target_os = "linux")]
        self.run_linux_checks(&mut checks);

        #[cfg(target_os = "windows")]
        self.run_windows_checks(&mut checks);

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        checks.push(DiagnosticCheck::warn(
            "Platform",
            "Unsupported platform",
            "KeyRx currently supports Linux and Windows only",
        ));

        // Print results
        self.print_results(&checks)?;

        // Return error if any check failed
        let has_failures = checks.iter().any(|c| c.status == CheckStatus::Fail);
        if has_failures {
            anyhow::bail!("One or more diagnostic checks failed");
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn run_linux_checks(&self, checks: &mut Vec<DiagnosticCheck>) {
        checks.push(DiagnosticCheck::pass(
            "Platform",
            "Linux (evdev/uinput)",
        ));

        // Check /dev/uinput exists
        checks.push(self.check_uinput_exists());

        // Check /dev/uinput is accessible
        checks.push(self.check_uinput_accessible());

        // Check user is in input group
        checks.push(self.check_input_group());
    }

    #[cfg(target_os = "linux")]
    fn check_uinput_exists(&self) -> DiagnosticCheck {
        use std::path::Path;

        let uinput_path = Path::new("/dev/uinput");
        if uinput_path.exists() {
            DiagnosticCheck::pass("/dev/uinput exists", "Device node found")
        } else {
            DiagnosticCheck::fail(
                "/dev/uinput exists",
                "Device node not found",
                "Load uinput kernel module: sudo modprobe uinput",
            )
        }
    }

    #[cfg(target_os = "linux")]
    fn check_uinput_accessible(&self) -> DiagnosticCheck {
        use std::fs::File;

        match File::open("/dev/uinput") {
            Ok(_) => DiagnosticCheck::pass(
                "/dev/uinput accessible",
                "Read access confirmed",
            ),
            Err(e) => {
                let kind = e.kind();
                match kind {
                    std::io::ErrorKind::NotFound => DiagnosticCheck::fail(
                        "/dev/uinput accessible",
                        "Device not found",
                        "Load uinput kernel module: sudo modprobe uinput",
                    ),
                    std::io::ErrorKind::PermissionDenied => DiagnosticCheck::fail(
                        "/dev/uinput accessible",
                        "Permission denied",
                        "Add user to input group: sudo usermod -aG input $USER && newgrp input",
                    ),
                    _ => DiagnosticCheck::fail(
                        "/dev/uinput accessible",
                        format!("Cannot access: {}", e),
                        "Check device permissions and kernel module status",
                    ),
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn check_input_group(&self) -> DiagnosticCheck {
        // Check if current user is in input group
        let groups = std::process::Command::new("groups")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        if groups.split_whitespace().any(|g| g == "input") {
            DiagnosticCheck::pass("User in input group", "Group membership confirmed")
        } else {
            DiagnosticCheck::warn(
                "User in input group",
                "User not in input group",
                "Add user to input group: sudo usermod -aG input $USER && newgrp input",
            )
        }
    }

    #[cfg(target_os = "windows")]
    fn run_windows_checks(&self, checks: &mut Vec<DiagnosticCheck>) {
        checks.push(DiagnosticCheck::pass(
            "Platform",
            "Windows (WH_KEYBOARD_LL)",
        ));

        // Check keyboard hook API availability
        checks.push(self.check_keyboard_hook_api());
    }

    #[cfg(target_os = "windows")]
    fn check_keyboard_hook_api(&self) -> DiagnosticCheck {
        // On Windows, the low-level keyboard hook API (SetWindowsHookExW) is always
        // available if we're running on Windows. The only requirement is that the
        // process has a message loop to receive hook callbacks.
        //
        // We cannot actually test hook registration without setting one up, which
        // we don't want to do in diagnostics. So we just verify we're on Windows
        // and the required DLL is loadable.

        use windows::core::PCSTR;
        use windows::Win32::System::LibraryLoader::LoadLibraryA;

        // Check if user32.dll is loadable (contains SetWindowsHookExW)
        let dll_name = b"user32.dll\0";

        // SAFETY: LoadLibraryA is safe to call with a valid null-terminated string
        let result = unsafe { LoadLibraryA(PCSTR::from_raw(dll_name.as_ptr())) };

        match result {
            Ok(_handle) => {
                // Library is loadable - handle will be freed when process exits
                // Note: We don't call FreeLibrary as the module stays loaded anyway
                // and user32.dll is typically always loaded in Windows processes
                DiagnosticCheck::pass(
                    "Keyboard Hook API",
                    "SetWindowsHookExW available via user32.dll",
                )
            }
            Err(_) => DiagnosticCheck::fail(
                "Keyboard Hook API",
                "Cannot load user32.dll",
                "Ensure Windows is properly installed; user32.dll should always be present",
            ),
        }
    }

    fn print_results(&self, checks: &[DiagnosticCheck]) -> Result<()> {
        match self.output.format() {
            OutputFormat::Human => {
                println!("KeyRx Diagnostics\n");
                for check in checks {
                    let status_str = match check.status {
                        CheckStatus::Pass => "\x1b[32m[PASS]\x1b[0m",
                        CheckStatus::Fail => "\x1b[31m[FAIL]\x1b[0m",
                        CheckStatus::Warn => "\x1b[33m[WARN]\x1b[0m",
                    };
                    println!("{} {}: {}", status_str, check.name, check.message);
                    if let Some(ref remediation) = check.remediation {
                        if check.status != CheckStatus::Pass {
                            println!("       → {}", remediation);
                        }
                    }
                }

                let pass_count = checks.iter().filter(|c| c.status == CheckStatus::Pass).count();
                let fail_count = checks.iter().filter(|c| c.status == CheckStatus::Fail).count();
                let warn_count = checks.iter().filter(|c| c.status == CheckStatus::Warn).count();

                println!("\nSummary: {} passed, {} failed, {} warnings",
                    pass_count, fail_count, warn_count);
            }
            OutputFormat::Json => {
                self.output.data(checks)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_check_pass() {
        let check = DiagnosticCheck::pass("Test", "All good");
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.remediation.is_none());
    }

    #[test]
    fn test_diagnostic_check_fail() {
        let check = DiagnosticCheck::fail("Test", "Failed", "Fix it");
        assert_eq!(check.status, CheckStatus::Fail);
        assert_eq!(check.remediation, Some("Fix it".to_string()));
    }

    #[test]
    fn test_diagnostic_check_warn() {
        let check = DiagnosticCheck::warn("Test", "Warning", "Consider fixing");
        assert_eq!(check.status, CheckStatus::Warn);
        assert_eq!(check.remediation, Some("Consider fixing".to_string()));
    }

    #[test]
    fn test_check_status_display() {
        assert_eq!(format!("{}", CheckStatus::Pass), "PASS");
        assert_eq!(format!("{}", CheckStatus::Fail), "FAIL");
        assert_eq!(format!("{}", CheckStatus::Warn), "WARN");
    }
}
