//! Validation and parsing utilities for device discovery.
//!
//! Handles device ID parsing, layout prompting, user confirmation, and error types.

use crate::discovery::DeviceId;
use std::io::{self, Write};
use thiserror::Error;

/// Exit reasons that map to specific CLI exit codes.
#[derive(Debug, Error)]
pub enum DiscoverExit {
    #[error("discovery cancelled")]
    Cancelled,
    #[error("discovery validation failed: {0}")]
    Validation(String),
}

/// Parse a device identifier string in "vendor:product" format.
///
/// Supports both hexadecimal (with 0x prefix) and decimal values.
pub fn parse_device_id(device: &str) -> Result<DeviceId, DiscoverExit> {
    let parts: Vec<_> = device.split(':').collect();
    if parts.len() != 2 {
        return Err(DiscoverExit::Validation(
            "Device must be in format vendor:product (hex or decimal)".to_string(),
        ));
    }

    let vendor_id = parse_u16(parts[0])?;
    let product_id = parse_u16(parts[1])?;
    Ok(DeviceId::new(vendor_id, product_id))
}

/// Parse a string as u16, supporting hex (0x prefix) or decimal.
pub fn parse_u16(value: &str) -> Result<u16, DiscoverExit> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16)
            .map_err(|_| DiscoverExit::Validation(format!("Invalid hex value: {value}")))
    } else {
        value
            .parse::<u16>()
            .map_err(|_| DiscoverExit::Validation(format!("Invalid number: {value}")))
    }
}

/// Return the default keyboard layout dimensions.
pub fn default_layout() -> (u8, Vec<u8>) {
    (5, vec![14, 14, 13, 13, 7])
}

/// Prompt the user for keyboard layout dimensions interactively.
pub fn prompt_for_layout() -> Result<(u8, Vec<u8>), DiscoverExit> {
    let (default_rows, default_cols) = default_layout();
    println!("Enter number of rows for this keyboard [default {default_rows}]: ");
    print_flush();

    let mut rows_input = String::new();
    io::stdin()
        .read_line(&mut rows_input)
        .map_err(|err| DiscoverExit::Validation(format!("Failed to read input: {err}")))?;
    let rows = if rows_input.trim().is_empty() {
        default_rows
    } else {
        rows_input
            .trim()
            .parse::<u8>()
            .map_err(|_| DiscoverExit::Validation("Rows must be a number".to_string()))?
    };

    println!(
        "Enter columns per row separated by commas [default {}]: ",
        default_cols
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    print_flush();

    let mut cols_input = String::new();
    io::stdin()
        .read_line(&mut cols_input)
        .map_err(|err| DiscoverExit::Validation(format!("Failed to read input: {err}")))?;

    let cols_per_row = if cols_input.trim().is_empty() {
        default_cols
    } else {
        cols_input
            .trim()
            .split(',')
            .map(|v| {
                v.trim()
                    .parse::<u8>()
                    .map_err(|_| DiscoverExit::Validation("Columns must be numbers".to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    validate_layout(rows, &cols_per_row)?;
    Ok((rows, cols_per_row))
}

/// Validate layout dimensions are sensible.
pub fn validate_layout(rows: u8, cols_per_row: &[u8]) -> Result<(), DiscoverExit> {
    if cols_per_row.len() != rows as usize || rows == 0 || cols_per_row.contains(&0) {
        return Err(DiscoverExit::Validation(
            "Rows must match columns length and all rows must be non-zero".to_string(),
        ));
    }
    Ok(())
}

/// Prompt user for y/n confirmation.
pub fn confirm(prompt: &str) -> Result<bool, DiscoverExit> {
    print!("{prompt}");
    print_flush();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| DiscoverExit::Validation(format!("Failed to read input: {err}")))?;
    let accepted = matches!(input.to_lowercase().trim(), "y" | "yes");
    Ok(accepted)
}

fn print_flush() {
    let _ = io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_device_id_hex_and_dec() {
        let hex = parse_device_id("0x1234:0xabcd").unwrap();
        assert_eq!(hex.vendor_id, 0x1234);
        assert_eq!(hex.product_id, 0xABCD);

        let dec = parse_device_id("4660:43981").unwrap();
        assert_eq!(dec.vendor_id, 0x1234);
        assert_eq!(dec.product_id, 0xABCD);

        let err = parse_device_id("bad").unwrap_err();
        assert!(matches!(err, DiscoverExit::Validation(_)));
    }

    #[test]
    fn parse_u16_variants() {
        assert_eq!(parse_u16("0x1234").unwrap(), 0x1234);
        assert_eq!(parse_u16("0XABCD").unwrap(), 0xABCD);
        assert_eq!(parse_u16("1234").unwrap(), 1234);
        assert!(parse_u16("invalid").is_err());
    }

    #[test]
    fn validate_layout_checks() {
        assert!(validate_layout(2, &[10, 10]).is_ok());
        assert!(validate_layout(0, &[]).is_err()); // zero rows
        assert!(validate_layout(2, &[10]).is_err()); // mismatch
        assert!(validate_layout(2, &[10, 0]).is_err()); // zero col
    }
}
