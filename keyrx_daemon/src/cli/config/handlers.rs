//! Command handlers that coordinate input, service, and output layers.

use super::input::parse_macro_sequence;
use super::output::*;
use super::service::ProfileService;
use crate::cli::logging;
use crate::config::rhai_generator::KeyAction;
use crate::error::DaemonResult;

/// Handles set-key command.
pub fn handle_set_key(
    mut service: ProfileService,
    key: String,
    target: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    logging::log_command_start(
        "config set-key",
        &format!("{} -> {} (layer: {})", key, target, layer),
    );

    let profile_name = service.get_profile_name(profile)?;
    let action = KeyAction::SimpleRemap {
        output: target.clone(),
    };

    let compile_time = service.apply_key_mapping(&profile_name, &layer, &key, action)?;

    logging::log_config_change(&profile_name, "set_key", &key, &layer);
    logging::log_command_success("config set-key", compile_time);

    let output = format_set_key_result(key, target, layer, profile_name, compile_time, json);
    println!("{}", output);

    Ok(())
}

/// Handles set-tap-hold command.
#[allow(clippy::too_many_arguments)]
pub fn handle_set_tap_hold(
    mut service: ProfileService,
    key: String,
    tap: String,
    hold: String,
    threshold: u16,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = service.get_profile_name(profile)?;
    let action = KeyAction::TapHold {
        tap: tap.clone(),
        hold: hold.clone(),
        threshold_ms: threshold,
    };

    let compile_time = service.apply_key_mapping(&profile_name, &layer, &key, action)?;

    if json {
        let output = SetKeyOutput {
            success: true,
            key,
            layer,
            profile: profile_name,
            compile_time_ms: Some(compile_time),
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(crate::error::CliError::from)?
        );
    } else {
        println!(
            "✓ Set {} -> tap:{} hold:{} ({}ms) in layer '{}' of profile '{}'",
            key, tap, hold, threshold, layer, profile_name
        );
        println!("  Compiled in {}ms", compile_time);
    }

    Ok(())
}

/// Handles set-macro command.
pub fn handle_set_macro(
    mut service: ProfileService,
    key: String,
    sequence: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = service.get_profile_name(profile)?;
    let macro_steps = parse_macro_sequence(&sequence)?;
    let action = KeyAction::Macro {
        sequence: macro_steps,
    };

    let compile_time = service.apply_key_mapping(&profile_name, &layer, &key, action)?;

    if json {
        let output = SetKeyOutput {
            success: true,
            key,
            layer,
            profile: profile_name,
            compile_time_ms: Some(compile_time),
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(crate::error::CliError::from)?
        );
    } else {
        println!(
            "✓ Set {} -> macro in layer '{}' of profile '{}'",
            key, layer, profile_name
        );
        println!("  Compiled in {}ms", compile_time);
    }

    Ok(())
}

/// Handles get-key command.
pub fn handle_get_key(
    service: ProfileService,
    key: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = service.get_profile_name(profile)?;
    let mapping = service.get_key_mapping(&profile_name, &layer, &key)?;

    if json {
        let output = GetKeyOutput {
            key,
            layer,
            mapping,
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(crate::error::CliError::from)?
        );
    } else if let Some(m) = mapping {
        println!("{}", m);
    } else {
        println!("No mapping found for {} in layer '{}'", key, layer);
    }

    Ok(())
}

/// Handles delete-key command.
pub fn handle_delete_key(
    mut service: ProfileService,
    key: String,
    layer: String,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = service.get_profile_name(profile)?;
    let compile_time = service.delete_key_mapping(&profile_name, &layer, &key)?;

    if json {
        let output = serde_json::json!({
            "success": true,
            "key": key,
            "layer": layer,
            "profile": profile_name,
            "compile_time_ms": compile_time,
        });
        println!(
            "{}",
            serde_json::to_string(&output).map_err(crate::error::CliError::from)?
        );
    } else {
        println!(
            "✓ Deleted mapping for {} in layer '{}' of profile '{}'",
            key, layer, profile_name
        );
        println!("  Compiled in {}ms", compile_time);
    }

    Ok(())
}

/// Handles validate command.
pub fn handle_validate(
    service: ProfileService,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = service.get_profile_name(profile)?;

    logging::log_command_start("config validate", &profile_name);

    match service.validate_profile(&profile_name) {
        Ok(_) => {
            logging::log_config_validate(&profile_name, true, None);
            logging::log_command_success("config validate", 0);

            let output = format_validation_result(profile_name, true, None, json);
            println!("{}", output);
            Ok(())
        }
        Err(e) => {
            let error_msg = e.to_string();
            logging::log_config_validate(&profile_name, false, Some(&error_msg));
            logging::log_command_error("config validate", &error_msg);

            let output = format_validation_result(profile_name, false, Some(error_msg), json);
            println!("{}", output);
            Err(e)
        }
    }
}

/// Handles show command.
pub fn handle_show(
    service: ProfileService,
    profile: Option<String>,
    json: bool,
) -> DaemonResult<()> {
    let profile_name = service.get_profile_name(profile)?;
    let (device_id, layers, mapping_count) = service.get_profile_info(&profile_name)?;

    if json {
        let output = ShowOutput {
            profile: profile_name,
            device_id,
            layers,
            mapping_count,
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(crate::error::CliError::from)?
        );
    } else {
        println!("Profile: {}", profile_name);
        println!("Device ID: {}", device_id);
        println!("Layers: {}", layers.join(", "));
        println!("Mappings: {}", mapping_count);
    }

    Ok(())
}

/// Handles diff command.
pub fn handle_diff(
    service: ProfileService,
    profile1: String,
    profile2: String,
    json: bool,
) -> DaemonResult<()> {
    let differences = service.compare_profiles(&profile1, &profile2)?;

    if json {
        let output = DiffOutput {
            profile1,
            profile2,
            differences,
        };
        println!(
            "{}",
            serde_json::to_string(&output).map_err(crate::error::CliError::from)?
        );
    } else if differences.is_empty() {
        println!("No differences between '{}' and '{}'", profile1, profile2);
    } else {
        println!("Differences between '{}' and '{}':", profile1, profile2);
        for diff in differences {
            println!("  {}", diff);
        }
    }

    Ok(())
}
