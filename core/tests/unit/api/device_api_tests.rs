//! Unit tests for device API methods using MockDeviceService.
//!
//! These tests verify ApiContext device operations work correctly with mocked
//! dependencies, enabling fast, isolated testing without I/O.

use std::sync::Arc;

use keyrx_core::api::ApiContext;
use keyrx_core::services::device::{DeviceServiceError, DeviceView};
use keyrx_core::services::{MockDeviceService, MockProfileService, MockRuntimeService};

/// Creates a test device with the given key.
fn test_device(key: &str) -> DeviceView {
    DeviceView {
        key: key.to_string(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial_number: "test-serial".to_string(),
        label: None,
        remap_enabled: false,
        profile_id: None,
        connected: true,
    }
}

/// Creates a test device with custom fields.
fn test_device_with_options(
    key: &str,
    label: Option<&str>,
    remap_enabled: bool,
    profile_id: Option<&str>,
) -> DeviceView {
    DeviceView {
        key: key.to_string(),
        vendor_id: 0x1234,
        product_id: 0x5678,
        serial_number: "test-serial".to_string(),
        label: label.map(String::from),
        remap_enabled,
        profile_id: profile_id.map(String::from),
        connected: true,
    }
}

/// Helper to create ApiContext with mocked services.
fn create_api_with_device_mock(device_mock: MockDeviceService) -> ApiContext {
    ApiContext::new(
        Arc::new(device_mock),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    )
}

#[tokio::test]
async fn test_list_devices_returns_mock_data() {
    let devices = vec![
        test_device("device-1"),
        test_device("device-2"),
        test_device("device-3"),
    ];

    let mock = MockDeviceService::new().with_devices(devices.clone());
    let api = create_api_with_device_mock(mock);

    let result = api.list_devices().await.unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].key, "device-1");
    assert_eq!(result[1].key, "device-2");
    assert_eq!(result[2].key, "device-3");
}

#[tokio::test]
async fn test_list_devices_empty() {
    let mock = MockDeviceService::new();
    let api = create_api_with_device_mock(mock);

    let result = api.list_devices().await.unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_get_device_success() {
    let devices = vec![
        test_device_with_options("device-1", Some("My Keyboard"), true, Some("profile-1")),
        test_device("device-2"),
    ];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api.get_device("device-1".to_string()).await.unwrap();

    assert_eq!(result.key, "device-1");
    assert_eq!(result.label, Some("My Keyboard".to_string()));
    assert!(result.remap_enabled);
    assert_eq!(result.profile_id, Some("profile-1".to_string()));
}

#[tokio::test]
async fn test_get_device_not_found() {
    let devices = vec![test_device("device-1")];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api.get_device("nonexistent".to_string()).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("nonexistent") || error_msg.contains("not found"));
}

#[tokio::test]
async fn test_set_device_remap_enabled() {
    let devices = vec![test_device("device-1")];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api
        .set_device_remap("device-1".to_string(), true)
        .await
        .unwrap();

    assert_eq!(result.key, "device-1");
}

#[tokio::test]
async fn test_set_device_remap_device_not_found() {
    let mock = MockDeviceService::new();
    let api = create_api_with_device_mock(mock);

    let result = api.set_device_remap("nonexistent".to_string(), true).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_assign_device_profile_success() {
    let devices = vec![test_device("device-1")];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api
        .assign_device_profile("device-1".to_string(), "profile-1".to_string())
        .await
        .unwrap();

    assert_eq!(result.key, "device-1");
}

#[tokio::test]
async fn test_assign_device_profile_device_not_found() {
    let mock = MockDeviceService::new();
    let api = create_api_with_device_mock(mock);

    let result = api
        .assign_device_profile("nonexistent".to_string(), "profile-1".to_string())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_unassign_device_profile_success() {
    let devices = vec![test_device_with_options(
        "device-1",
        None,
        false,
        Some("profile-1"),
    )];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api
        .unassign_device_profile("device-1".to_string())
        .await
        .unwrap();

    assert_eq!(result.key, "device-1");
}

#[tokio::test]
async fn test_set_device_label_success() {
    let devices = vec![test_device("device-1")];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api
        .set_device_label("device-1".to_string(), Some("New Label".to_string()))
        .await
        .unwrap();

    assert_eq!(result.key, "device-1");
}

#[tokio::test]
async fn test_set_device_label_clear() {
    let devices = vec![test_device_with_options(
        "device-1",
        Some("Old Label"),
        false,
        None,
    )];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api
        .set_device_label("device-1".to_string(), None)
        .await
        .unwrap();

    assert_eq!(result.key, "device-1");
}

#[tokio::test]
async fn test_api_handles_io_error() {
    let mock = MockDeviceService::new().with_list_error(DeviceServiceError::Io(
        std::io::Error::other("disk failure"),
    ));
    let api = create_api_with_device_mock(mock);

    let result = api.list_devices().await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("disk failure"));
}

#[tokio::test]
async fn test_api_handles_get_device_error() {
    let mock = MockDeviceService::new()
        .with_devices(vec![test_device("device-1")])
        .with_get_error(DeviceServiceError::Io(std::io::Error::other(
            "connection lost",
        )));
    let api = create_api_with_device_mock(mock);

    let result = api.get_device("device-1".to_string()).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("connection lost"));
}

#[tokio::test]
async fn test_mock_tracks_method_calls() {
    let devices = vec![test_device("device-1")];
    let mock = Arc::new(MockDeviceService::new().with_devices(devices));
    let api = ApiContext::new(
        mock.clone(),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    );

    // Perform multiple operations
    let _ = api.list_devices().await;
    let _ = api.list_devices().await;
    let _ = api.get_device("device-1".to_string()).await;
    let _ = api.set_device_remap("device-1".to_string(), true).await;
    let _ = api
        .assign_device_profile("device-1".to_string(), "profile".to_string())
        .await;
    let _ = api.unassign_device_profile("device-1".to_string()).await;
    let _ = api
        .set_device_label("device-1".to_string(), Some("label".to_string()))
        .await;

    // Verify call counts
    assert_eq!(mock.get_call_count("list_devices"), 2);
    assert_eq!(mock.get_call_count("get_device"), 1);
    assert_eq!(mock.get_call_count("set_remap_enabled"), 1);
    assert_eq!(mock.get_call_count("assign_profile"), 1);
    assert_eq!(mock.get_call_count("unassign_profile"), 1);
    assert_eq!(mock.get_call_count("set_label"), 1);
}

#[tokio::test]
async fn test_multiple_devices_with_various_states() {
    let devices = vec![
        test_device_with_options(
            "kb-1",
            Some("Gaming Keyboard"),
            true,
            Some("gaming-profile"),
        ),
        test_device_with_options("kb-2", Some("Work Keyboard"), false, None),
        test_device_with_options("kb-3", None, true, Some("default-profile")),
    ];

    let mock = MockDeviceService::new().with_devices(devices);
    let api = create_api_with_device_mock(mock);

    let result = api.list_devices().await.unwrap();

    assert_eq!(result.len(), 3);

    // Verify first device
    let kb1 = result.iter().find(|d| d.key == "kb-1").unwrap();
    assert_eq!(kb1.label, Some("Gaming Keyboard".to_string()));
    assert!(kb1.remap_enabled);
    assert_eq!(kb1.profile_id, Some("gaming-profile".to_string()));

    // Verify second device
    let kb2 = result.iter().find(|d| d.key == "kb-2").unwrap();
    assert_eq!(kb2.label, Some("Work Keyboard".to_string()));
    assert!(!kb2.remap_enabled);
    assert_eq!(kb2.profile_id, None);

    // Verify third device
    let kb3 = result.iter().find(|d| d.key == "kb-3").unwrap();
    assert_eq!(kb3.label, None);
    assert!(kb3.remap_enabled);
    assert_eq!(kb3.profile_id, Some("default-profile".to_string()));
}

#[tokio::test]
async fn test_api_context_new_accepts_trait_objects() {
    // This test verifies that ApiContext::new() correctly accepts Arc<dyn Trait>
    let device_mock: Arc<dyn keyrx_core::services::DeviceServiceTrait> =
        Arc::new(MockDeviceService::new());
    let profile_mock: Arc<dyn keyrx_core::services::ProfileServiceTrait> =
        Arc::new(MockProfileService::new());
    let runtime_mock: Arc<dyn keyrx_core::services::RuntimeServiceTrait> =
        Arc::new(MockRuntimeService::new());

    let api = ApiContext::new(device_mock, profile_mock, runtime_mock);

    // Should compile and work correctly
    let result = api.list_devices().await.unwrap();
    assert!(result.is_empty());
}
