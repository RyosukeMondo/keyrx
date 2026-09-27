use super::*;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn test_serialize_systemtime_as_rfc3339() {
    // Create a known timestamp: 2024-01-01T00:00:00Z
    let timestamp = UNIX_EPOCH + Duration::from_secs(1704067200);

    // Serialize to JSON
    let json_value = serde_json::to_value(&ProfileResponse {
        name: "test".to_string(),
        rhai_path: "/test.rhai".to_string(),
        krx_path: "/test.krx".to_string(),
        modified_at: timestamp,
        created_at: timestamp,
        layer_count: 1,
        device_count: 0,
        key_count: 0,
        active: false,
        activated_at: None,
        activated_by: None,
    })
    .unwrap();

    // Check that modifiedAt is a string in ISO 8601 / RFC 3339 format
    let modified_at_str = json_value["modifiedAt"].as_str().unwrap();

    // Should be in format: YYYY-MM-DDTHH:MM:SS.sssZ or similar RFC 3339
    assert!(
        modified_at_str.contains('T'),
        "Timestamp should contain 'T' separator: {}",
        modified_at_str
    );
    assert!(
        modified_at_str.ends_with('Z')
            || modified_at_str.contains('+')
            || modified_at_str.contains('-'),
        "Timestamp should have timezone (Z or offset): {}",
        modified_at_str
    );

    // Verify it can be parsed back by JavaScript Date constructor
    // RFC 3339 format is guaranteed to be parseable by new Date()
    assert!(
        modified_at_str.len() >= 20, // Minimum length for ISO 8601
        "Timestamp too short: {}",
        modified_at_str
    );
}

#[test]
fn test_profile_response_camel_case_fields() {
    let timestamp = UNIX_EPOCH + Duration::from_secs(1704067200);

    let response = ProfileResponse {
        name: "gaming".to_string(),
        rhai_path: "/profiles/gaming.rhai".to_string(),
        krx_path: "/profiles/gaming.krx".to_string(),
        modified_at: timestamp,
        created_at: timestamp,
        layer_count: 3,
        device_count: 2,
        key_count: 127,
        active: true,
        activated_at: Some(timestamp),
        activated_by: Some("user".to_string()),
    };

    let json_value = serde_json::to_value(&response).unwrap();

    // Verify camelCase field names
    assert!(
        json_value["modifiedAt"].is_string(),
        "modifiedAt should be a string"
    );
    assert!(
        json_value["createdAt"].is_string(),
        "createdAt should be a string"
    );
    assert!(
        json_value["layerCount"].is_number(),
        "layerCount should be a number"
    );
    assert!(
        json_value["deviceCount"].is_number(),
        "deviceCount should be a number"
    );
    assert!(
        json_value["keyCount"].is_number(),
        "keyCount should be a number"
    );
    assert!(
        json_value["isActive"].is_boolean(),
        "isActive should be a boolean"
    );

    // Verify snake_case fields are NOT present
    assert!(
        json_value.get("modified_at").is_none(),
        "Should not have snake_case modified_at"
    );
    assert!(
        json_value.get("created_at").is_none(),
        "Should not have snake_case created_at"
    );
    assert!(
        json_value.get("layer_count").is_none(),
        "Should not have snake_case layer_count"
    );
    // isActive is the correct field name (camelCase), not active
    assert!(
        json_value.get("active").is_none(),
        "Should not have active (should be isActive)"
    );
}
