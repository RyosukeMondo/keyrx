//! Import a layout file (`.krx` or `.rhai`) as a new profile.
//!
//! The ONE import path: REST, WS-RPC and the CLI all call
//! [`ProfileService::import_layout`], so a file is validated, converted and
//! stored identically wherever it comes from.
//!
//! - `.rhai` is the profile's source and is stored as-is (after it compiles).
//! - `.krx` carries no source. It is validated with the shared `.krx`
//!   validator, decompiled to Rhai ([`keyrx_core::parser::decompile`]), and
//!   that Rhai is compiled again and compared with the original mappings:
//!   a layout that cannot be reproduced exactly is refused, never imported
//!   approximately. What is lost is only what a `.krx` never stored
//!   (comments, formatting) plus its compile timestamp.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::{ProfileError, ProfileManager};
use crate::services::profile_service::ProfileInfo;
use crate::services::ProfileService;

/// Largest layout file accepted, in bytes. A real `.krx` is a few hundred
/// bytes to a few KiB; this bounds memory for hostile uploads.
pub const MAX_LAYOUT_BYTES: usize = 1024 * 1024;

/// The kind of layout file being imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutFormat {
    /// Compiled binary configuration.
    Krx,
    /// Rhai source.
    Rhai,
}

impl LayoutFormat {
    /// The format a file name's extension denotes (case-insensitive).
    ///
    /// # Errors
    ///
    /// `InvalidLayout` for any other extension.
    pub fn from_path(path: &Path) -> Result<Self, ProfileError> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("krx") => Ok(Self::Krx),
            Some("rhai") => Ok(Self::Rhai),
            _ => Err(ProfileError::InvalidLayout(format!(
                "{}: expected a .krx or .rhai file",
                path.display()
            ))),
        }
    }
}

/// Result of a successful import.
#[derive(Debug, Clone)]
pub struct ImportedLayout {
    /// The new profile.
    pub profile: ProfileInfo,
    /// Which kind of file was imported.
    pub format: LayoutFormat,
    /// True when Rhai had to be generated (a `.krx` import).
    pub converted: bool,
    /// Lint warnings from compiling the profile (dead mappings etc.).
    pub warnings: Vec<String>,
}

/// Rhai source for `bytes`, plus the mappings it must compile to (`.krx`).
fn to_source(
    format: LayoutFormat,
    bytes: &[u8],
) -> Result<(String, Option<Vec<keyrx_core::config::DeviceConfig>>), ProfileError> {
    if bytes.is_empty() {
        return Err(ProfileError::InvalidLayout("the file is empty".into()));
    }
    if bytes.len() > MAX_LAYOUT_BYTES {
        return Err(ProfileError::InvalidLayout(format!(
            "the file is {} bytes; the limit is {MAX_LAYOUT_BYTES} bytes",
            bytes.len()
        )));
    }
    match format {
        LayoutFormat::Rhai => {
            let text = std::str::from_utf8(bytes).map_err(|_| {
                ProfileError::InvalidLayout("the file is not UTF-8 text (is it a .krx?)".into())
            })?;
            if text.contains('\0') {
                return Err(ProfileError::InvalidLayout(
                    "the file contains binary data, not Rhai source".into(),
                ));
            }
            Ok((text.to_string(), None))
        }
        LayoutFormat::Krx => {
            let config = crate::config_loader::parse_config_bytes(bytes).map_err(|reason| {
                ProfileError::InvalidLayout(format!("not a valid .krx file: {reason}"))
            })?;
            let source = keyrx_core::parser::decompile::decompile(&config)
                .map_err(|e| ProfileError::InvalidLayout(e.to_string()))?;
            Ok((source, Some(config.devices)))
        }
    }
}

/// Imports `bytes` as profile `name` (everything but the service plumbing).
pub(crate) fn import_blocking(
    manager: &ProfileManager,
    name: &str,
    format: LayoutFormat,
    bytes: &[u8],
) -> Result<ImportedLayout, ProfileError> {
    let (source, expected) = to_source(format, bytes)?;
    let metadata = manager.import_source(name, &source, expected.as_deref())?;
    // Lint warnings are recomputed from the stored source; they are advisory.
    let warnings = crate::config::ProfileCompiler::new()
        .parse(&metadata.rhai_path)
        .map(|(_, warnings)| warnings)
        .unwrap_or_default();
    Ok(ImportedLayout {
        profile: ProfileInfo::from_metadata(metadata, None),
        format,
        converted: format == LayoutFormat::Krx,
        warnings,
    })
}

/// Scopes a Rhai layout written for every keyboard to one device: each
/// `device_start("*")` becomes `device_start("<pattern>")`, so a sample such as
/// `examples/user_layout.rhai` can be installed for one real keyboard without
/// hand-editing it.
///
/// # Errors
///
/// `InvalidLayout` for an empty or unquotable `pattern`, or when the source
/// has no `device_start("*")` to scope (it is never silently left unscoped).
pub fn scope_rhai_to_device(source: &str, pattern: &str) -> Result<String, ProfileError> {
    if pattern.trim().is_empty()
        || pattern
            .chars()
            .any(|c| c == '"' || c == '\\' || c.is_control())
    {
        return Err(ProfileError::InvalidLayout(format!(
            "invalid device pattern {pattern:?}: must be non-empty, without quotes, backslashes or control characters"
        )));
    }
    const WILDCARD: &str = "device_start(\"*\")";
    if !source.contains(WILDCARD) {
        return Err(ProfileError::InvalidLayout(
            "the layout has no device_start(\"*\") to scope to a device".into(),
        ));
    }
    Ok(source.replace(WILDCARD, &format!("device_start(\"{pattern}\")")))
}

#[cfg(test)]
mod scope_tests {
    use super::scope_rhai_to_device;

    #[test]
    fn rewrites_every_wildcard_block() {
        let src = "device_start(\"*\");\nmap(\"A\",\"VK_B\");\ndevice_start(\"*\");\n";
        let out = scope_rhai_to_device(src, "USB Keyboard").unwrap();
        assert_eq!(out.matches("device_start(\"USB Keyboard\")").count(), 2);
        assert!(!out.contains("\"*\""));
    }

    #[test]
    fn refuses_to_leave_a_layout_unscoped_or_inject_code() {
        assert!(scope_rhai_to_device("device_start(\"X\");", "k").is_err());
        let src = "device_start(\"*\");";
        for bad in ["", "  ", "a\"b", "a\\b", "a\nb"] {
            assert!(scope_rhai_to_device(src, bad).is_err(), "{bad:?}");
        }
    }
}

/// The wire form of an import request, shared by REST (`POST
/// /api/profiles/import`) and WS-RPC (`import_profile`) so they cannot drift.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLayoutRequest {
    /// Name of the new profile.
    pub name: String,
    /// Kind of file in `content_base64`.
    pub format: LayoutFormat,
    /// The file's bytes, standard base64.
    pub content_base64: String,
}

impl ImportLayoutRequest {
    /// The file's bytes. The encoded size is checked BEFORE decoding, so an
    /// oversized upload never allocates its decoded form.
    ///
    /// # Errors
    ///
    /// `InvalidLayout` when it is too large or not valid base64.
    pub fn decode(&self) -> Result<Vec<u8>, ProfileError> {
        use base64::Engine as _;
        if self.content_base64.len() > MAX_LAYOUT_BYTES.div_ceil(3) * 4 {
            return Err(ProfileError::InvalidLayout(format!(
                "the file is larger than the {MAX_LAYOUT_BYTES} byte limit"
            )));
        }
        base64::engine::general_purpose::STANDARD
            .decode(self.content_base64.as_bytes())
            .map_err(|e| ProfileError::InvalidLayout(format!("content is not valid base64: {e}")))
    }
}

impl ImportedLayout {
    /// The JSON every transport returns for a successful import.
    #[must_use]
    pub fn to_wire(&self, profiles_dir: &Path) -> serde_json::Value {
        let name = &self.profile.name;
        serde_json::json!({
            "success": true,
            "profile": {
                "name": name,
                "rhaiPath": profiles_dir.join(format!("{name}.rhai")).display().to_string(),
                "krxPath": profiles_dir.join(format!("{name}.krx")).display().to_string(),
                "layerCount": self.profile.layer_count,
                "keyCount": self.profile.key_count,
            },
            "format": self.format,
            "converted": self.converted,
            "warnings": self.warnings,
        })
    }
}

impl ProfileService {
    /// Imports a layout file's `bytes` as a new profile `name`; it is not
    /// activated (use [`ProfileService::activate_profile`] for that).
    ///
    /// # Errors
    ///
    /// `InvalidName`, `AlreadyExists`, `ProfileLimitExceeded`, `Compilation`
    /// (a `.rhai` that does not compile) or `InvalidLayout` (empty, too
    /// large, corrupt, wrong type or not exactly convertible). Nothing is
    /// stored on error.
    pub async fn import_layout(
        &self,
        name: &str,
        format: LayoutFormat,
        bytes: Vec<u8>,
    ) -> Result<ImportedLayout, ProfileError> {
        log::info!(
            "Importing {format:?} layout ({} bytes) as '{name}'",
            bytes.len()
        );
        let manager: Arc<ProfileManager> = Arc::clone(self.profile_manager());
        let name = name.to_string();
        tokio::task::spawn_blocking(move || import_blocking(&manager, &name, format, &bytes))
            .await
            .map_err(|e| ProfileError::LockError(format!("Task join error: {e}")))?
    }
}
