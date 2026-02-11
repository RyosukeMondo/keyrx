//! Settings RPC method handlers.
//!
//! This module implements all settings-related RPC methods for WebSocket communication.
//! Each method accepts parameters as serde_json::Value, validates them, and delegates
//! to the SettingsService for business logic execution.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::SettingsService;
use crate::web::rpc_types::{RpcError, INTERNAL_ERROR, INVALID_PARAMS};

/// Parameters for get_global_layout query
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GetGlobalLayoutParams {
    // No parameters needed
}

/// Parameters for set_global_layout command
#[derive(Debug, Deserialize)]
struct SetGlobalLayoutParams {
    layout: Option<String>,
}

/// Global layout information returned by get_global_layout
#[derive(Debug, Serialize)]
struct GlobalLayoutRpcInfo {
    layout: Option<String>,
}

/// Get global layout setting
pub async fn get_global_layout(
    settings_service: &SettingsService,
    _params: Value,
) -> Result<Value, RpcError> {
    let layout = settings_service
        .get_global_layout()
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e))?;

    let info = GlobalLayoutRpcInfo { layout };

    serde_json::to_value(&info).map_err(|e| RpcError::new(INTERNAL_ERROR, e.to_string()))
}

/// Set global layout setting
pub async fn set_global_layout(
    settings_service: &SettingsService,
    params: Value,
) -> Result<Value, RpcError> {
    let params: SetGlobalLayoutParams = serde_json::from_value(params)
        .map_err(|e| RpcError::new(INVALID_PARAMS, format!("Invalid parameters: {}", e)))?;

    settings_service
        .set_global_layout(params.layout)
        .await
        .map_err(|e| RpcError::new(INTERNAL_ERROR, e))?;

    Ok(serde_json::json!({ "success": true }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_get_global_layout_params() {
        let params = json!({});
        let result: Result<GetGlobalLayoutParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_set_global_layout_params_with_layout() {
        let params = json!({
            "layout": "ANSI_104"
        });
        let result: Result<SetGlobalLayoutParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().layout, Some("ANSI_104".to_string()));
    }

    #[test]
    fn test_deserialize_set_global_layout_params_with_null() {
        let params = json!({
            "layout": null
        });
        let result: Result<SetGlobalLayoutParams, _> = serde_json::from_value(params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().layout, None);
    }

    #[test]
    fn test_global_layout_rpc_info_serialization() {
        let info = GlobalLayoutRpcInfo {
            layout: Some("ISO_105".to_string()),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["layout"], "ISO_105");
    }

    #[test]
    fn test_global_layout_rpc_info_serialization_none() {
        let info = GlobalLayoutRpcInfo { layout: None };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["layout"], json!(null));
    }
}
