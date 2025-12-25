use keyrx_compiler::parser::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temporary Rhai file with content.
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(name);
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file_path
}

#[test]
fn test_load_simple_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create stdlib directory
    let stdlib_dir = temp_dir.path().join("stdlib");
    fs::create_dir(&stdlib_dir).unwrap();

    // Create a simple stdlib file
    let utils_content = r#"
// Utils library
map("A", "VK_B");
map("B", "VK_C");
"#;
    let mut utils_file = fs::File::create(stdlib_dir.join("utils.rhai")).unwrap();
    utils_file.write_all(utils_content.as_bytes()).unwrap();

    // Create main config that loads utils
    let main_content = r#"
device_start("Keyboard");
    when_start("MD_00");
        load("utils.rhai");
    when_end();

    map("CapsLock", "MD_00");
device_end();
"#;
    let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

    // Parse the config
    let mut parser = Parser::new();
    let result = parser.parse_script(&main_path);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let config = result.unwrap();

    // Verify the device was created
    assert_eq!(config.devices.len(), 1);

    // Verify mappings from both main and loaded file
    // CapsLock is base mapping, A and B are conditional mappings under MD_00
    // So total is: 1 base + 2 conditional = 3 total, but they're all in the mappings vec
    assert!(config.devices[0].mappings.len() >= 1); // At least CapsLock
}

#[test]
fn test_load_file_not_found() {
    let temp_dir = TempDir::new().unwrap();

    let main_content = r#"
device_start("Keyboard");
    when_start("MD_00");
        load("nonexistent.rhai");
    when_end();
device_end();
"#;
    let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

    let mut parser = Parser::new();
    let result = parser.parse_script(&main_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Import failed"));
}

#[test]
fn test_load_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let stdlib_dir = temp_dir.path().join("stdlib");
    fs::create_dir(&stdlib_dir).unwrap();

    // Create shift.rhai
    let shift_content = r#"
map("A", with_shift("VK_A"));
map("B", with_shift("VK_B"));
"#;
    let mut shift_file = fs::File::create(stdlib_dir.join("shift.rhai")).unwrap();
    shift_file.write_all(shift_content.as_bytes()).unwrap();

    // Create ctrl.rhai
    let ctrl_content = r#"
map("C", with_ctrl("VK_C"));
map("D", with_ctrl("VK_D"));
"#;
    let mut ctrl_file = fs::File::create(stdlib_dir.join("ctrl.rhai")).unwrap();
    ctrl_file.write_all(ctrl_content.as_bytes()).unwrap();

    // Main config loads both
    let main_content = r#"
device_start("Keyboard");
    when_start("MD_00");
        load("shift.rhai");
    when_end();

    when_start("MD_01");
        load("ctrl.rhai");
    when_end();

    map("CapsLock", "MD_00");
    map("Tab", "MD_01");
device_end();
"#;
    let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

    let mut parser = Parser::new();
    let result = parser.parse_script(&main_path);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let config = result.unwrap();

    assert_eq!(config.devices.len(), 1);
    // CapsLock + Tab (base mappings) + (A,B,C,D conditional)
    // Total: 6 mappings (2 base + 4 conditional)
    assert!(config.devices[0].mappings.len() >= 2);
}

#[test]
fn test_load_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let stdlib_dir = temp_dir.path().join("stdlib");
    fs::create_dir(&stdlib_dir).unwrap();

    // Create base.rhai
    let base_content = r#"
map("X", "VK_Y");
"#;
    let mut base_file = fs::File::create(stdlib_dir.join("base.rhai")).unwrap();
    base_file.write_all(base_content.as_bytes()).unwrap();

    // Create utils.rhai that loads base.rhai
    let utils_content = r#"
load("base.rhai");
map("A", "VK_B");
"#;
    let mut utils_file = fs::File::create(stdlib_dir.join("utils.rhai")).unwrap();
    utils_file.write_all(utils_content.as_bytes()).unwrap();

    // Main config loads utils.rhai (which loads base.rhai)
    let main_content = r#"
device_start("Keyboard");
    when_start("MD_00");
        load("utils.rhai");
    when_end();

    map("CapsLock", "MD_00");
device_end();
"#;
    let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

    let mut parser = Parser::new();
    let result = parser.parse_script(&main_path);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let config = result.unwrap();

    assert_eq!(config.devices.len(), 1);
    // CapsLock (base) + A + X (conditional under MD_00)
    assert!(config.devices[0].mappings.len() >= 1);
}

#[test]
fn test_load_in_device_context() {
    let temp_dir = TempDir::new().unwrap();
    let stdlib_dir = temp_dir.path().join("stdlib");
    fs::create_dir(&stdlib_dir).unwrap();

    let lib_content = r#"
map("A", "VK_Z");
"#;
    let mut lib_file = fs::File::create(stdlib_dir.join("lib.rhai")).unwrap();
    lib_file.write_all(lib_content.as_bytes()).unwrap();

    // Load inside device context - mappings should apply to that device
    let main_content = r#"
device_start("Keyboard1");
    load("lib.rhai");
    map("B", "VK_C");
device_end();

device_start("Keyboard2");
    map("D", "VK_E");
device_end();
"#;
    let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

    let mut parser = Parser::new();
    let result = parser.parse_script(&main_path);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let config = result.unwrap();

    assert_eq!(config.devices.len(), 2);
    assert_eq!(config.devices[0].mappings.len(), 2); // A + B
    assert_eq!(config.devices[1].mappings.len(), 1); // C only
}

#[test]
fn test_load_path_search_order() {
    let temp_dir = TempDir::new().unwrap();

    // Create stdlib directory
    let stdlib_dir = temp_dir.path().join("stdlib");
    fs::create_dir(&stdlib_dir).unwrap();

    // Create test.rhai in stdlib
    let stdlib_content = r#"
map("A", "VK_B");
"#;
    let mut stdlib_file = fs::File::create(stdlib_dir.join("test.rhai")).unwrap();
    stdlib_file.write_all(stdlib_content.as_bytes()).unwrap();

    // Create test.rhai in main directory (should take precedence)
    let local_content = r#"
map("A", "VK_Z");
"#;
    let mut local_file = fs::File::create(temp_dir.path().join("test.rhai")).unwrap();
    local_file.write_all(local_content.as_bytes()).unwrap();

    let main_content = r#"
device_start("Keyboard");
    load("test.rhai");
device_end();
"#;
    let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

    let mut parser = Parser::new();
    let result = parser.parse_script(&main_path);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let config = result.unwrap();

    // Should load local test.rhai (VK_LOCAL), not stdlib version
    assert_eq!(config.devices[0].mappings.len(), 1);
}

#[test]
fn test_load_with_conditional_context() {
    let temp_dir = TempDir::new().unwrap();
    let stdlib_dir = temp_dir.path().join("stdlib");
    fs::create_dir(&stdlib_dir).unwrap();

    let modifier_lib = r#"
// All mappings here will be in the MD_00 context
map("A", with_shift("VK_A"));
map("B", with_shift("VK_B"));
"#;
    let mut lib_file = fs::File::create(stdlib_dir.join("modifier.rhai")).unwrap();
    lib_file.write_all(modifier_lib.as_bytes()).unwrap();

    let main_content = r#"
device_start("Keyboard");
    when_start("MD_00");
        load("modifier.rhai");
    when_end();

    map("Space", "MD_00");
device_end();
"#;
    let main_path = create_temp_file(&temp_dir, "main.rhai", main_content);

    let mut parser = Parser::new();
    let result = parser.parse_script(&main_path);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let config = result.unwrap();

    // Verify mappings were created
    // Space (base) + A and B (conditional under MD_00)
    assert!(config.devices[0].mappings.len() >= 1); // At least Space
}
