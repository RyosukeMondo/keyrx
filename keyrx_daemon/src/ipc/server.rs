//! IPC server: an [`IpcEndpoint`] that answers [`IpcRequest`]s via
//! [`IpcCommandHandler`](super::commands::IpcCommandHandler).

use super::{IpcEndpoint, IpcRequest, IpcResponse};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Binds an IPC server at `endpoint` and serves `handler` on a background
/// thread. The single way runners expose IPC (production and test mode).
///
/// Fails if the endpoint cannot be bound, e.g. on Windows when another
/// daemon already owns the pipe.
pub fn spawn(
    endpoint: IpcEndpoint,
    handler: Arc<super::commands::IpcCommandHandler>,
) -> Result<(), std::io::Error> {
    let mut server = IpcServer::new(endpoint);
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
    endpoint: IpcEndpoint,
    listener: Option<LocalSocketListener>,
}

impl IpcServer {
    /// Create a new IPC server for `endpoint` (not bound until [`Self::start`])
    pub fn new(endpoint: IpcEndpoint) -> Self {
        Self {
            endpoint,
            listener: None,
        }
    }

    /// Start the IPC server and bind to its endpoint
    pub fn start(&mut self) -> Result<(), std::io::Error> {
        self.endpoint.prepare_bind()?;
        let listener = LocalSocketListener::bind(self.endpoint.local_socket_name())?;
        self.endpoint.restrict_access()?;

        self.listener = Some(listener);
        log::info!("IPC server listening on {}", self.endpoint);
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

    /// The endpoint this server serves
    pub fn endpoint(&self) -> &IpcEndpoint {
        &self.endpoint
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Only a server that bound its endpoint may remove it
        if self.listener.is_some() {
            self.endpoint.remove();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::client::IpcClient;
    use crate::ipc::{DaemonIpc, IpcError};
    use crate::services::DaemonQueryService;

    /// A unique endpoint per test (socket file in `dir`, or a named pipe).
    fn test_endpoint(dir: &std::path::Path, tag: &str) -> IpcEndpoint {
        let name = format!("keyrx-server-test-{tag}-{}", std::process::id());
        if cfg!(windows) {
            IpcEndpoint::NamedPipe(name)
        } else {
            IpcEndpoint::SocketFile(dir.join(format!("{name}.sock")))
        }
    }

    fn spawn_handler(dir: &std::path::Path, endpoint: &IpcEndpoint) {
        let manager = Arc::new(crate::config::ProfileManager::new(dir.to_path_buf()).unwrap());
        let handler = Arc::new(super::super::commands::IpcCommandHandler::new(
            manager,
            Arc::new(DaemonQueryService::without_daemon()),
        ));
        spawn(endpoint.clone(), handler).unwrap();
    }

    #[test]
    fn test_server_creation() {
        let endpoint = IpcEndpoint::test_for_process(1);
        let server = IpcServer::new(endpoint.clone());
        assert_eq!(server.endpoint(), &endpoint);
    }

    /// The production path end to end: `spawn` -> `IpcClient` on the
    /// platform's transport (named pipe on Windows).
    #[test]
    fn test_spawned_server_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let endpoint = test_endpoint(dir.path(), "roundtrip");
        spawn_handler(dir.path(), &endpoint);

        let mut client = IpcClient::new(endpoint);
        let status = client.send_request(&IpcRequest::GetStatus);
        assert!(
            matches!(status, Ok(IpcResponse::Status { running: false, .. })),
            "{status:?}"
        );
        let events = client.send_request(&IpcRequest::GetEventsTail { count: 5 });
        assert!(
            matches!(events, Ok(IpcResponse::Events { .. })),
            "{events:?}"
        );
        let cleared = client.send_request(&IpcRequest::ClearEvents);
        assert!(
            matches!(cleared, Ok(IpcResponse::EventsCleared { .. })),
            "{cleared:?}"
        );
    }

    /// Regression: the server answered one request per connection, so a
    /// client reusing its connection (`metrics events --follow`) got EPIPE.
    #[test]
    fn test_several_requests_on_one_connection() {
        let dir = tempfile::tempdir().unwrap();
        let endpoint = test_endpoint(dir.path(), "reuse");
        spawn_handler(dir.path(), &endpoint);

        let mut client = IpcClient::new(endpoint);
        for _ in 0..3 {
            let response = client.send_request(&IpcRequest::GetEventsTail { count: 5 });
            assert!(
                matches!(response, Ok(IpcResponse::Events { .. })),
                "{response:?}"
            );
        }
    }

    /// A second daemon on the same pipe fails to bind (the runner logs it and
    /// runs without IPC) instead of taking the endpoint over. Unix replaces
    /// stale socket files, so this is a named-pipe property.
    #[test]
    #[cfg(windows)]
    fn test_second_server_on_same_pipe_fails() {
        let dir = tempfile::tempdir().unwrap();
        let endpoint = test_endpoint(dir.path(), "single");
        spawn_handler(dir.path(), &endpoint);

        let mut second = IpcServer::new(endpoint);
        assert!(second.start().is_err());
    }

    #[test]
    fn test_no_server_is_socket_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let mut client = IpcClient::new(test_endpoint(dir.path(), "absent"));
        let err = client.send_request(&IpcRequest::GetStatus).unwrap_err();
        assert!(matches!(err, IpcError::SocketNotFound(_)), "{err:?}");
    }
}
