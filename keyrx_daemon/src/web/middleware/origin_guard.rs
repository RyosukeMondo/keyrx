//! Only the local UI (and tools without a browser) may talk to the daemon.
//!
//! CORS does not protect this server: browsers apply it to reading fetch
//! responses, not to WebSocket handshakes or to "simple" requests (a form
//! POST with `text/plain` is sent without a preflight). Without this guard
//! any web page the user opened could stream every keystroke from `/ws`,
//! drive `/ws-rpc`, or switch profiles with a cross-site POST.
//!
//! Rules, applied to every request:
//! - An `Origin` header, when present, must be a loopback origin (the
//!   embedded UI, `vite dev`) or one of the configured CORS origins.
//!   Requests without `Origin` (CLI, curl, native clients) pass.
//! - While the server is bound to loopback, `Host` must be a loopback name:
//!   this stops DNS rebinding (an attacker's domain resolving to 127.0.0.1
//!   makes their page "same-origin", so it sends no `Origin` on GETs).

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::daemon_config::DaemonConfig;

/// Which origins and hosts may reach the server (see the module docs).
#[derive(Clone, Debug)]
pub struct OriginGuard {
    allowed_origins: Vec<String>,
    check_host: bool,
}

impl OriginGuard {
    /// Allowed origins from the config's CORS list; the `Host` check is on
    /// when the server binds to a loopback address.
    pub fn from_config(config: &DaemonConfig) -> Self {
        let loopback = config
            .socket_addr()
            .map(|addr| addr.ip().is_loopback())
            .unwrap_or(true);
        Self::new(config.cors_origins().to_vec(), loopback)
    }

    pub fn new(allowed_origins: Vec<String>, check_host: bool) -> Self {
        Self {
            allowed_origins,
            check_host,
        }
    }

    /// `Err` with the reason when the request must be refused.
    pub fn check(&self, origin: Option<&str>, host: Option<&str>) -> Result<(), &'static str> {
        if let Some(origin) = origin {
            let configured = self.allowed_origins.iter().any(|o| o == origin);
            if !configured && !origin_is_loopback(origin) {
                return Err("cross-origin request refused");
            }
        }
        if self.check_host && !host.is_some_and(host_is_loopback) {
            return Err("request for a non-local host name refused");
        }
        Ok(())
    }
}

/// `http(s)://<loopback host>[:port]`.
fn origin_is_loopback(origin: &str) -> bool {
    origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .is_some_and(host_is_loopback)
}

/// A `Host`-style authority (`name[:port]`) naming this machine.
fn host_is_loopback(authority: &str) -> bool {
    let name = if let Some(rest) = authority.strip_prefix('[') {
        rest.split(']').next().unwrap_or_default()
    } else {
        authority.split(':').next().unwrap_or_default()
    };
    name.eq_ignore_ascii_case("localhost")
        || name
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

/// Refuses requests that fail [`OriginGuard::check`] with 403.
pub async fn origin_guard_middleware(
    State(guard): State<OriginGuard>,
    request: Request,
    next: Next,
) -> Response {
    // Owned copies, read in a block: a borrow of the request must not live
    // across the await below (the future has to be Send).
    let (origin, host) = {
        let header_str = |name| {
            request
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        (header_str(header::ORIGIN), header_str(header::HOST))
    };
    match guard.check(origin.as_deref(), host.as_deref()) {
        Ok(()) => next.run(request).await,
        Err(reason) => {
            log::warn!(
                "Refused {} {} (origin {:?}, host {:?}): {reason}",
                request.method(),
                request.uri().path(),
                origin,
                host
            );
            (StatusCode::FORBIDDEN, reason).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> OriginGuard {
        OriginGuard::new(vec!["http://my-ui.example".into()], true)
    }

    #[test]
    fn local_ui_and_tools_pass() {
        let g = guard();
        assert!(g.check(None, Some("127.0.0.1:9867")).is_ok());
        assert!(g
            .check(Some("http://127.0.0.1:9867"), Some("127.0.0.1:9867"))
            .is_ok());
        assert!(g
            .check(Some("http://localhost:5173"), Some("localhost:9867"))
            .is_ok());
        assert!(g
            .check(Some("http://[::1]:9867"), Some("[::1]:9867"))
            .is_ok());
        assert!(g
            .check(Some("http://my-ui.example"), Some("127.0.0.1:9867"))
            .is_ok());
    }

    #[test]
    fn foreign_origins_are_refused() {
        let g = guard();
        for origin in [
            "http://evil.example",
            "null",
            "http://127.0.0.1.evil.example",
            "http://localhost.evil.example:9867",
        ] {
            assert!(
                g.check(Some(origin), Some("127.0.0.1:9867")).is_err(),
                "{origin}"
            );
        }
    }

    #[test]
    fn dns_rebinding_host_is_refused_on_loopback_only() {
        assert!(guard().check(None, Some("evil.example:9867")).is_err());
        assert!(guard().check(None, None).is_err());
        let lan = OriginGuard::new(Vec::new(), false);
        assert!(lan.check(None, Some("192.168.1.5:9867")).is_ok());
    }
}
