//! Metrics CLI command.
//!
//! This module implements the `keyrx metrics` command for querying daemon performance
//! metrics via IPC: latency statistics and the recent-event tail (optionally
//! followed). The records are the same `LatencyStats` / `KeyEventData` the REST
//! API returns.

use crate::ipc::unix_socket::UnixSocketIpc;
use crate::ipc::{DaemonIpc, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};
use crate::web::events::{KeyEventData, LatencyStats};
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

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

        /// Follow mode: keep printing new events until interrupted (Ctrl+C).
        #[arg(short, long)]
        follow: bool,
    },
}

/// How often follow mode polls the daemon.
const FOLLOW_POLL: Duration = Duration::from_millis(200);
/// Events requested per follow poll (the daemon's ring is bounded anyway).
const FOLLOW_BATCH: usize = 1000;

/// JSON output structure for events.
#[derive(Serialize)]
struct EventsOutput<'a> {
    count: usize,
    events: &'a [KeyEventData],
}

/// Execute the metrics command.
pub fn execute(args: MetricsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = args
        .socket
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH));
    let mut ipc = UnixSocketIpc::new(socket_path);
    match args.command {
        MetricsCommand::Latency => execute_latency(&mut ipc, args.json),
        MetricsCommand::Events { count, follow } => {
            let events = fetch_events(&mut ipc, count)?;
            print_events(&events, args.json)?;
            if follow {
                follow_events(&mut ipc, events.last(), args.json)?;
            }
            Ok(())
        }
    }
}

/// Execute the latency subcommand.
fn execute_latency(ipc: &mut UnixSocketIpc, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    match ipc.send_request(&IpcRequest::GetLatencyMetrics)? {
        IpcResponse::Latency { stats } => {
            if json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                print_latency_human(&stats);
            }
            Ok(())
        }
        IpcResponse::Error { code, message } => {
            Err(format!("Daemon error {code}: {message}").into())
        }
        _ => Err("Unexpected response from daemon".into()),
    }
}

/// The `count` most recent events, oldest first.
fn fetch_events(
    ipc: &mut UnixSocketIpc,
    count: usize,
) -> Result<Vec<KeyEventData>, Box<dyn std::error::Error>> {
    match ipc.send_request(&IpcRequest::GetEventsTail { count })? {
        IpcResponse::Events { events } => Ok(events),
        IpcResponse::Error { code, message } => {
            Err(format!("Daemon error {code}: {message}").into())
        }
        _ => Err("Unexpected response from daemon".into()),
    }
}

/// Polls the daemon and prints events newer than `last` until interrupted.
fn follow_events(
    ipc: &mut UnixSocketIpc,
    last: Option<&KeyEventData>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cursor = FollowCursor::after(last);
    loop {
        std::thread::sleep(FOLLOW_POLL);
        let batch = fetch_events(ipc, FOLLOW_BATCH)?;
        for event in cursor.take_new(batch) {
            if json {
                println!("{}", serde_json::to_string(&event)?);
            } else {
                println!("  {}", event.summary());
            }
        }
    }
}

/// Remembers the newest event already printed so repeated tails print each
/// event once. Events are ordered by timestamp (µs); several can share one,
/// so the cursor also remembers which ones at that timestamp were seen.
#[derive(Debug, Default)]
struct FollowCursor {
    timestamp: u64,
    seen_at_timestamp: Vec<KeyEventData>,
}

impl FollowCursor {
    fn after(last: Option<&KeyEventData>) -> Self {
        let mut cursor = Self::default();
        if let Some(event) = last {
            cursor.timestamp = event.timestamp;
            cursor.seen_at_timestamp.push(event.clone());
        }
        cursor
    }

    /// The events of `batch` (oldest first) not returned before.
    fn take_new(&mut self, batch: Vec<KeyEventData>) -> Vec<KeyEventData> {
        let mut fresh = Vec::new();
        for event in batch {
            if event.timestamp < self.timestamp
                || (event.timestamp == self.timestamp && self.seen_at_timestamp.contains(&event))
            {
                continue;
            }
            if event.timestamp > self.timestamp {
                self.timestamp = event.timestamp;
                self.seen_at_timestamp.clear();
            }
            self.seen_at_timestamp.push(event.clone());
            fresh.push(event);
        }
        fresh
    }
}

fn print_events(events: &[KeyEventData], json: bool) -> Result<(), Box<dyn std::error::Error>> {
    if json {
        let output = EventsOutput {
            count: events.len(),
            events,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Recent Events ({} total):", events.len());
        for (i, event) in events.iter().enumerate() {
            println!("  [{}] {}", i + 1, event.summary());
        }
    }
    Ok(())
}

/// Print latency metrics in human-readable format.
fn print_latency_human(stats: &LatencyStats) {
    let row = |name: &str, us: u64| println!("  {name:<8} {us} μs ({:.2} ms)", us as f64 / 1000.0);
    println!("Latency Metrics ({} samples):", stats.samples);
    row("Min:", stats.min);
    row("Average:", stats.avg);
    row("P50:", stats.p50);
    row("P95:", stats.p95);
    row("P99:", stats.p99);
    row("Max:", stats.max);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(input: &str, timestamp: u64) -> KeyEventData {
        KeyEventData {
            timestamp,
            key_code: input.to_string(),
            event_type: "press".to_string(),
            input: input.to_string(),
            output: input.to_string(),
            latency: 0,
            device_id: None,
            device_name: None,
            mapping_type: None,
            mapping_triggered: false,
        }
    }

    fn inputs(events: Vec<KeyEventData>) -> Vec<String> {
        events.into_iter().map(|e| e.input).collect()
    }

    #[test]
    fn follow_prints_each_event_once() {
        let mut cursor = FollowCursor::after(Some(&event("A", 10)));
        let tail = vec![event("A", 10), event("B", 20), event("C", 30)];
        assert_eq!(inputs(cursor.take_new(tail.clone())), vec!["B", "C"]);
        assert!(cursor.take_new(tail).is_empty());
    }

    #[test]
    fn follow_keeps_distinct_events_sharing_a_timestamp() {
        let mut cursor = FollowCursor::after(None);
        assert_eq!(inputs(cursor.take_new(vec![event("A", 5)])), vec!["A"]);
        let tail = vec![event("A", 5), event("B", 5), event("C", 6)];
        assert_eq!(inputs(cursor.take_new(tail)), vec!["B", "C"]);
    }

    #[test]
    fn follow_after_clear_restarts_from_new_timestamps() {
        let mut cursor = FollowCursor::after(Some(&event("A", 10)));
        // The ring was cleared; only newer events show up.
        assert_eq!(inputs(cursor.take_new(vec![event("Z", 11)])), vec!["Z"]);
    }
}
