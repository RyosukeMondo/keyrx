//! Unix socket-based IPC implementation for daemon communication.

use super::{DaemonIpc, IpcError, IpcRequest, IpcResponse, DEFAULT_TIMEOUT};
use interprocess::local_socket::LocalSocketStream;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Connection state for the Unix socket IPC client.
///
/// This enum tracks the lifecycle of a socket connection to prevent
/// operations on disconnected sockets and detect state violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    /// Socket is not connected.
    Disconnected,
    /// Socket is in the process of connecting.
    Connecting,
    /// Socket is connected and ready for communication.
    Connected,
}

/// Unix socket IPC client implementation
pub struct UnixSocketIpc {
    socket_path: PathBuf,
    timeout: Duration,
    stream: Option<LocalSocketStream>,
    state: ConnectionState,
}

impl UnixSocketIpc {
    /// Create a new Unix socket IPC client
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            timeout: DEFAULT_TIMEOUT,
            stream: None,
            state: ConnectionState::Disconnected,
        }
    }

    /// Create a new Unix socket IPC client with custom timeout
    pub fn with_timeout(socket_path: PathBuf, timeout: Duration) -> Self {
        Self {
            socket_path,
            timeout,
            stream: None,
            state: ConnectionState::Disconnected,
        }
    }

    /// Connect to the daemon socket
    fn connect(&mut self) -> Result<(), IpcError> {
        // Check current state
        if self.state == ConnectionState::Connected {
            // Already connected, nothing to do
            return Ok(());
        }

        // Transition to connecting state
        self.state = ConnectionState::Connecting;

        // Check if socket file exists
        if !self.socket_path.exists() {
            self.state = ConnectionState::Disconnected;
            return Err(IpcError::SocketNotFound(
                self.socket_path.display().to_string(),
            ));
        }

        // Get the socket name based on platform support
        let name = self.socket_path.to_string_lossy();

        // Connect to the socket
        let stream = LocalSocketStream::connect(name.as_ref()).map_err(|e| {
            self.state = ConnectionState::Disconnected;
            if e.kind() == std::io::ErrorKind::ConnectionRefused {
                // The file exists (checked above) but nobody is listening.
                IpcError::StaleSocket(self.socket_path.display().to_string())
            } else {
                IpcError::IoError(e)
            }
        })?;

        // Note: LocalSocketStream doesn't support set_timeout directly
        // We'll implement timeout using a manual timeout check
        self.stream = Some(stream);
        self.state = ConnectionState::Connected;
        Ok(())
    }

    /// Send a request and receive a response
    fn send_and_receive(&mut self, request: &IpcRequest) -> Result<IpcResponse, IpcError> {
        // Ensure we're connected
        if self.state != ConnectionState::Connected || self.stream.is_none() {
            self.connect()?;
        }

        let start_time = Instant::now();

        // Get stream reference with state validation
        let stream = self.stream.as_mut().ok_or_else(|| {
            // This should never happen after successful connect(), but handle gracefully
            self.state = ConnectionState::Disconnected;
            IpcError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Socket stream not available despite connected state",
            ))
        })?;

        // Serialize request to JSON
        let json =
            serde_json::to_string(request).map_err(|e| IpcError::SerializeError(e.to_string()))?;

        // Send request (newline-delimited)
        if let Err(e) = stream.write_all(json.as_bytes()) {
            // Connection lost during write
            self.state = ConnectionState::Disconnected;
            self.stream = None;
            return Err(IpcError::IoError(e));
        }
        if let Err(e) = stream.write_all(b"\n") {
            self.state = ConnectionState::Disconnected;
            self.stream = None;
            return Err(IpcError::IoError(e));
        }
        if let Err(e) = stream.flush() {
            self.state = ConnectionState::Disconnected;
            self.stream = None;
            return Err(IpcError::IoError(e));
        }

        // Check timeout before reading
        if start_time.elapsed() >= self.timeout {
            self.state = ConnectionState::Disconnected;
            self.stream = None;
            return Err(IpcError::Timeout(self.timeout));
        }

        // Read response (newline-delimited)
        let mut response_line = String::new();
        let mut reader = BufReader::new(stream);

        if let Err(e) = reader.read_line(&mut response_line) {
            // Connection lost during read or timeout
            self.state = ConnectionState::Disconnected;
            self.stream = None;
            return if e.kind() == std::io::ErrorKind::WouldBlock
                || start_time.elapsed() >= self.timeout
            {
                Err(IpcError::Timeout(self.timeout))
            } else {
                Err(IpcError::IoError(e))
            };
        }

        // Deserialize response
        let response: IpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| IpcError::DeserializeError(e.to_string()))?;

        Ok(response)
    }
}

impl DaemonIpc for UnixSocketIpc {
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

impl Drop for UnixSocketIpc {
    fn drop(&mut self) {
        // Stream will be automatically closed when dropped
    }
}

// Skip IPC socket tests on Windows due to named pipe permission issues in VM/CI environments
// The interprocess crate's named pipes require special permissions that may not be available
// in headless/VM environments. The IPC functionality works in production but these specific
// tests fail due to environment constraints.
#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::*;
    use interprocess::local_socket::LocalSocketListener;
    use std::io::BufRead;
    use std::thread;
    use tempfile::TempDir;

    fn setup_test_socket() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();

        #[cfg(unix)]
        let socket_path = temp_dir.path().join("test.sock");

        #[cfg(windows)]
        let socket_path = {
            // On Windows, named pipes use a special namespace notation
            // The interprocess crate requires the namespace prefix (with @)
            // or the full pipe path format for Windows
            use std::time::SystemTime;
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            // Use namespaced socket with @ prefix for cross-platform compatibility
            PathBuf::from(format!("@Local\\keyrx-test-{}", timestamp))
        };

        (temp_dir, socket_path)
    }

    #[test]
    fn test_socket_not_found() {
        let (_temp_dir, socket_path) = setup_test_socket();
        let mut client = UnixSocketIpc::new(socket_path.clone());

        let result = client.send_request(&IpcRequest::GetStatus);
        assert!(matches!(result, Err(IpcError::SocketNotFound(_))));
    }

    /// Regression: a socket left behind by a crashed/killed daemon used to
    /// surface as "connection refused" instead of "daemon not running".
    #[cfg(unix)]
    #[test]
    fn test_stale_socket_reports_daemon_not_running() {
        let (_temp_dir, socket_path) = setup_test_socket();
        drop(std::os::unix::net::UnixListener::bind(&socket_path).unwrap());
        assert!(socket_path.exists(), "listener drop leaves the file behind");

        let mut client = UnixSocketIpc::new(socket_path.clone());
        let err = client.send_request(&IpcRequest::GetStatus).unwrap_err();
        assert!(matches!(err, IpcError::StaleSocket(_)), "{err:?}");
        assert_eq!(err.code(), 3005);
    }

    #[test]
    fn test_connect_and_roundtrip() {
        let (_temp_dir, socket_path) = setup_test_socket();

        // Start a simple echo server in a background thread
        let server_path = socket_path.clone();
        let server_handle = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path.to_string_lossy().as_ref())
                .expect("Failed to bind listener");

            // Accept one connection
            let mut conn = listener.accept().expect("Failed to accept connection");

            // Read request
            let mut reader = BufReader::new(&mut conn);
            let mut request_line = String::new();
            reader
                .read_line(&mut request_line)
                .expect("Failed to read request");

            // Parse request
            let _request: IpcRequest =
                serde_json::from_str(&request_line).expect("Failed to parse request");

            // Send response
            let response = IpcResponse::Status {
                running: true,
                uptime_secs: 100,
                active_profile: Some("test".to_string()),
                device_count: 1,
            };
            let json = serde_json::to_string(&response).expect("Failed to serialize response");
            conn.write_all(json.as_bytes()).unwrap();
            conn.write_all(b"\n").unwrap();
            conn.flush().unwrap();
        });

        // Wait for server to start
        thread::sleep(Duration::from_millis(100));

        // Create client and send request
        let mut client = UnixSocketIpc::new(socket_path);
        let response = client
            .send_request(&IpcRequest::GetStatus)
            .expect("Request failed");

        match response {
            IpcResponse::Status {
                running,
                uptime_secs,
                active_profile,
                device_count,
            } => {
                assert!(running);
                assert_eq!(uptime_secs, 100);
                assert_eq!(active_profile, Some("test".to_string()));
                assert_eq!(device_count, 1);
            }
            _ => panic!("Unexpected response type"),
        }

        server_handle.join().unwrap();
    }

    #[test]
    fn test_timeout_handling() {
        let (_temp_dir, socket_path) = setup_test_socket();

        // Start a server that never responds
        let server_path = socket_path.clone();
        let server_handle = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path.to_string_lossy().as_ref())
                .expect("Failed to bind listener");

            // Accept connection but never respond
            let _conn = listener.accept().expect("Failed to accept connection");
            thread::sleep(Duration::from_secs(10)); // Sleep longer than timeout
        });

        // Wait for server to start
        thread::sleep(Duration::from_millis(100));

        // Create client with short timeout
        let mut client = UnixSocketIpc::with_timeout(socket_path, Duration::from_millis(100));
        let result = client.send_request(&IpcRequest::GetStatus);

        assert!(matches!(result, Err(IpcError::Timeout(_))));

        // Let server thread finish
        drop(server_handle);
    }

    #[test]
    fn test_custom_timeout() {
        let (_temp_dir, socket_path) = setup_test_socket();
        let client = UnixSocketIpc::with_timeout(socket_path, Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_request_serialization() {
        let request = IpcRequest::GetEventsTail { count: 50 };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("get_events_tail"));
        assert!(json.contains("50"));
    }

    #[test]
    fn test_initial_state_is_disconnected() {
        let (_temp_dir, socket_path) = setup_test_socket();
        let client = UnixSocketIpc::new(socket_path);
        assert_eq!(client.state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_state_transitions_on_connection() {
        let (_temp_dir, socket_path) = setup_test_socket();

        // Start server
        let server_path = socket_path.clone();
        let server_handle = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path.to_string_lossy().as_ref())
                .expect("Failed to bind listener");
            let mut conn = listener.accept().expect("Failed to accept connection");

            // Read request and send response
            let mut reader = BufReader::new(&mut conn);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();

            let response = IpcResponse::Status {
                running: true,
                uptime_secs: 100,
                active_profile: Some("test".to_string()),
                device_count: 1,
            };
            let json = serde_json::to_string(&response).unwrap();
            conn.write_all(json.as_bytes()).unwrap();
            conn.write_all(b"\n").unwrap();
            conn.flush().unwrap();
        });

        thread::sleep(Duration::from_millis(100));

        let mut client = UnixSocketIpc::new(socket_path);

        // Initially disconnected
        assert_eq!(client.state, ConnectionState::Disconnected);

        // After successful request, should be connected
        let _ = client.send_request(&IpcRequest::GetStatus);
        assert_eq!(client.state, ConnectionState::Connected);

        server_handle.join().unwrap();
    }

    #[test]
    fn test_state_reset_on_write_error() {
        let (_temp_dir, socket_path) = setup_test_socket();

        // Start server that accepts but immediately closes
        let server_path = socket_path.clone();
        let server_handle = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path.to_string_lossy().as_ref())
                .expect("Failed to bind listener");
            let _conn = listener.accept().expect("Failed to accept connection");
            // Drop connection immediately
        });

        thread::sleep(Duration::from_millis(100));

        let mut client = UnixSocketIpc::new(socket_path);

        // Try to send request - should fail and reset state
        let result = client.send_request(&IpcRequest::GetStatus);
        assert!(result.is_err());

        // State should be back to disconnected after error
        assert_eq!(client.state, ConnectionState::Disconnected);
        assert!(client.stream.is_none());

        drop(server_handle);
    }

    #[test]
    fn test_reconnection_after_disconnect() {
        // Use two separate socket paths to avoid address-in-use errors
        let temp_dir1 = TempDir::new().unwrap();
        let socket_path1 = temp_dir1.path().join("test1.sock");

        let temp_dir2 = TempDir::new().unwrap();
        let socket_path2 = temp_dir2.path().join("test2.sock");

        // First server
        let server_path1 = socket_path1.clone();
        let server_handle1 = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path1.to_string_lossy().as_ref())
                .expect("Failed to bind listener");
            let mut conn = listener.accept().expect("Failed to accept connection");

            let mut reader = BufReader::new(&mut conn);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();

            let response = IpcResponse::Status {
                running: true,
                uptime_secs: 100,
                active_profile: Some("test".to_string()),
                device_count: 1,
            };
            let json = serde_json::to_string(&response).unwrap();
            conn.write_all(json.as_bytes()).unwrap();
            conn.write_all(b"\n").unwrap();
            conn.flush().unwrap();
        });

        thread::sleep(Duration::from_millis(100));

        let mut client = UnixSocketIpc::new(socket_path1.clone());

        // First connection succeeds
        let result = client.send_request(&IpcRequest::GetStatus);
        assert!(result.is_ok());
        assert_eq!(client.state, ConnectionState::Connected);

        server_handle1.join().unwrap();

        // Update client to use new socket path and reset state
        client.socket_path = socket_path2.clone();
        client.state = ConnectionState::Disconnected;
        client.stream = None;

        // Start second server on different socket
        let server_path2 = socket_path2.clone();
        let server_handle2 = thread::spawn(move || {
            let listener = LocalSocketListener::bind(server_path2.to_string_lossy().as_ref())
                .expect("Failed to bind listener");
            let mut conn = listener.accept().expect("Failed to accept connection");

            let mut reader = BufReader::new(&mut conn);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();

            let response = IpcResponse::Status {
                running: true,
                uptime_secs: 200,
                active_profile: Some("test2".to_string()),
                device_count: 2,
            };
            let json = serde_json::to_string(&response).unwrap();
            conn.write_all(json.as_bytes()).unwrap();
            conn.write_all(b"\n").unwrap();
            conn.flush().unwrap();
        });

        thread::sleep(Duration::from_millis(100));

        // Second connection should also succeed
        let result = client.send_request(&IpcRequest::GetStatus);
        assert!(result.is_ok());
        assert_eq!(client.state, ConnectionState::Connected);

        server_handle2.join().unwrap();
    }
}
