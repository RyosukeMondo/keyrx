//! IPC client: sends [`IpcRequest`]s to a running daemon over its
//! [`IpcEndpoint`] (Unix socket file or Windows named pipe).

use super::{DaemonIpc, IpcEndpoint, IpcError, IpcRequest, IpcResponse, DEFAULT_TIMEOUT};
use interprocess::local_socket::LocalSocketStream;
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};

/// Connection state for the IPC client.
///
/// This enum tracks the lifecycle of a connection to prevent operations on
/// disconnected streams and detect state violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    /// Not connected.
    Disconnected,
    /// In the process of connecting.
    Connecting,
    /// Connected and ready for communication.
    Connected,
}

/// IPC client used by every CLI command that talks to the daemon.
pub struct IpcClient {
    endpoint: IpcEndpoint,
    timeout: Duration,
    stream: Option<LocalSocketStream>,
    state: ConnectionState,
}

impl IpcClient {
    /// Creates a client for `endpoint` with the default timeout.
    pub fn new(endpoint: IpcEndpoint) -> Self {
        Self::with_timeout(endpoint, DEFAULT_TIMEOUT)
    }

    /// Creates a client for `endpoint` with a custom timeout.
    pub fn with_timeout(endpoint: IpcEndpoint, timeout: Duration) -> Self {
        Self {
            endpoint,
            timeout,
            stream: None,
            state: ConnectionState::Disconnected,
        }
    }

    /// Connects to the daemon endpoint.
    fn connect(&mut self) -> Result<(), IpcError> {
        if self.state == ConnectionState::Connected {
            return Ok(());
        }
        self.state = ConnectionState::Connecting;

        let connected = self.endpoint.check_present().and_then(|()| {
            LocalSocketStream::connect(self.endpoint.local_socket_name())
                .map_err(|e| self.endpoint.connect_error(e))
        });
        match connected {
            Ok(stream) => {
                // LocalSocketStream has no read timeout; `send_and_receive`
                // checks elapsed time instead.
                self.stream = Some(stream);
                self.state = ConnectionState::Connected;
                Ok(())
            }
            Err(e) => {
                self.state = ConnectionState::Disconnected;
                Err(e)
            }
        }
    }

    /// Sends a request and receives a response. Any failure drops the
    /// connection, so the next request reconnects.
    fn send_and_receive(&mut self, request: &IpcRequest) -> Result<IpcResponse, IpcError> {
        if self.state != ConnectionState::Connected || self.stream.is_none() {
            self.connect()?;
        }
        let result = self.exchange(request);
        if result.is_err() {
            self.state = ConnectionState::Disconnected;
            self.stream = None;
        }
        result
    }

    /// One newline-delimited JSON request/response on the open stream.
    fn exchange(&mut self, request: &IpcRequest) -> Result<IpcResponse, IpcError> {
        let start_time = Instant::now();
        let timeout = self.timeout;
        let stream = self.stream.as_mut().ok_or_else(|| {
            IpcError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "IPC stream not available despite connected state",
            ))
        })?;

        let json =
            serde_json::to_string(request).map_err(|e| IpcError::SerializeError(e.to_string()))?;
        stream.write_all(json.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        if start_time.elapsed() >= timeout {
            return Err(IpcError::Timeout(timeout));
        }

        let mut response_line = String::new();
        let read = BufReader::new(stream).read_line(&mut response_line);
        match read {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return Err(IpcError::Timeout(timeout))
            }
            Err(_) if start_time.elapsed() >= timeout => return Err(IpcError::Timeout(timeout)),
            Err(e) => return Err(IpcError::IoError(e)),
            Ok(_) => {}
        }

        serde_json::from_str(&response_line).map_err(|e| IpcError::DeserializeError(e.to_string()))
    }
}

impl DaemonIpc for IpcClient {
    fn send_request(&mut self, request: &IpcRequest) -> Result<IpcResponse, IpcError> {
        self.send_and_receive(request)
    }

    fn receive_response(&mut self) -> Result<IpcResponse, IpcError> {
        // This is used by the server-side (daemon) to receive requests
        // For client-side, we use send_and_receive
        Err(IpcError::IoError(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "receive_response is only for server-side use",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use interprocess::local_socket::LocalSocketListener;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;
    use tempfile::TempDir;

    /// A fresh endpoint per test: a socket file in a temp dir on Unix, a
    /// uniquely named pipe on Windows. Keep the `TempDir` alive.
    fn setup_test_endpoint() -> (TempDir, IpcEndpoint) {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let temp_dir = TempDir::new().unwrap();
        let unique = format!(
            "keyrx-client-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let endpoint = if cfg!(windows) {
            IpcEndpoint::NamedPipe(unique)
        } else {
            IpcEndpoint::SocketFile(temp_dir.path().join(format!("{unique}.sock")))
        };
        (temp_dir, endpoint)
    }

    fn status(uptime_secs: u64, profile: &str) -> IpcResponse {
        IpcResponse::Status {
            running: true,
            uptime_secs,
            active_profile: Some(profile.to_string()),
            device_count: 1,
            input_overflows: 0,
            config_error: None,
            output_device: None,
            web_server: crate::web_server_status::WebServerStatus::Up,
        }
    }

    /// Binds `endpoint`, answers one request with `response`, then hangs up.
    fn serve_once(endpoint: &IpcEndpoint, response: IpcResponse) -> thread::JoinHandle<()> {
        let listener = LocalSocketListener::bind(endpoint.local_socket_name())
            .expect("Failed to bind listener");
        thread::spawn(move || {
            let mut conn = BufReader::new(listener.accept().expect("Failed to accept"));
            let mut request_line = String::new();
            conn.read_line(&mut request_line).expect("read request");
            let _: IpcRequest = serde_json::from_str(&request_line).expect("parse request");
            let json = serde_json::to_string(&response).unwrap();
            let conn = conn.get_mut();
            conn.write_all(json.as_bytes()).unwrap();
            conn.write_all(b"\n").unwrap();
            conn.flush().unwrap();
        })
    }

    #[test]
    fn test_endpoint_not_found() {
        let (_temp_dir, endpoint) = setup_test_endpoint();
        let mut client = IpcClient::new(endpoint);

        let err = client.send_request(&IpcRequest::GetStatus).unwrap_err();
        assert!(matches!(err, IpcError::SocketNotFound(_)), "{err:?}");
        assert!(err.to_string().contains("Daemon not running"), "{err}");
    }

    /// Regression: a socket left behind by a crashed/killed daemon used to
    /// surface as "connection refused" instead of "daemon not running".
    #[cfg(unix)]
    #[test]
    fn test_stale_socket_reports_daemon_not_running() {
        let (_temp_dir, endpoint) = setup_test_endpoint();
        let IpcEndpoint::SocketFile(ref socket_path) = endpoint else {
            unreachable!()
        };
        drop(std::os::unix::net::UnixListener::bind(socket_path).unwrap());
        assert!(socket_path.exists(), "listener drop leaves the file behind");

        let mut client = IpcClient::new(endpoint.clone());
        let err = client.send_request(&IpcRequest::GetStatus).unwrap_err();
        assert!(matches!(err, IpcError::StaleSocket(_)), "{err:?}");
        assert_eq!(err.code(), 3005);
    }

    #[test]
    fn test_connect_and_roundtrip() {
        let (_temp_dir, endpoint) = setup_test_endpoint();
        let server = serve_once(&endpoint, status(100, "test"));

        let mut client = IpcClient::new(endpoint);
        let response = client
            .send_request(&IpcRequest::GetStatus)
            .expect("Request failed");
        assert_eq!(response, status(100, "test"));

        server.join().unwrap();
    }

    // Unix only: the client has no real read timeout, so on Windows this
    // would block on the pipe until the server thread gives up.
    #[cfg(unix)]
    #[test]
    fn test_timeout_handling() {
        let (_temp_dir, endpoint) = setup_test_endpoint();

        // Start a server that never responds
        let listener = LocalSocketListener::bind(endpoint.local_socket_name()).unwrap();
        let server_handle = thread::spawn(move || {
            let _conn = listener.accept().expect("Failed to accept connection");
            thread::sleep(Duration::from_secs(10)); // Sleep longer than timeout
        });

        let mut client = IpcClient::with_timeout(endpoint, Duration::from_millis(100));
        let result = client.send_request(&IpcRequest::GetStatus);

        assert!(matches!(result, Err(IpcError::Timeout(_))));

        // Let server thread finish
        drop(server_handle);
    }

    #[test]
    fn test_custom_timeout() {
        let (_temp_dir, endpoint) = setup_test_endpoint();
        let client = IpcClient::with_timeout(endpoint, Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_initial_state_is_disconnected() {
        let (_temp_dir, endpoint) = setup_test_endpoint();
        let client = IpcClient::new(endpoint);
        assert_eq!(client.state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_state_transitions_on_connection() {
        let (_temp_dir, endpoint) = setup_test_endpoint();
        let server = serve_once(&endpoint, status(100, "test"));

        let mut client = IpcClient::new(endpoint);
        assert_eq!(client.state, ConnectionState::Disconnected);

        let _ = client.send_request(&IpcRequest::GetStatus);
        assert_eq!(client.state, ConnectionState::Connected);

        server.join().unwrap();
    }

    #[test]
    fn test_state_reset_on_error() {
        let (_temp_dir, endpoint) = setup_test_endpoint();

        // Server that accepts but immediately closes
        let listener = LocalSocketListener::bind(endpoint.local_socket_name()).unwrap();
        let server = thread::spawn(move || {
            let _conn = listener.accept().expect("Failed to accept connection");
        });

        let mut client = IpcClient::new(endpoint);
        let result = client.send_request(&IpcRequest::GetStatus);
        assert!(result.is_err());

        // Any failure drops the connection
        assert_eq!(client.state, ConnectionState::Disconnected);
        assert!(client.stream.is_none());

        server.join().unwrap();
    }

    #[test]
    fn test_reconnection_after_disconnect() {
        let (_temp_dir1, endpoint1) = setup_test_endpoint();
        let (_temp_dir2, endpoint2) = setup_test_endpoint();

        let server1 = serve_once(&endpoint1, status(100, "test"));
        let mut client = IpcClient::new(endpoint1);
        assert!(client.send_request(&IpcRequest::GetStatus).is_ok());
        assert_eq!(client.state, ConnectionState::Connected);
        server1.join().unwrap();

        // Point the client elsewhere and reset its connection
        client.endpoint = endpoint2.clone();
        client.state = ConnectionState::Disconnected;
        client.stream = None;

        let server2 = serve_once(&endpoint2, status(200, "test2"));
        let response = client.send_request(&IpcRequest::GetStatus);
        assert_eq!(response.unwrap(), status(200, "test2"));
        assert_eq!(client.state, ConnectionState::Connected);
        server2.join().unwrap();
    }
}
