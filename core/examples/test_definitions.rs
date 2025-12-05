use keyrx_core::definitions::DeviceDefinition;
use std::fs;

fn main() {
    println!("Testing device definitions...\n");

    // Test ANSI definition
    println!("Loading ANSI 104-key definition...");
    let ansi_content = fs::read_to_string("device_definitions/standard/ansi-104.toml")
        .expect("Failed to read ANSI definition");
    let ansi_def: DeviceDefinition =
        toml::from_str(&ansi_content).expect("Failed to parse ANSI definition");
    println!("  Name: {}", ansi_def.name);
    println!(
        "  VID:PID: {:04x}:{:04x}",
        ansi_def.vendor_id, ansi_def.product_id
    );
    println!(
        "  Layout: {} ({}x{:?})",
        ansi_def.layout.layout_type, ansi_def.layout.rows, ansi_def.layout.cols_per_row
    );
    println!("  Matrix entries: {}", ansi_def.matrix_map.len());

    ansi_def.validate().expect("ANSI validation failed");
    println!("  ✓ Validation passed!\n");

    // Test ISO definition
    println!("Loading ISO 105-key definition...");
    let iso_content = fs::read_to_string("device_definitions/standard/iso-105.toml")
        .expect("Failed to read ISO definition");
    let iso_def: DeviceDefinition =
        toml::from_str(&iso_content).expect("Failed to parse ISO definition");
    println!("  Name: {}", iso_def.name);
    println!(
        "  VID:PID: {:04x}:{:04x}",
        iso_def.vendor_id, iso_def.product_id
    );
    println!(
        "  Layout: {} ({}x{:?})",
        iso_def.layout.layout_type, iso_def.layout.rows, iso_def.layout.cols_per_row
    );
    println!("  Matrix entries: {}", iso_def.matrix_map.len());

    iso_def.validate().expect("ISO validation failed");
    println!("  ✓ Validation passed!\n");

    println!("All device definitions are valid!");
}
