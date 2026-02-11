//! Metrics CLI command.
//!
//! This module implements the `keyrx metrics` command for querying daemon performance
//! metrics via IPC. Provides latency statistics and recent event tail.

use crate::ipc::unix_socket::UnixSocketIpc;
use crate::ipc::{DaemonIpc, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// Metrics subcommands.
#[derive(Args, Debug)]
pub struct MetricsArgs {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: MetricsCommand,

    /// Output as JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Custom socket path (defaults to /tmp/keyrx-daemon.sock).
    #[arg(long, global = true)]
    pub socket: Option<PathBuf>,
}

/// Metrics subcommands.
#[derive(Subcommand, Debug)]
pub enum MetricsCommand {
    /// Query latency metrics (min, avg, max, p95, p99).
    Latency,

    /// Tail recent events.
    Events {
        /// Number of events to retrieve (default: 100).
        #[arg(short, long, default_value = "100")]
        count: usize,

        /// Follow mode: continuously tail events (not implemented yet).
        #[arg(short, long)]
        follow: bool,
    },
}

/// JSON output structure for latency metrics.
#[derive(Serialize)]
struct LatencyOutput {
    min_us: u64,
    avg_us: u64,
    max_us: u64,
    p95_us: u64,
    p99_us: u64,
}

/// JSON output structure for events.
#[derive(Serialize)]
struct EventsOutput {
    count: usize,
    events: Vec<String>,
}

/// Execute the metrics command.
pub fn execute(args: MetricsArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        MetricsCommand::Latency => execute_latency(args.json, args.socket),
        MetricsCommand::Events { count, follow } => {
            if follow {
                return Err("Follow mode is not implemented yet".into());
            }
            execute_events(count, args.json, args.socket)
        }
    }
}

/// Execute the latency subcommand.
fn execute_latency(json: bool, socket: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Determine socket path
    let socket_path = socket.unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH));

    // Create IPC client
    let mut ipc = UnixSocketIpc::new(socket_path);

    // Send GetLatencyMetrics request
    let response = ipc.send_request(&IpcRequest::GetLatencyMetrics)?;

    // Parse response
    match response {
        IpcResponse::Latency {
            min_us,
            avg_us,
            max_us,
            p95_us,
            p99_us,
        } => {
            if json {
                print_latency_json(min_us, avg_us, max_us, p95_us, p99_us)?;
            } else {
                print_latency_human(min_us, avg_us, max_us, p95_us, p99_us);
            }
            Ok(())
        }
        IpcResponse::Error { code, message } => {
            Err(format!("Daemon error {}: {}", code, message).into())
        }
        _ => Err("Unexpected response from daemon".into()),
    }
}

/// Execute the events subcommand.
fn execute_events(
    count: usize,
    json: bool,
    socket: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine socket path
    let socket_path = socket.unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH));

    // Create IPC client
    let mut ipc = UnixSocketIpc::new(socket_path);

    // Send GetEventsTail request
    let response = ipc.send_request(&IpcRequest::GetEventsTail { count })?;

    // Parse response
    match response {
        IpcResponse::Events { events } => {
            if json {
                print_events_json(&events)?;
            } else {
                print_events_human(&events);
            }
            Ok(())
        }
        IpcResponse::Error { code, message } => {
            Err(format!("Daemon error {}: {}", code, message).into())
        }
        _ => Err("Unexpected response from daemon".into()),
    }
}

/// Print latency metrics as JSON.
fn print_latency_json(
    min_us: u64,
    avg_us: u64,
    max_us: u64,
    p95_us: u64,
    p99_us: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = LatencyOutput {
        min_us,
        avg_us,
        max_us,
        p95_us,
        p99_us,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print latency metrics in human-readable format.
fn print_latency_human(min_us: u64, avg_us: u64, max_us: u64, p95_us: u64, p99_us: u64) {
    println!("Latency Metrics:");
    println!(
        "  Min:     {} μs ({:.2} ms)",
        min_us,
        min_us as f64 / 1000.0
    );
    println!(
        "  Average: {} μs ({:.2} ms)",
        avg_us,
        avg_us as f64 / 1000.0
    );
    println!(
        "  Max:     {} μs ({:.2} ms)",
        max_us,
        max_us as f64 / 1000.0
    );
    println!(
        "  P95:     {} μs ({:.2} ms)",
        p95_us,
        p95_us as f64 / 1000.0
    );
    println!(
        "  P99:     {} μs ({:.2} ms)",
        p99_us,
        p99_us as f64 / 1000.0
    );
}

/// Print events as JSON.
fn print_events_json(events: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let output = EventsOutput {
        count: events.len(),
        events: events.to_vec(),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print events in human-readable format.
fn print_events_human(events: &[String]) {
    println!("Recent Events ({} total):", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  [{}] {}", i + 1, event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_output_format() {
        let output = LatencyOutput {
            min_us: 50,
            avg_us: 150,
            max_us: 500,
            p95_us: 350,
            p99_us: 450,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"min_us\":50"));
        assert!(json.contains("\"avg_us\":150"));
        assert!(json.contains("\"max_us\":500"));
        assert!(json.contains("\"p95_us\":350"));
        assert!(json.contains("\"p99_us\":450"));
    }

    #[test]
    fn test_events_output_format() {
        let events = vec!["event1".to_string(), "event2".to_string()];
        let output = EventsOutput {
            count: events.len(),
            events: events.clone(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"event1\""));
        assert!(json.contains("\"event2\""));
    }

    #[test]
    fn test_empty_events_output() {
        let output = EventsOutput {
            count: 0,
            events: vec![],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("\"events\":[]"));
    }
}
