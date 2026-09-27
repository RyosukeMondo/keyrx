use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[cfg(unix)]
use keyrx_daemon::ipc::unix_socket::UnixSocketIpc;
#[cfg(unix)]
use keyrx_daemon::ipc::{DaemonIpc, IpcRequest, IpcResponse};
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::Duration;
#[cfg(unix)]
use tempfile::TempDir;

// Mock IPC server for benchmarking
#[cfg(unix)]
fn start_mock_ipc_server(socket_path: PathBuf) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        use std::os::unix::net::UnixListener;

        // Create the socket
        let listener = UnixListener::bind(&socket_path).expect("Failed to bind Unix socket");

        // Accept a single connection for this benchmark
        if let Ok((mut stream, _)) = listener.accept() {
            use std::io::{Read, Write};

            loop {
                // Read request
                let mut len_buf = [0u8; 4];
                if stream.read_exact(&mut len_buf).is_err() {
                    break;
                }

                let len = u32::from_le_bytes(len_buf) as usize;
                let mut buf = vec![0u8; len];

                if stream.read_exact(&mut buf).is_err() {
                    break;
                }

                // Deserialize request
                let request: IpcRequest = match serde_json::from_slice(&buf) {
                    Ok(req) => req,
                    Err(_) => break,
                };

                // Generate mock response
                let response = match request {
                    IpcRequest::GetStatus => IpcResponse::Status {
                        running: true,
                        uptime_secs: 3600,
                        active_profile: Some("default".to_string()),
                        device_count: 2,
                    },
                    IpcRequest::GetState => IpcResponse::State {
                        state: vec![false; 255],
                    },
                    IpcRequest::GetLatencyMetrics => IpcResponse::Latency {
                        min_us: 50,
                        avg_us: 100,
                        max_us: 500,
                        p95_us: 200,
                        p99_us: 300,
                    },
                    IpcRequest::GetEventsTail { count: _ } => {
                        IpcResponse::Events { events: vec![] }
                    }
                    IpcRequest::ClearEvents => IpcResponse::EventsCleared { count: 0 },
                    IpcRequest::ActivateProfile { name } => IpcResponse::ProfileActivated { name },
                };

                // Serialize and send response
                let response_json = serde_json::to_vec(&response).unwrap();
                let response_len = (response_json.len() as u32).to_le_bytes();

                if stream.write_all(&response_len).is_err()
                    || stream.write_all(&response_json).is_err()
                {
                    break;
                }
            }
        }
    })
}

/// Benchmark IPC status query roundtrip.
/// Target: <10ms for status queries
fn benchmark_ipc_status_query(c: &mut Criterion) {
    // Skip this benchmark if not on Unix (IPC is Unix-specific)
    #[cfg(not(unix))]
    {
        c.bench_function("ipc_status_query_noop", |b| {
            b.iter(|| {
                black_box(());
            });
        });
        return;
    }

    #[cfg(unix)]
    {
        // Setup: Create temp socket path
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("keyrx-bench.sock");

        // Start mock IPC server
        let server_handle = start_mock_ipc_server(socket_path.clone());

        // Give server time to start
        thread::sleep(Duration::from_millis(100));

        // Create IPC client
        let mut client = UnixSocketIpc::new(socket_path.clone());

        c.bench_function("ipc_status_query_roundtrip", |b| {
            b.iter(|| {
                // Send request and receive response
                let request = black_box(IpcRequest::GetStatus);
                let result = client.send_request(&request);

                assert!(result.is_ok(), "IPC request failed");

                if let Ok(response) = result {
                    black_box(response);
                }
            });
        });

        // Cleanup: Drop client to close connection
        drop(client);

        // Server thread will exit when connection closes
        let _ = server_handle.join();
    }
}

/// Benchmark IPC latency metrics query.
/// This tests a slightly more complex response payload.
fn benchmark_ipc_latency_metrics(c: &mut Criterion) {
    #[cfg(not(unix))]
    {
        c.bench_function("ipc_latency_metrics_noop", |b| {
            b.iter(|| {
                black_box(());
            });
        });
        return;
    }

    #[cfg(unix)]
    {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("keyrx-bench-metrics.sock");

        let server_handle = start_mock_ipc_server(socket_path.clone());
        thread::sleep(Duration::from_millis(100));

        let mut client = UnixSocketIpc::new(socket_path.clone());

        c.bench_function("ipc_latency_metrics_roundtrip", |b| {
            b.iter(|| {
                let request = black_box(IpcRequest::GetLatencyMetrics);
                let result = client.send_request(&request);

                assert!(result.is_ok(), "IPC latency metrics request failed");

                if let Ok(response) = result {
                    black_box(response);
                }
            });
        });

        drop(client);
        let _ = server_handle.join();
    }
}

/// Benchmark IPC state query.
/// This tests the largest payload (255-element boolean array).
fn benchmark_ipc_state_query(c: &mut Criterion) {
    #[cfg(not(unix))]
    {
        c.bench_function("ipc_state_query_noop", |b| {
            b.iter(|| {
                black_box(());
            });
        });
        return;
    }

    #[cfg(unix)]
    {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("keyrx-bench-state.sock");

        let server_handle = start_mock_ipc_server(socket_path.clone());
        thread::sleep(Duration::from_millis(100));

        let mut client = UnixSocketIpc::new(socket_path.clone());

        c.bench_function("ipc_state_query_roundtrip", |b| {
            b.iter(|| {
                let request = black_box(IpcRequest::GetState);
                let result = client.send_request(&request);

                assert!(result.is_ok(), "IPC state request failed");

                if let Ok(response) = result {
                    black_box(response);
                }
            });
        });

        drop(client);
        let _ = server_handle.join();
    }
}

criterion_group!(
    benches,
    benchmark_ipc_status_query,
    benchmark_ipc_latency_metrics,
    benchmark_ipc_state_query
);
criterion_main!(benches);
