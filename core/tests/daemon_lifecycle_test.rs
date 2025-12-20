use anyhow::{Context, Result};
use assert_cmd::cargo::CommandCargoExt;
use serde_json::{json, Value};
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

/// Test the full lifecycle of the daemon: start process -> connect -> start engine -> stop engine -> stop process.
#[tokio::test]
#[allow(clippy::print_stdout, clippy::unwrap_used, deprecated)]
async fn test_daemon_lifecycle() -> Result<()> {
    // 1. Setup
    let temp_dir = TempDir::new()?;
    let socket_path = temp_dir.path().join("daemon.sock");
    let _pid_file = temp_dir.path().join("daemon.pid");

    println!("Socket path: {:?}", socket_path);

    // 2. Start Daemon Process
    let mut cmd = Command::cargo_bin("keyrx")?;
    cmd.arg("daemon")
        .arg("--socket")
        .arg(&socket_path)
        .env("RUST_LOG", "debug");

    let mut child = cmd.spawn().context("Failed to spawn daemon")?;

    // 3. Wait for Socket
    let socket = timeout(Duration::from_secs(5), async {
        loop {
            if socket_path.exists() {
                if let Ok(stream) = UnixStream::connect(&socket_path).await {
                    return Ok::<UnixStream, anyhow::Error>(stream);
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("Timed out waiting for daemon socket")??;

    println!("Connected to daemon");

    let (reader, writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();
    let mut writer = writer;

    // Helper macro to read line with timeout
    macro_rules! read_line {
        ($timeout:expr, $err:expr) => {{
            let result = timeout(Duration::from_secs($timeout), lines.next_line()).await;
            match result {
                Ok(Ok(Some(line))) => line,
                Ok(Ok(None)) => return Err(anyhow::anyhow!("Stream ended: {}", $err)),
                Ok(Err(e)) => return Err(anyhow::anyhow!("IO Error: {}", e)),
                Err(_) => return Err(anyhow::anyhow!("Timeout: {}", $err)),
            }
        }};
    }

    // 4. Test: engine.status
    {
        let req = json!({
            "jsonrpc": "2.0",
            "method": "engine.status",
            "params": null,
            "id": 1
        });
        let req_str = serde_json::to_string(&req).unwrap();
        writer.write_all(req_str.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
    }

    loop {
        let line = read_line!(2, "Waiting for status");

        let msg: Value = serde_json::from_str(&line)?;
        if msg.get("id") == Some(&json!(1)) {
            println!("Response 1: {}", line);
            assert_eq!(msg["result"]["status"], "stopped");
            break;
        }
    }

    // 5. Test: engine.start
    {
        let req = json!({
            "jsonrpc": "2.0",
            "method": "engine.start",
            "params": {"use_all_devices": false},
            "id": 2
        });
        let req_str = serde_json::to_string(&req).unwrap();
        writer.write_all(req_str.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
    }

    loop {
        let line = read_line!(5, "Waiting for start");

        println!("Received: {}", line);
        let msg: Value = serde_json::from_str(&line)?;

        if msg.get("id") == Some(&json!(2)) {
            assert!(msg.get("error").is_none(), "engine.start failed: {:?}", msg);
            assert_eq!(msg["result"]["status"], "started");
            break;
        }
    }

    // 6. Test: engine.stop
    {
        let req = json!({
            "jsonrpc": "2.0",
            "method": "engine.stop",
            "params": null,
            "id": 3
        });
        let req_str = serde_json::to_string(&req).unwrap();
        writer.write_all(req_str.as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
    }

    loop {
        let line = read_line!(5, "DEADLOCK DETECTED: Waiting for stop");

        println!("Response 3 candidate: {}", line);
        let msg: Value = serde_json::from_str(&line)?;

        if msg.get("id") == Some(&json!(3)) {
            assert!(msg.get("error").is_none(), "engine.stop failed");
            assert_eq!(msg["result"]["status"], "stopped");
            break;
        }
    }

    // 7. Cleanup
    let _ = child.kill();
    let _ = temp_dir.close();

    println!("Test passed successfully");
    Ok(())
}
