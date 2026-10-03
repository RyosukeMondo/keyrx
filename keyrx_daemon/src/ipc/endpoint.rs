//! Where the daemon's IPC server listens and the CLI connects.
//!
//! [`IpcEndpoint`] owns the one platform difference of the transport:
//! - Unix: a per-user socket file (`$XDG_RUNTIME_DIR/keyrx-daemon.sock`, or
//!   `/tmp/keyrx-daemon-<uid>.sock` without a runtime dir). Stale files are
//!   removed before binding - a file a live daemon still answers on is not -
//!   the file is restricted to its owner (0600) and removed on shutdown.
//! - Windows: a named pipe (`\\.\pipe\<name>`). There is no file, so none of
//!   the file steps apply. The pipe keeps the default named-pipe DACL, which
//!   gives other users read-only access: they cannot send requests. Do not
//!   widen it.

use super::IpcError;
use std::fmt;
use std::path::PathBuf;

/// Production socket file name on Unix (inside the user's runtime dir).
const UNIX_SOCKET_NAME: &str = "keyrx-daemon.sock";
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
            let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR");
            Self::SocketFile(unix_default_socket(runtime_dir.as_deref(), current_uid()))
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

    /// True when a daemon is accepting connections on this endpoint right now
    /// (a socket file nobody listens on is stale, not live). Pipes cannot be
    /// probed without side effects; they report false and rely on the bind.
    pub(crate) fn probe_live(&self) -> bool {
        #[cfg(unix)]
        if let Self::SocketFile(path) = self {
            return std::os::unix::net::UnixStream::connect(path).is_ok();
        }
        false
    }

    /// Clears a stale socket file before binding, but refuses to take over
    /// one a running daemon still answers on (a second daemon would orphan
    /// it). No-op for pipes: a pipe someone still owns makes the bind fail.
    pub(crate) fn prepare_bind(&self) -> std::io::Result<()> {
        let Self::SocketFile(path) = self else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }
        if self.probe_live() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!(
                    "another keyrx daemon is already listening on {}; stop it first \
                     (keyrx_daemon status shows it)",
                    path.display()
                ),
            ));
        }
        std::fs::remove_file(path)
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

/// The per-user production socket: inside `runtime_dir` (systemd's
/// `$XDG_RUNTIME_DIR`, already private to the user) when there is one, else
/// a uid-qualified file in /tmp so two users never share a socket.
fn unix_default_socket(runtime_dir: Option<&std::ffi::OsStr>, uid: u32) -> PathBuf {
    match runtime_dir.filter(|dir| !dir.is_empty()) {
        Some(dir) => PathBuf::from(dir).join(UNIX_SOCKET_NAME),
        None => PathBuf::from(format!("/tmp/keyrx-daemon-{uid}.sock")),
    }
}

#[cfg(target_os = "linux")]
fn current_uid() -> u32 {
    nix::unistd::getuid().as_raw()
}

#[cfg(not(target_os = "linux"))]
fn current_uid() -> u32 {
    0
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
            assert!(endpoint.to_string().ends_with(".sock"), "{endpoint}");
            assert_eq!(endpoint.local_socket_name(), endpoint.to_string());
        }
    }

    #[test]
    fn unix_socket_is_per_user() {
        use std::ffi::OsStr;
        assert_eq!(
            unix_default_socket(Some(OsStr::new("/run/user/1000")), 1000),
            PathBuf::from("/run/user/1000/keyrx-daemon.sock")
        );
        assert_eq!(
            unix_default_socket(None, 1001),
            PathBuf::from("/tmp/keyrx-daemon-1001.sock")
        );
        assert_eq!(
            unix_default_socket(Some(OsStr::new("")), 7),
            PathBuf::from("/tmp/keyrx-daemon-7.sock")
        );
    }

    #[test]
    #[cfg(unix)]
    fn bind_refuses_a_socket_a_daemon_still_answers_on() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("live.sock");
        let _listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        let endpoint = IpcEndpoint::SocketFile(path.clone());
        let err = endpoint.prepare_bind().unwrap_err();
        assert_eq!(err.kind(), ErrorKind::AddrInUse);
        assert!(path.exists(), "a live daemon's socket must not be removed");
    }

    #[test]
    #[cfg(unix)]
    fn probe_tells_a_live_daemon_from_a_stale_socket() {
        let dir = tempfile::tempdir().unwrap();
        let live = dir.path().join("live.sock");
        let _listener = std::os::unix::net::UnixListener::bind(&live).unwrap();
        assert!(IpcEndpoint::SocketFile(live).probe_live());

        let stale = dir.path().join("stale.sock");
        drop(std::os::unix::net::UnixListener::bind(&stale).unwrap());
        assert!(!IpcEndpoint::SocketFile(stale).probe_live());
        assert!(!IpcEndpoint::SocketFile(dir.path().join("none.sock")).probe_live());
    }

    #[test]
    #[cfg(unix)]
    fn bind_clears_a_stale_socket() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stale.sock");
        drop(std::os::unix::net::UnixListener::bind(&path).unwrap());
        IpcEndpoint::SocketFile(path.clone())
            .prepare_bind()
            .unwrap();
        assert!(!path.exists());
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
