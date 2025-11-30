//! Execution tests for RunCommand.
//! Focused on deterministic coverage of run() behavior.

use keyrx_core::cli::commands::RunCommand;
use keyrx_core::cli::OutputFormat;
use std::path::PathBuf;
use tokio::task::LocalSet;
use tokio::time::{sleep, timeout, Duration};

#[cfg(target_os = "linux")]
use signal_hook::consts::SIGINT;
#[cfg(target_os = "linux")]
use signal_hook::low_level::raise;

/// Ensure debug mode initializes and run() exits cleanly after SIGINT when using the mock driver.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "current_thread")]
async fn run_with_mock_exits_after_sigint_in_debug_mode() {
    let cmd = RunCommand::new(None, true, true, None, OutputFormat::Human);

    let local = LocalSet::new();
    local
        .run_until(async move {
            let handle = tokio::task::spawn_local(async move { cmd.run().await });

            tokio::task::spawn_local(async {
                sleep(Duration::from_millis(50)).await;
                raise(SIGINT).expect("should be able to raise SIGINT");
            });

            let result = timeout(Duration::from_secs(1), handle)
                .await
                .expect("join should complete");

            assert!(result.unwrap().is_ok());
        })
        .await;
}

/// Validate that invalid script paths surface an error instead of blocking indefinitely.
#[tokio::test(flavor = "current_thread")]
async fn run_with_missing_script_fails_fast() {
    let cmd = RunCommand::new(
        Some(PathBuf::from("/definitely/missing/script.rhai")),
        false,
        true,
        None,
        OutputFormat::Human,
    );

    let result = timeout(Duration::from_millis(300), cmd.run()).await;
    assert!(matches!(result, Ok(Err(_))));
}

/// Ensure Linux platform driver errors are propagated when device discovery fails.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "current_thread")]
async fn run_with_platform_driver_propagates_init_error() {
    let cmd = RunCommand::new(
        None,
        false,
        false,
        Some(PathBuf::from("/dev/nonexistent-keyrx")),
        OutputFormat::Human,
    );

    let result = timeout(Duration::from_secs(1), cmd.run()).await;
    assert!(matches!(result, Ok(Err(_))));
}
