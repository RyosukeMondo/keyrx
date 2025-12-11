#[cfg(test)]
mod tests {
    use crate::registry::DeviceBindings;
    use crate::services::traits::DeviceServiceTrait;
    use crate::services::DeviceService;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_device_service_lifecycle() {
        let dir = tempdir().unwrap();
        let bindings_path = dir.path().join("bindings.json");

        // Initialize service (offline mode, no registry)
        let bindings = DeviceBindings::with_path(bindings_path.clone());
        let service = DeviceService::new(None, bindings);

        let device_key = "1234:5678:SERIAL";

        // 1. Get non-existent device
        let view = service.get_device(device_key).await.expect("get_device");
        assert_eq!(view.key, device_key);
        assert!(!view.connected);
        assert!(view.profile_id.is_none());

        // 2. Assign profile
        let view = service
            .assign_profile(device_key, "profile-1")
            .await
            .expect("assign");
        assert_eq!(view.profile_id.as_deref(), Some("profile-1"));

        // Verify persistence
        let mut bindings = DeviceBindings::with_path(bindings_path.clone());
        bindings.load().unwrap();
        // Identity reconstruction from key is needed to verify persistence details if we care,
        // but service.get_device returning correct data implies persistence worked (or cache).
        // Let's create a new service instance to verify persistence

        let bindings2 = DeviceBindings::with_path(bindings_path.clone());
        let service2 = DeviceService::new(None, bindings2);
        let view2 = service2.get_device(device_key).await.expect("get_device 2");
        assert_eq!(view2.profile_id.as_deref(), Some("profile-1"));

        // 3. Remap
        let view = service
            .set_remap_enabled(device_key, false)
            .await
            .expect("remap");
        assert!(!view.remap_enabled);

        // 4. Label
        let view = service
            .set_label(device_key, Some("My Keeb".to_string()))
            .await
            .expect("label");
        assert_eq!(view.label.as_deref(), Some("My Keeb"));

        // 5. Unassign
        let view = service
            .unassign_profile(device_key)
            .await
            .expect("unassign");
        assert!(view.profile_id.is_none());
    }
}
