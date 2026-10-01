//! Runtime tuning that is not part of a profile: the minimum key-down time and
//! the emergency stop. One resolver, so the daemon, `doctor` and the docs all
//! agree on what is in force.
//!
//! Precedence, highest first: command-line flag, environment variable,
//! `settings.json` in the config dir, built-in default.
//!
//! | setting          | flag                   | env                        | settings.json        |
//! |------------------|------------------------|----------------------------|----------------------|
//! | min key-down ms  | `--min-key-down-ms`    | `KEYRX_MIN_KEY_DOWN_MS`    | `min_key_down_ms`    |
//! | emergency chord  | `--emergency-chord`    | `KEYRX_EMERGENCY_CHORD`    | `emergency_chord`    |
//! | emergency hold   | `--emergency-hold-ms`  | `KEYRX_EMERGENCY_HOLD_MS`  | `emergency_hold_ms`  |

use std::path::Path;
use std::time::Duration;

use crate::platform::emergency::EmergencyConfig;
use crate::platform::min_key_down::{DEFAULT_MIN_KEY_DOWN, MAX_MIN_KEY_DOWN};
use crate::services::settings_service::{DaemonSettings, SettingsService};

/// Values given on the command line (all optional).
#[derive(Debug, Clone, Default, PartialEq, Eq, clap::Args)]
pub struct OptionOverrides {
    /// Shortest time an output key stays down, in ms, so tools that poll key
    /// state per frame cannot miss a tap (default 5; 0 turns it off).
    /// Env KEYRX_MIN_KEY_DOWN_MS, settings.json min_key_down_ms.
    #[arg(long, value_name = "MS")]
    pub min_key_down_ms: Option<u64>,

    /// Emergency-stop chord: 2 to 4 keys joined by '+', e.g. LCtrl+RCtrl+Escape
    /// (the default). Env KEYRX_EMERGENCY_CHORD, settings.json emergency_chord.
    #[arg(long, value_name = "KEYS")]
    pub emergency_chord: Option<String>,

    /// One-handed emergency stop: hold Escape alone for this many ms (1000 to
    /// 30000, default 3000; 0 turns it off). Env KEYRX_EMERGENCY_HOLD_MS,
    /// settings.json emergency_hold_ms.
    #[arg(long, value_name = "MS")]
    pub emergency_hold_ms: Option<u64>,
}

/// The resolved tuning the daemon runs with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOptions {
    /// Shortest time an output key stays down; zero turns the feature off.
    pub min_key_down: Duration,
    pub emergency: EmergencyConfig,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            min_key_down: DEFAULT_MIN_KEY_DOWN,
            emergency: EmergencyConfig::default(),
        }
    }
}

impl RuntimeOptions {
    /// Resolves from the process environment and `config_dir/settings.json`.
    ///
    /// # Errors
    ///
    /// A message naming the offending setting and where it came from.
    pub fn from_environment(config_dir: &Path, cli: &OptionOverrides) -> Result<Self, String> {
        let settings = SettingsService::new(config_dir.to_path_buf()).load_settings()?;
        Self::resolve(&settings, &|name| std::env::var(name).ok(), cli)
    }

    /// Resolves from explicit sources (`env` is injected so tests need no
    /// process-global state).
    ///
    /// # Errors
    ///
    /// A message naming the offending setting and where it came from.
    pub fn resolve(
        settings: &DaemonSettings,
        env: &dyn Fn(&str) -> Option<String>,
        cli: &OptionOverrides,
    ) -> Result<Self, String> {
        let mut options = Self::default();

        let min_ms = pick(
            cli.min_key_down_ms
                .map(|v| ("--min-key-down-ms", v.to_string())),
            number_env(env, "KEYRX_MIN_KEY_DOWN_MS")?,
            settings
                .min_key_down_ms
                .map(|v| ("settings.json min_key_down_ms", v.to_string())),
        );
        if let Some((source, text)) = min_ms {
            let ms = parse_ms(&source, &text)?;
            let value = Duration::from_millis(ms);
            if value > MAX_MIN_KEY_DOWN {
                return Err(format!(
                    "{source}: {ms} ms is too long (0 turns it off, the limit is {} ms)",
                    MAX_MIN_KEY_DOWN.as_millis()
                ));
            }
            options.min_key_down = value;
        }

        let chord = pick(
            cli.emergency_chord
                .clone()
                .map(|v| ("--emergency-chord", v)),
            env("KEYRX_EMERGENCY_CHORD").map(|v| ("KEYRX_EMERGENCY_CHORD", v)),
            settings
                .emergency_chord
                .clone()
                .map(|v| ("settings.json emergency_chord", v)),
        );
        if let Some((source, text)) = chord {
            options
                .emergency
                .set_chord(&text)
                .map_err(|e| format!("{source}: {e}"))?;
        }

        let hold = pick(
            cli.emergency_hold_ms
                .map(|v| ("--emergency-hold-ms", v.to_string())),
            number_env(env, "KEYRX_EMERGENCY_HOLD_MS")?,
            settings
                .emergency_hold_ms
                .map(|v| ("settings.json emergency_hold_ms", v.to_string())),
        );
        if let Some((source, text)) = hold {
            let ms = parse_ms(&source, &text)?;
            options
                .emergency
                .set_hold_ms(ms)
                .map_err(|e| format!("{source}: {e}"))?;
        }
        Ok(options)
    }
}

/// The first present of three sources (flag, env, file), tagged with where
/// it came from for error messages.
fn pick<S: Into<String>>(
    cli: Option<(&'static str, S)>,
    env: Option<(&'static str, S)>,
    file: Option<(&'static str, S)>,
) -> Option<(String, String)> {
    cli.or(env)
        .or(file)
        .map(|(source, value)| (source.to_string(), value.into()))
}

fn number_env(
    env: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
) -> Result<Option<(&'static str, String)>, String> {
    Ok(env(name).map(|v| (name, v.trim().to_string())))
}

fn parse_ms(source: &str, text: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .map_err(|_| format!("{source}: '{text}' is not a whole number of milliseconds"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |name| map.get(name).cloned()
    }

    fn resolve(
        settings: &DaemonSettings,
        env: &[(&str, &str)],
        cli: &OptionOverrides,
    ) -> Result<RuntimeOptions, String> {
        RuntimeOptions::resolve(settings, &env_of(env), cli)
    }

    #[test]
    fn defaults_are_five_ms_min_down_and_the_documented_stop() {
        let o = resolve(&DaemonSettings::default(), &[], &OptionOverrides::default()).unwrap();
        assert_eq!(o, RuntimeOptions::default());
        assert_eq!(o.min_key_down, Duration::from_millis(5));
    }

    #[test]
    fn flag_beats_env_beats_settings_file() {
        let settings = DaemonSettings {
            min_key_down_ms: Some(30),
            ..DaemonSettings::default()
        };
        let none = OptionOverrides::default();
        let ms = |o: Result<RuntimeOptions, String>| o.unwrap().min_key_down.as_millis();
        assert_eq!(ms(resolve(&settings, &[], &none)), 30);
        assert_eq!(
            ms(resolve(
                &settings,
                &[("KEYRX_MIN_KEY_DOWN_MS", "20")],
                &none
            )),
            20
        );
        let cli = OptionOverrides {
            min_key_down_ms: Some(10),
            ..OptionOverrides::default()
        };
        assert_eq!(
            ms(resolve(&settings, &[("KEYRX_MIN_KEY_DOWN_MS", "20")], &cli)),
            10
        );
    }

    #[test]
    fn zero_turns_min_key_down_off_and_huge_values_are_rejected() {
        let none = OptionOverrides::default();
        let env0 = [("KEYRX_MIN_KEY_DOWN_MS", "0")];
        let o = resolve(&DaemonSettings::default(), &env0, &none).unwrap();
        assert_eq!(o.min_key_down, Duration::ZERO);
        let big = [("KEYRX_MIN_KEY_DOWN_MS", "5000")];
        let err = resolve(&DaemonSettings::default(), &big, &none).unwrap_err();
        assert!(
            err.contains("KEYRX_MIN_KEY_DOWN_MS") && err.contains("too long"),
            "{err}"
        );
    }

    #[test]
    fn emergency_settings_come_from_env_or_file_with_named_errors() {
        let none = OptionOverrides::default();
        let env = [
            ("KEYRX_EMERGENCY_CHORD", "LShift+RShift+Escape"),
            ("KEYRX_EMERGENCY_HOLD_MS", "0"),
        ];
        let o = resolve(&DaemonSettings::default(), &env, &none).unwrap();
        assert_eq!(o.emergency.hold, None);
        assert_eq!(o.emergency.chord.len(), 3);

        let settings = DaemonSettings {
            emergency_chord: Some("Escape".to_string()),
            ..DaemonSettings::default()
        };
        let err = resolve(&settings, &[], &none).unwrap_err();
        assert!(err.contains("settings.json emergency_chord"), "{err}");

        let err = resolve(
            &DaemonSettings::default(),
            &[("KEYRX_EMERGENCY_HOLD_MS", "soon")],
            &none,
        )
        .unwrap_err();
        assert!(
            err.contains("KEYRX_EMERGENCY_HOLD_MS") && err.contains("soon"),
            "{err}"
        );
    }
}
