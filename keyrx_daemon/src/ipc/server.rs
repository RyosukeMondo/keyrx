//! IPC server: an [`IpcEndpoint`] that answers [`IpcRequest`]s via
//! [`IpcCommandHandler`](super::commands::IpcCommandHandler).

use super::{IpcEndpoint, IpcRequest, IpcResponse};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Longest request line accepted. Requests are one small JSON object; a
/// client that sends more without a newline is broken or hostile and must not
/// be buffered without bound.
pub const MAX_REQUEST_BYTES: u64 = 1024 * 1024;

/// Simultaneous client connections served (one thread each). Further
/// connections are closed at once instead of spawning threads without limit.
pub const MAX_CLIENTS: usize = 256;

/// Error code for a request the server could not parse.
const BAD_REQUEST_CODE: u16 = 4000;

/// Releases a client slot when its thread ends, however it ends.
struct ClientSlot(Arc<AtomicUsize>);

impl ClientSlot {
    fn acquire(active: &Arc<AtomicUsize>) -> Option<Self> {
        if active.fetch_add(1, Ordering::AcqRel) >= MAX_CLIENTS {
            active.fetch_sub(1, Ordering::AcqRel);
            return None;
        }
        Some(Self(Arc::clone(active)))
    }
}

impl Drop for ClientSlot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

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

        let active = Arc::new(AtomicUsize::new(0));
        loop {
            match listener.accept() {
                Ok(stream) => {
                    let Some(slot) = ClientSlot::acquire(&active) else {
                        log::warn!("IPC: {MAX_CLIENTS} clients already connected; refusing one");
                        continue; // dropping the stream closes it
                    };
                    let handler = Arc::clone(&handler);
                    std::thread::spawn(move || {
                        let _slot = slot;
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
            let request = match read_request(&mut reader)? {
                Incoming::Closed => return Ok(()),
                Incoming::Request(request) => request,
                Incoming::Bad(message) => {
                    log::warn!("IPC: rejected a request: {message}");
                    let response = IpcResponse::Error {
                        code: BAD_REQUEST_CODE,
                        message,
                    };
                    write_response(reader.get_mut(), &response)?;
                    continue;
                }
                Incoming::TooLong => {
                    let response = IpcResponse::Error {
                        code: BAD_REQUEST_CODE,
                        message: format!("request longer than {MAX_REQUEST_BYTES} bytes"),
                    };
                    // Cannot resynchronise mid-line: answer, then hang up.
                    let _ = write_response(reader.get_mut(), &response);
                    return Ok(());
                }
            };
            log::debug!("Received IPC request: {:?}", request);

            let response = {
                let handler_guard = handler.blocking_lock();
                handler_guard(request).unwrap_or_else(|message| IpcResponse::Error {
                    code: 5000,
                    message,
                })
            };

            write_response(reader.get_mut(), &response)?;
            log::debug!("Sent IPC response");
        }
    }

    /// The endpoint this server serves
    pub fn endpoint(&self) -> &IpcEndpoint {
        &self.endpoint
    }
}

/// One line read from a client.
enum Incoming {
    Closed,
    Request(IpcRequest),
    /// A complete line that is not a request (message says why).
    Bad(String),
    /// More than [`MAX_REQUEST_BYTES`] without a newline.
    TooLong,
}

/// Reads one newline-delimited request without buffering more than
/// [`MAX_REQUEST_BYTES`].
fn read_request<R: BufRead>(reader: &mut R) -> std::io::Result<Incoming> {
    let mut line = Vec::new();
    let read = reader
        .by_ref()
        .take(MAX_REQUEST_BYTES + 1)
        .read_until(b'\n', &mut line)?;
    if read == 0 {
        return Ok(Incoming::Closed);
    }
    if line.last() != Some(&b'\n') {
        // No newline: either the limit was hit or the client hung up mid-line.
        return Ok(if read as u64 > MAX_REQUEST_BYTES {
            Incoming::TooLong
        } else {
            Incoming::Closed
        });
    }
    let text = match std::str::from_utf8(&line) {
        Ok(text) => text.trim(),
        Err(e) => return Ok(Incoming::Bad(format!("request is not valid UTF-8: {e}"))),
    };
    Ok(match serde_json::from_str(text) {
        Ok(request) => Incoming::Request(request),
        Err(e) => Incoming::Bad(format!("invalid request: {e}")),
    })
}

fn write_response<W: Write>(stream: &mut W, response: &IpcResponse) -> std::io::Result<()> {
    let json = serde_json::to_string(response).map_err(std::io::Error::other)?;
    stream.write_all(json.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()
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

    #[cfg(unix)]
    fn raw(endpoint: &IpcEndpoint) -> std::os::unix::net::UnixStream {
        let IpcEndpoint::SocketFile(path) = endpoint else {
            unreachable!()
        };
        let stream = std::os::unix::net::UnixStream::connect(path).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .unwrap();
        stream
    }

    #[cfg(unix)]
    fn read_reply(stream: &mut std::os::unix::net::UnixStream) -> Option<String> {
        let mut line = String::new();
        let mut reader = BufReader::new(stream);
        (reader.read_line(&mut line).ok()? > 0).then_some(line)
    }

    /// A garbage line is answered with an error and the connection stays
    /// usable (it used to be dropped silently, killing the client's session).
    #[test]
    #[cfg(unix)]
    fn test_garbage_line_gets_an_error_and_the_connection_survives() {
        let dir = tempfile::tempdir().unwrap();
        let endpoint = test_endpoint(dir.path(), "garbage");
        spawn_handler(dir.path(), &endpoint);
        let mut stream = raw(&endpoint);

        stream.write_all(b"\xff\xfe not json\n").unwrap();
        let reply = read_reply(&mut stream).expect("an error reply");
        assert!(reply.contains("\"code\":4000"), "{reply}");
        stream.write_all(b"{\"type\":\"nonsense\"}\n").unwrap();
        assert!(read_reply(&mut stream).unwrap().contains("4000"));

        let request = serde_json::to_string(&IpcRequest::GetStatus).unwrap();
        stream.write_all(format!("{request}\n").as_bytes()).unwrap();
        let reply = read_reply(&mut stream).unwrap();
        assert!(
            reply.contains("Status") || reply.contains("running"),
            "{reply}"
        );
    }

    /// A line without a newline is cut off at MAX_REQUEST_BYTES instead of
    /// being buffered whole; the client gets an error and is disconnected.
    #[test]
    #[cfg(unix)]
    fn test_unterminated_oversized_request_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let endpoint = test_endpoint(dir.path(), "oversize");
        spawn_handler(dir.path(), &endpoint);
        let mut stream = raw(&endpoint);

        let chunk = vec![b'a'; 64 * 1024];
        let mut sent = 0u64;
        while sent < MAX_REQUEST_BYTES + 64 * 1024 {
            if stream.write_all(&chunk).is_err() {
                break; // the server already hung up
            }
            sent += chunk.len() as u64;
        }
        let reply = read_reply(&mut stream).expect("an error reply");
        assert!(reply.contains("longer than"), "{reply}");
        assert!(
            read_reply(&mut stream).is_none(),
            "connection must be closed"
        );
    }

    /// Past MAX_CLIENTS simultaneous connections the server closes new ones
    /// instead of spawning a thread each; slots free up when clients leave.
    #[test]
    #[cfg(unix)]
    fn test_client_count_is_capped_and_slots_are_released() {
        let dir = tempfile::tempdir().unwrap();
        let endpoint = test_endpoint(dir.path(), "cap");
        spawn_handler(dir.path(), &endpoint);

        let mut held: Vec<_> = (0..MAX_CLIENTS).map(|_| raw(&endpoint)).collect();
        // Let the accept loop register all of them.
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mut extra = raw(&endpoint);
        assert!(
            read_reply(&mut extra).is_none(),
            "the extra client is closed"
        );

        held.truncate(MAX_CLIENTS - 5);
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mut client = IpcClient::new(endpoint);
        let status = client.send_request(&IpcRequest::GetStatus);
        assert!(
            matches!(status, Ok(IpcResponse::Status { .. })),
            "{status:?}"
        );
    }

    #[test]
    fn test_no_server_is_socket_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let mut client = IpcClient::new(test_endpoint(dir.path(), "absent"));
        let err = client.send_request(&IpcRequest::GetStatus).unwrap_err();
        assert!(matches!(err, IpcError::SocketNotFound(_)), "{err:?}");
    }
}
