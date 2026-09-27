//! Profile activation and hot-reload compilation.
//!
//! `activate` compiles a profile and atomically swaps it in as the active one;
//! `reload_active` recompiles the active profile only if its `.rhai` source changed.

use std::time::Instant;

use super::types::{ActivationResult, ProfileError, ProfileMetadata, ReloadResult};
use super::ProfileManager;

impl ProfileManager {
    /// Activate a profile with hot-reload.
    pub fn activate(&self, name: &str) -> Result<ActivationResult, ProfileError> {
        // Acquire activation lock to serialize concurrent activations
        let _lock = self.activation_lock.lock().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire activation lock: {}", e))
        })?;

        let start = Instant::now();

        // Get profile metadata
        let profiles = self.profiles.read().map_err(|e| {
            ProfileError::LockError(format!("Failed to acquire profiles read lock: {}", e))
        })?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| ProfileError::NotFound(name.to_string()))?
            .clone();
        drop(profiles);

        // Compile and reload
        let (compile_time, reload_time) = match self.compile_and_reload(name, &profile) {
            Ok(times) => times,
            Err((compile_time, e)) => {
                return Ok(ActivationResult {
                    compile_time_ms: compile_time,
                    reload_time_ms: 0,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        };

        log::info!(
            "Profile '{}' activated in {}ms (compile: {}ms, reload: {}ms)",
            name,
            start.elapsed().as_millis(),
            compile_time,
            reload_time
        );

        Ok(ActivationResult {
            compile_time_ms: compile_time,
            reload_time_ms: reload_time,
            success: true,
            error: None,
        })
    }

    /// Compile and reload a profile.
    /// PROF-003: Enhanced error handling with structured errors and context.
    fn compile_and_reload(
        &self,
        name: &str,
        profile: &ProfileMetadata,
    ) -> Result<(u64, u64), (u64, ProfileError)> {
        // PROF-003: Validate profile exists before attempting compilation
        if !profile.rhai_path.exists() {
            return Err((
                0,
                ProfileError::NotFound(format!(
                    "Profile '{}' source file not found at {:?}",
                    name, profile.rhai_path
                )),
            ));
        }

        // Compile .rhai → .krx with timeout
        let compile_result = self
            .compiler
            .compile_profile(&profile.rhai_path, &profile.krx_path);

        let compile_time = match compile_result {
            Ok(result) => result.compile_time_ms,
            Err(e) => {
                // PROF-003: Return structured compilation error with context
                log::error!("Compilation failed for profile '{}': {}", name, e);
                return Err((0, ProfileError::Compilation(e)));
            }
        };

        // Atomic swap
        let reload_start = Instant::now();
        *self.active_profile.write().map_err(|e| {
            (
                compile_time,
                ProfileError::LockError(format!(
                    "Failed to acquire write lock during activation of '{}': {}",
                    name, e
                )),
            )
        })? = Some(name.to_string());
        let reload_time = reload_start.elapsed().as_millis() as u64;

        // PROF-003: Persist active profile to disk with proper error handling
        if let Err(e) = self.save_active_profile(name) {
            log::warn!(
                "Failed to persist active profile '{}' (non-fatal): {}",
                name,
                e
            );
            // Don't fail the activation, but log the issue
        }

        Ok((compile_time, reload_time))
    }

    /// Reload the active profile, recompiling if .rhai is newer than .krx.
    ///
    /// Returns `ReloadResult` indicating whether recompilation occurred.
    /// If no profile is active, returns an error.
    pub fn reload_active(&self) -> Result<ReloadResult, ProfileError> {
        let active_name = self
            .get_active()?
            .ok_or_else(|| ProfileError::NotFound("No active profile".to_string()))?;

        let profile = self
            .get(&active_name)
            .ok_or_else(|| ProfileError::NotFound(active_name.clone()))?;

        if !profile.rhai_path.exists() {
            return Err(ProfileError::NotFound(format!(
                "Source file missing: {:?}",
                profile.rhai_path
            )));
        }

        let needs_compile = if profile.krx_path.exists() {
            let rhai_modified = profile.rhai_path.metadata()?.modified()?;
            let krx_modified = profile.krx_path.metadata()?.modified()?;
            rhai_modified > krx_modified
        } else {
            true
        };

        if !needs_compile {
            log::info!(
                "Profile '{}': .krx is up-to-date, skipping compilation",
                active_name
            );
            return Ok(ReloadResult {
                recompiled: false,
                compile_time_ms: 0,
                success: true,
                error: None,
            });
        }

        log::info!(
            "Profile '{}': .rhai is newer than .krx, recompiling",
            active_name
        );

        match self
            .compiler
            .compile_profile(&profile.rhai_path, &profile.krx_path)
        {
            Ok(result) => {
                // Update metadata (modified time changed)
                if let Ok(updated) = self.load_profile_metadata(&active_name) {
                    if let Ok(mut profiles) = self.profiles.write() {
                        profiles.insert(active_name.clone(), updated);
                    }
                }

                Ok(ReloadResult {
                    recompiled: true,
                    compile_time_ms: result.compile_time_ms,
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                log::error!("Compilation failed for profile '{}': {}", active_name, e);
                Ok(ReloadResult {
                    recompiled: false,
                    compile_time_ms: 0,
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}
