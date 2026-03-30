//! MCP (Model Context Protocol) endpoint for AI agent integration.
//!
//! Exposes KeyRx daemon capabilities as MCP tools via Streamable HTTP transport.
//! AI agents (Claude Code, Cursor, etc.) connect to `/mcp` to discover and invoke tools
//! for profile management, simulation, diagnostics, and more.

pub mod tool_handlers;
pub mod tools;

use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    ServerHandler,
};

use crate::web::AppState;
use tools::*;

/// MCP server backed by KeyRx daemon services.
///
/// Each instance holds a reference to the shared `AppState`, so all MCP sessions
/// share the same service layer as REST and WebSocket RPC endpoints.
#[derive(Clone)]
pub struct KeyrxMcpServer {
    state: Arc<AppState>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl std::fmt::Debug for KeyrxMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyrxMcpServer").finish_non_exhaustive()
    }
}

impl KeyrxMcpServer {
    fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl KeyrxMcpServer {
    #[tool(description = "List all keyboard remapping profiles")]
    async fn keyrx_list_profiles(
        &self,
        Parameters(_input): Parameters<ListProfilesInput>,
    ) -> Result<String, String> {
        tool_handlers::list_profiles(&self.state).await
    }

    #[tool(description = "Get the Rhai DSL source code of a profile configuration")]
    async fn keyrx_get_profile_config(
        &self,
        Parameters(input): Parameters<GetProfileConfigInput>,
    ) -> Result<String, String> {
        tool_handlers::get_profile_config(&self.state, &input.name).await
    }

    #[tool(description = "Set/update the Rhai DSL source code of a profile configuration")]
    async fn keyrx_set_profile_config(
        &self,
        Parameters(input): Parameters<SetProfileConfigInput>,
    ) -> Result<String, String> {
        tool_handlers::set_profile_config(&self.state, &input.name, &input.source).await
    }

    #[tool(description = "Activate a profile (compile and hot-reload)")]
    async fn keyrx_activate_profile(
        &self,
        Parameters(input): Parameters<ActivateProfileInput>,
    ) -> Result<String, String> {
        tool_handlers::activate_profile(&self.state, &input.name).await
    }

    #[tool(description = "Create a new profile from a template")]
    async fn keyrx_create_profile(
        &self,
        Parameters(input): Parameters<CreateProfileInput>,
    ) -> Result<String, String> {
        tool_handlers::create_profile(&self.state, &input.name, input.template.as_deref()).await
    }

    #[tool(description = "Validate a profile's Rhai DSL configuration by compiling it")]
    async fn keyrx_validate_profile(
        &self,
        Parameters(input): Parameters<ValidateProfileInput>,
    ) -> Result<String, String> {
        tool_handlers::validate_profile(&self.state, &input.name).await
    }

    #[tool(description = "Simulate key events through the remapping engine")]
    async fn keyrx_simulate(
        &self,
        Parameters(input): Parameters<SimulateInput>,
    ) -> Result<String, String> {
        tool_handlers::simulate(&self.state, input.scenario.as_deref())
    }

    #[tool(description = "Get daemon status (running, version, active profile, devices)")]
    async fn keyrx_get_status(
        &self,
        Parameters(_input): Parameters<GetStatusInput>,
    ) -> Result<String, String> {
        tool_handlers::get_status(&self.state)
    }

    #[tool(description = "Get daemon runtime state (modifiers, locks, layers)")]
    async fn keyrx_get_state(
        &self,
        Parameters(_input): Parameters<GetStateInput>,
    ) -> Result<String, String> {
        tool_handlers::get_state(&self.state)
    }

    #[tool(description = "List connected keyboard/input devices")]
    async fn keyrx_list_devices(
        &self,
        Parameters(_input): Parameters<ListDevicesInput>,
    ) -> Result<String, String> {
        tool_handlers::list_devices(&self.state).await
    }

    #[tool(description = "Get system diagnostics (version, build, platform)")]
    async fn keyrx_get_diagnostics(
        &self,
        Parameters(_input): Parameters<GetDiagnosticsInput>,
    ) -> Result<String, String> {
        tool_handlers::get_diagnostics()
    }

    #[tool(description = "Get key processing latency statistics (min, avg, max, p95, p99)")]
    async fn keyrx_get_latency(
        &self,
        Parameters(_input): Parameters<GetLatencyInput>,
    ) -> Result<String, String> {
        tool_handlers::get_latency(&self.state)
    }
}

#[tool_handler]
impl ServerHandler for KeyrxMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "KeyRx keyboard remapping daemon. \
                 Manage profiles, simulate key events, and query diagnostics.",
        )
    }
}

/// Create the MCP Streamable HTTP service for mounting in the axum router.
///
/// The service creates a new `KeyrxMcpServer` per session, each sharing
/// the same `Arc<AppState>` for access to all daemon services.
pub fn create_service(
    state: Arc<AppState>,
) -> StreamableHttpService<KeyrxMcpServer, LocalSessionManager> {
    let config = StreamableHttpServerConfig {
        stateful_mode: false,
        json_response: true,
        ..Default::default()
    };

    StreamableHttpService::new(
        move || Ok(KeyrxMcpServer::new(Arc::clone(&state))),
        Arc::new(LocalSessionManager::default()),
        config,
    )
}
