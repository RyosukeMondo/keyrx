//! Integration tests for authentication system

#[cfg(test)]
mod tests {
    use crate::auth::*;

    #[test]
    fn test_full_authentication_flow() {
        // Initialize auth service
        std::env::set_var("KEYRX_JWT_SECRET", "test_secret_key_for_testing");
        std::env::set_var("KEYRX_ADMIN_PASSWORD", "SecureP@ssw0rd123");

        let service = AuthService::new();

        // Verify admin user was created
        assert!(service.get_user_password_hash("admin").is_some());

        // Test password verification
        let hasher = PasswordHasher::new();
        let stored_hash = service.get_user_password_hash("admin").unwrap();
        assert!(hasher
            .verify_password("SecureP@ssw0rd123", &stored_hash)
            .unwrap());

        // Test JWT generation
        let access_token = service.jwt_manager.generate_access_token("admin").unwrap();
        let refresh_token = service.jwt_manager.generate_refresh_token("admin").unwrap();

        // Test JWT validation
        let claims = service
            .jwt_manager
            .validate_access_token(&access_token)
            .unwrap();
        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.token_type, TokenType::Access);

        let refresh_claims = service
            .jwt_manager
            .validate_refresh_token(&refresh_token)
            .unwrap();
        assert_eq!(refresh_claims.sub, "admin");
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);

        std::env::remove_var("KEYRX_JWT_SECRET");
        std::env::remove_var("KEYRX_ADMIN_PASSWORD");
    }

    #[test]
    fn test_rate_limiting() {
        use std::net::{IpAddr, Ipv4Addr};

        let limiter = LoginRateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

        // First 5 attempts should succeed
        for i in 0..5 {
            assert!(
                limiter.check_rate_limit(ip).is_ok(),
                "Attempt {} failed",
                i + 1
            );
        }

        // 6th attempt should fail
        assert!(limiter.check_rate_limit(ip).is_err());

        // Reset should allow new attempts
        limiter.reset(ip);
        assert!(limiter.check_rate_limit(ip).is_ok());
    }

    #[test]
    fn test_password_complexity_validation() {
        let hasher = PasswordHasher::new();

        // Too short
        assert!(hasher.hash_password("Short1!").is_err());

        // No complexity
        assert!(hasher.hash_password("alllowercase").is_err());

        // Valid complex password
        assert!(hasher.hash_password("ValidP@ssw0rd123").is_ok());
        assert!(hasher.hash_password("AnotherG00d!Pass").is_ok());
    }

    #[test]
    fn test_token_expiration() {
        std::env::set_var("KEYRX_JWT_SECRET", "test_secret_key");
        let service = AuthService::new();

        // Generate a token
        let token = service
            .jwt_manager
            .generate_access_token("testuser")
            .unwrap();

        // Token should be valid immediately
        assert!(service.jwt_manager.validate_access_token(&token).is_ok());

        // Note: We cannot easily test expiration without mocking time
        // In production, tokens expire after 15 minutes

        std::env::remove_var("KEYRX_JWT_SECRET");
    }
}
