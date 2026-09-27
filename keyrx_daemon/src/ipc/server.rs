//! IPC server: a local socket that answers [`IpcRequest`]s via
//! [`IpcCommandHandler`](super::commands::IpcCommandHandler).

use super::{IpcRequest, IpcResponse};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Binds an IPC server at `socket_path` and serves `handler` on a background
/// thread. The single way runners expose IPC (production and test mode).
pub fn spawn(
    socket_path: PathBuf,
    handler: Arc<super::commands::IpcCommandHandler>,
) -> Result<(), std::io::Error> {
    let mut server = IpcServer::new(socket_path)?;
    server.start()?;
    std::thread::spawn(move || {
        let handler_fn = Arc::new(Mutex::new(
            move |request: IpcRequest| -> Result<IpcResponse, String> {
                Ok(handler.handle(request))
            },
        ));
        if let Err(e) = server.handle_connections(handler_fn) {
            log::error!("IPC server error: {e}");
        }
    });
    Ok(())
}

/// IPC server for daemon commands
pub struct IpcServer {
    socket_path: PathBuf,
    listener: Option<LocalSocketListener>,
}

impl IpcServer {
    /// Create a new IPC server with the given socket path
    pub fn new(socket_path: PathBuf) -> Result<Self, std::io::Error> {
        Ok(Self {
            socket_path,
            listener: None,
        })
    }

    /// Start the IPC server and bind to the socket
    pub fn start(&mut self) -> Result<(), std::io::Error> {
        // Remove socket file if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        // Bind to the socket
        let listener = LocalSocketListener::bind(self.socket_path.to_string_lossy().as_ref())?;

        // Set socket permissions to 600 (owner only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.socket_path, perms)?;
        }

        self.listener = Some(listener);
        log::info!("IPC server listening on {}", self.socket_path.display());
        Ok(())
    }

    /// Handle incoming connections in a loop
    ///
    /// This function spawns a new thread for each connection and handles
    /// IPC requests. The handler closure is called for each request.
    pub fn handle_connections<F>(&self, handler: Arc<Mutex<F>>) -> Result<(), std::io::Error>
    where
        F: Fn(IpcRequest) -> Result<IpcResponse, String> + Send + 'static,
    {
        let listener = self.listener.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Server not started - call start() first",
            )
        })?;

        loop {
            match listener.accept() {
                Ok(stream) => {
                    let handler = Arc::clone(&handler);
                    std::thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, handler) {
                            log::error!("Error handling IPC client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to accept IPC connection: {}", e);
                    // Continue accepting other connections
                }
            }
        }
    }

    /// Handle a single client connection
    fn handle_client<F>(
        stream: LocalSocketStream,
        handler: Arc<Mutex<F>>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(IpcRequest) -> Result<IpcResponse, String> + Send + 'static,
    {
        // Newline-delimited JSON, any number of requests per connection until
        // the client hangs up. (Answering one request and dropping the stream
        // broke clients that reuse their connection, e.g. `metrics --follow`.)
        let mut reader = BufReader::new(stream);
        loop {
            let mut request_line = String::new();
            if reader.read_line(&mut request_line)? == 0 {
                return Ok(()); // EOF: client closed the connection
            }
            let request: IpcRequest = serde_json::from_str(request_line.trim())?;
            log::debug!("Received IPC request: {:?}", request);

            let response = {
                let handler_guard = handler.blocking_lock();
                handler_guard(request).unwrap_or_else(|message| IpcResponse::Error {
                    code: 5000,
                    message,
                })
            };

            let stream = reader.get_mut();
            let response_json = serde_json::to_string(&response)?;
            stream.write_all(response_json.as_bytes())?;
            stream.write_all(b"\n")?;
            stream.flush()?;
            log::debug!("Sent IPC response");
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Clean up socket file
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                log::warn!(
                    "Failed to remove socket file {}: {}",
                    self.socket_path.display(),
                    e
                );
            } else {
                log::info!("Cleaned up socket file {}", self.socket_path.display());
            }
        }
    }
}

/// Get the test mode IPC socket path for the current process
pub fn get_test_socket_path() -> PathBuf {
    let pid = std::process::id();
    PathBuf::from(format!("/tmp/keyrx-test-{}.sock", pid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_format() {
        let path = get_test_socket_path();
        let path_str = path.to_str().unwrap();
        assert!(path_str.starts_with("/tmp/keyrx-test-"));
        assert!(path_str.ends_with(".sock"));
    }

    #[test]
    fn test_server_creation() {
        let socket_path = PathBuf::from("/tmp/keyrx-test-unittest.sock");
        let server = IpcServer::new(socket_path.clone());
        assert!(server.is_ok());
        let server = server.unwrap();
        assert_eq!(server.socket_path(), &socket_path);
    }

    // Integration test for start/stop would require actual socket creation
    // which is tested at a higher level

    /// Regression: the server answered one request per connection, so a
    /// client reusing its connection (`metrics events --follow`) got EPIPE.
    #[test]
    #[cfg(unix)]
    fn test_several_requests_on_one_connection() {
        use crate::ipc::unix_socket::UnixSocketIpc;
        use crate::ipc::DaemonIpc;
        use crate::services::DaemonQueryService;

        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("reuse.sock");
        let manager =
            Arc::new(crate::config::ProfileManager::new(dir.path().to_path_buf()).unwrap());
        let handler = Arc::new(super::super::commands::IpcCommandHandler::new(
            manager,
            Arc::new(DaemonQueryService::without_daemon()),
        ));
        spawn(socket.clone(), handler).unwrap();

        let mut client = UnixSocketIpc::new(socket);
        for _ in 0..3 {
            let response = client.send_request(&IpcRequest::GetEventsTail { count: 5 });
            assert!(
                matches!(response, Ok(IpcResponse::Events { .. })),
                "{response:?}"
            );
        }
    }
}
