//! Request and response DTOs for the profile management endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProfileResponse {
    pub(super) name: String,
    #[serde(rename = "rhaiPath")]
    pub(super) rhai_path: String,
    #[serde(rename = "krxPath")]
    pub(super) krx_path: String,
    #[serde(
        rename = "createdAt",
        serialize_with = "serialize_systemtime_as_rfc3339"
    )]
    pub(super) created_at: std::time::SystemTime,
    #[serde(
        rename = "modifiedAt",
        serialize_with = "serialize_systemtime_as_rfc3339"
    )]
    pub(super) modified_at: std::time::SystemTime,
    #[serde(rename = "layerCount")]
    pub(super) layer_count: usize,
    #[serde(rename = "deviceCount")]
    pub(super) device_count: usize,
    #[serde(rename = "keyCount")]
    pub(super) key_count: usize,
    #[serde(rename = "isActive")]
    pub(super) active: bool,
    #[serde(
        rename = "activatedAt",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_systemtime_as_rfc3339"
    )]
    pub(super) activated_at: Option<std::time::SystemTime>,
    #[serde(rename = "activatedBy", skip_serializing_if = "Option::is_none")]
    pub(super) activated_by: Option<String>,
}

/// Serialize SystemTime as RFC 3339 / ISO 8601 string
fn serialize_systemtime_as_rfc3339<S>(
    time: &std::time::SystemTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let datetime: DateTime<Utc> = (*time).into();
    datetime.to_rfc3339().serialize(serializer)
}

/// Serialize Option<SystemTime> as RFC 3339 / ISO 8601 string
fn serialize_optional_systemtime_as_rfc3339<S>(
    time: &Option<std::time::SystemTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    match time {
        Some(t) => {
            let datetime: DateTime<Utc> = (*t).into();
            datetime.to_rfc3339().serialize(serializer)
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Serialize)]
pub(super) struct ProfilesListResponse {
    pub(super) profiles: Vec<ProfileResponse>,
}

/// POST /api/profiles - Create new profile
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateProfileRequest {
    pub(super) name: String,
    pub(super) template: String, // "blank", "simple_remap", "capslock_escape", "vim_navigation", "gaming"
}

/// POST /api/profiles/:name/duplicate - Duplicate profile
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DuplicateProfileRequest {
    #[serde(rename = "newName")]
    pub(super) new_name: String,
}

/// PUT /api/profiles/:name/rename - Rename profile
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RenameProfileRequest {
    #[serde(rename = "newName")]
    pub(super) new_name: String,
}

/// PUT /api/profiles/:name/config - Set profile configuration
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SetProfileConfigRequest {
    pub(super) config: String,
}

/// POST /api/profiles/:name/validate - Validate profile configuration
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ValidationError {
    /// 1-based line (1 when the compiler reported no position).
    pub(super) line: usize,
    /// 1-based column (1 when unknown).
    pub(super) column: usize,
    /// Characters to highlight from `column` (the editor marks at least one).
    pub(super) length: usize,
    pub(super) message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ValidationResponse {
    pub(super) valid: bool,
    pub(super) errors: Vec<ValidationError>,
}
