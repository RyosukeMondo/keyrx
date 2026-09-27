//! Where the daemon's IPC server listens and the CLI connects.
//!
//! [`IpcEndpoint`] owns the one platform difference of the transport:
//! - Unix: a socket file. Stale files are removed before binding, the file is
//!   restricted to its owner (0600) and removed on shutdown.
//! - Windows: a named pipe (`\\.\pipe\<name>`). There is no file, so none of
//!   the file steps apply. The pipe keeps the default named-pipe DACL, which
//!   gives other users read-only access: they cannot send requests. Do not
//!   widen it.

use super::IpcError;
use std::fmt;
use std::path::PathBuf;

/// Production socket file on Unix.
const UNIX_DEFAULT_SOCKET: &str = "/tmp/keyrx-daemon.sock";
/// Production pipe name on Windows (`\\.\pipe\keyrx-daemon`).
const WINDOWS_DEFAULT_PIPE: &str = "keyrx-daemon";
/// Prefix Windows puts in front of every local pipe name.
const PIPE_PREFIX: &str = r"\\.\pipe\";

/// An IPC endpoint: a Unix socket file or a Windows named pipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcEndpoint {
    /// A Unix domain socket file.
    SocketFile(PathBuf),
    /// A named pipe, stored without the `\\.\pipe\` prefix.
    NamedPipe(String),
}

impl IpcEndpoint {
    /// The endpoint the production daemon serves and the CLI uses by default.
    pub fn default_for_platform() -> Self {
        if cfg!(windows) {
            Self::NamedPipe(WINDOWS_DEFAULT_PIPE.to_string())
        } else {
            Self::SocketFile(PathBuf::from(UNIX_DEFAULT_SOCKET))
        }
    }

    /// The endpoint a test-mode daemon with process id `pid` serves.
    pub fn test_for_process(pid: u32) -> Self {
        if cfg!(windows) {
            Self::NamedPipe(format!("keyrx-test-{pid}"))
        } else {
            Self::SocketFile(PathBuf::from(format!("/tmp/keyrx-test-{pid}.sock")))
        }
    }

    /// Parses a user-supplied endpoint (the CLI `--socket` flag).
    ///
    /// On Windows it is a pipe name, with or without the `\\.\pipe\` prefix
    /// or the `@` namespace marker; elsewhere it is a socket file path.
    pub fn parse(value: &str) -> Self {
        if cfg!(windows) {
            let name = value.strip_prefix(PIPE_PREFIX).unwrap_or(value);
            Self::NamedPipe(name.strip_prefix('@').unwrap_or(name).to_string())
        } else {
            Self::SocketFile(PathBuf::from(value))
        }
    }

    /// `--socket` if given, else [`Self::default_for_platform`].
    pub fn from_cli(socket: Option<&str>) -> Self {
        socket.map_or_else(Self::default_for_platform, Self::parse)
    }

    /// The name to hand to `interprocess` (`@` marks a namespaced pipe name).
    pub(crate) fn local_socket_name(&self) -> String {
        match self {
            Self::SocketFile(path) => path.to_string_lossy().into_owned(),
            Self::NamedPipe(name) => format!("@{name}"),
        }
    }

    /// Clears a stale socket file before binding. No-op for pipes: a pipe
    /// someone still owns makes the bind fail instead.
    pub(crate) fn prepare_bind(&self) -> std::io::Result<()> {
        match self {
            Self::SocketFile(path) if path.exists() => std::fs::remove_file(path),
            _ => Ok(()),
        }
    }

    /// Restricts a freshly bound socket file to its owner. No-op for pipes
    /// (default DACL, see the module docs).
    pub(crate) fn restrict_access(&self) -> std::io::Result<()> {
        #[cfg(unix)]
        if let Self::SocketFile(path) = self {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    /// Removes the socket file so the CLI does not find a stale one. No-op
    /// for pipes, which vanish with their last handle.
    pub fn remove(&self) {
        let Self::SocketFile(path) = self else {
            return;
        };
        match std::fs::remove_file(path) {
            Ok(()) => log::info!("Removed IPC socket {self}"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => log::warn!("Failed to remove IPC socket {self}: {e}"),
        }
    }

    /// Fails fast with [`IpcError::SocketNotFound`] when a socket file is
    /// missing. Pipes cannot be probed; their absence shows up on connect.
    pub(crate) fn check_present(&self) -> Result<(), IpcError> {
        match self {
            Self::SocketFile(path) if !path.exists() => {
                Err(IpcError::SocketNotFound(self.to_string()))
            }
            _ => Ok(()),
        }
    }

    /// Maps a connect error to "daemon not running" where that is what it
    /// means: an absent endpoint, or (Unix) a socket file nobody listens on.
    pub(crate) fn connect_error(&self, error: std::io::Error) -> IpcError {
        match (self, error.kind()) {
            (_, std::io::ErrorKind::NotFound) => IpcError::SocketNotFound(self.to_string()),
            (Self::SocketFile(_), std::io::ErrorKind::ConnectionRefused) => {
                IpcError::StaleSocket(self.to_string())
            }
            _ => IpcError::IoError(error),
        }
    }
}

impl fmt::Display for IpcEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SocketFile(path) => write!(f, "{}", path.display()),
            Self::NamedPipe(name) => write!(f, "{PIPE_PREFIX}{name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn default_matches_platform() {
        let endpoint = IpcEndpoint::default_for_platform();
        if cfg!(windows) {
            assert_eq!(endpoint.to_string(), r"\\.\pipe\keyrx-daemon");
            assert_eq!(endpoint.local_socket_name(), "@keyrx-daemon");
        } else {
            assert_eq!(endpoint.to_string(), "/tmp/keyrx-daemon.sock");
            assert_eq!(endpoint.local_socket_name(), "/tmp/keyrx-daemon.sock");
        }
    }

    #[test]
    fn test_endpoint_is_per_process() {
        assert_ne!(
            IpcEndpoint::test_for_process(1),
            IpcEndpoint::test_for_process(2)
        );
        assert!(IpcEndpoint::test_for_process(42)
            .to_string()
            .contains("keyrx-test-42"));
    }

    #[test]
    #[cfg(windows)]
    fn parse_accepts_every_pipe_spelling() {
        let expected = IpcEndpoint::NamedPipe("keyrx-daemon".to_string());
        assert_eq!(IpcEndpoint::parse("keyrx-daemon"), expected);
        assert_eq!(IpcEndpoint::parse("@keyrx-daemon"), expected);
        assert_eq!(IpcEndpoint::parse(r"\\.\pipe\keyrx-daemon"), expected);
    }

    #[test]
    #[cfg(unix)]
    fn parse_is_a_path_on_unix() {
        assert_eq!(
            IpcEndpoint::parse("/run/x.sock"),
            IpcEndpoint::SocketFile(PathBuf::from("/run/x.sock"))
        );
    }

    #[test]
    fn from_cli_falls_back_to_default() {
        assert_eq!(
            IpcEndpoint::from_cli(None),
            IpcEndpoint::default_for_platform()
        );
    }

    #[test]
    fn missing_endpoint_means_not_running() {
        let pipe = IpcEndpoint::NamedPipe("x".to_string());
        let err = pipe.connect_error(Error::from(ErrorKind::NotFound));
        assert!(matches!(err, IpcError::SocketNotFound(_)), "{err:?}");
        assert_eq!(err.code(), 3005);
    }

    #[test]
    fn refused_is_stale_only_for_socket_files() {
        let file = IpcEndpoint::SocketFile(PathBuf::from("/tmp/x.sock"));
        let refused = || Error::from(ErrorKind::ConnectionRefused);
        assert!(matches!(
            file.connect_error(refused()),
            IpcError::StaleSocket(_)
        ));
        let pipe = IpcEndpoint::NamedPipe("x".to_string());
        assert!(matches!(
            pipe.connect_error(refused()),
            IpcError::IoError(_)
        ));
    }

    #[test]
    fn pipes_skip_file_steps() {
        let pipe = IpcEndpoint::NamedPipe("x".to_string());
        assert!(pipe.check_present().is_ok());
        assert!(pipe.prepare_bind().is_ok());
        assert!(pipe.restrict_access().is_ok());
        pipe.remove();
    }

    #[test]
    fn missing_socket_file_fails_fast() {
        let dir = tempfile::tempdir().unwrap();
        let file = IpcEndpoint::SocketFile(dir.path().join("absent.sock"));
        assert!(matches!(
            file.check_present(),
            Err(IpcError::SocketNotFound(_))
        ));
        file.remove(); // absent file: no panic, no warning
    }
}
