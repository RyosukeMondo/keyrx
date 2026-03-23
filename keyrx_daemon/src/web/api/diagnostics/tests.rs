use super::*;

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1024), "1.00 KB");
    assert_eq!(format_bytes(1536), "1.50 KB");
    assert_eq!(format_bytes(1048576), "1.00 MB");
    assert_eq!(format_bytes(1073741824), "1.00 GB");
}

#[test]
fn test_platform_info() {
    let platform_info = PlatformInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    };
    assert!(!platform_info.os.is_empty());
    assert!(!platform_info.arch.is_empty());
}

#[test]
fn test_diagnostics_response_serialization() {
    let response = DiagnosticsResponse {
        version: "0.1.0".to_string(),
        build_time: "2024-01-01".to_string(),
        git_hash: "abc123".to_string(),
        binary_timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        admin_status: true,
        hook_status: HookStatus {
            installed: true,
            remapped_keys_count: 5,
        },
        platform_info: PlatformInfo {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
        },
        memory_usage: MemoryUsage {
            process_memory_bytes: 10485760,
            process_memory_human: "10.00 MB".to_string(),
        },
        config_validation_status: ConfigStatus {
            valid: true,
            message: "Configuration is valid".to_string(),
        },
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"version\":\"0.1.0\""));
    assert!(json.contains("\"admin_status\":true"));
}
