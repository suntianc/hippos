//! Security Module Tests
//!
//! Tests for authentication, authorization, rate limiting, and validation.

#[cfg(test)]
mod auth_tests {
    use super::super::*;
    use crate::security::auth::*;
    use crate::security::config::SecuritySettings;

    #[tokio::test]
    async fn test_api_key_authentication_valid_key() {
        let mut api_keys = std::collections::HashSet::new();
        api_keys.insert("test-api-key".to_string());

        let auth = ApiKeyAuth::new(api_keys);
        let credentials = Credentials::new(Some("test-api-key".to_string()), None);

        let result = auth.authenticate(&credentials).await;
        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.token_type, TokenType::ApiKey);
        assert!(token.tenant_id.is_some());
    }

    #[tokio::test]
    async fn test_api_key_authentication_invalid_key() {
        let mut api_keys = std::collections::HashSet::new();
        api_keys.insert("valid-key".to_string());

        let auth = ApiKeyAuth::new(api_keys);
        let credentials = Credentials::new(Some("invalid-key".to_string()), None);

        let result = auth.authenticate(&credentials).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_key_authentication_no_key() {
        let auth = ApiKeyAuth::development();
        let credentials = Credentials::new(None, None);

        let result = auth.authenticate(&credentials).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_jwt_authentication_valid_token() {
        let jwt_auth = JwtAuth::development();

        // First generate a token
        let generator = JwtTokenGenerator::new(
            "dev-secret-change-in-production-min-32-chars".to_string(),
            "hippos".to_string(),
            "hippos-api".to_string(),
            3600,
        );

        let token = generator
            .generate_token(
                "user123".to_string(),
                "tenant1".to_string(),
                "user".to_string(),
            )
            .unwrap();

        // Then authenticate with it
        let credentials = Credentials::new(None, Some(token.clone()));
        let result = jwt_auth.authenticate(&credentials).await;

        assert!(result.is_ok());
        let auth_token = result.unwrap();
        assert_eq!(auth_token.token_type, TokenType::Bearer);
    }

    #[tokio::test]
    async fn test_jwt_authentication_invalid_token() {
        let jwt_auth = JwtAuth::development();
        let credentials = Credentials::new(None, Some("invalid.jwt.token".to_string()));

        let result = jwt_auth.authenticate(&credentials).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_combined_authenticator_api_key_first() {
        let combined = CombinedAuthenticator::development();
        let credentials = Credentials::new(Some("dev-api-key".to_string()), None);

        let result = combined.authenticate(&credentials).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_combined_authenticator_jwt_fallback() {
        let combined = CombinedAuthenticator::development();

        // Generate a valid JWT
        let generator = JwtTokenGenerator::new(
            "dev-secret-change-in-production-min-32-chars".to_string(),
            "hippos".to_string(),
            "hippos-api".to_string(),
            3600,
        );

        let token = generator
            .generate_token(
                "user123".to_string(),
                "tenant1".to_string(),
                "user".to_string(),
            )
            .unwrap();

        let credentials = Credentials::new(None, Some(token));
        let result = combined.authenticate(&credentials).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_credentials_from_authorization_header() {
        // Test ApiKey prefix
        let creds = Credentials::from_authorization_header(Some("ApiKey test-key"));
        assert_eq!(creds.api_key, Some("test-key".to_string()));
        assert!(creds.jwt_token.is_none());

        // Test Bearer prefix
        let creds = Credentials::from_authorization_header(Some("Bearer test-token"));
        assert!(creds.api_key.is_none());
        assert_eq!(creds.jwt_token, Some("test-token".to_string()));

        // Test invalid header
        let creds = Credentials::from_authorization_header(Some("Basic base64"));
        assert!(creds.api_key.is_none());
        assert!(creds.jwt_token.is_none());
    }

    #[tokio::test]
    async fn test_claims_creation() {
        let claims = Claims::new(
            "user123".to_string(),
            "tenant1".to_string(),
            "admin".to_string(),
            3600,
            "hippos".to_string(),
            "hippos-api".to_string(),
        );

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.tenant_id, "tenant1");
        assert_eq!(claims.role, "admin");
        assert!(!claims.jti.is_empty());
        assert!(claims.exp > claims.iat);
    }
}

#[cfg(test)]
mod rbac_tests {
    use super::super::*;
    use crate::security::rbac::*;

    #[tokio::test]
    async fn test_role_from_string() {
        assert_eq!(Role::from_string("admin"), Role::Admin);
        assert_eq!(Role::from_string("ADMIN"), Role::Admin);
        assert_eq!(Role::from_string("tenant_admin"), Role::TenantAdmin);
        assert_eq!(Role::from_string("user"), Role::User);
        assert_eq!(Role::from_string("unknown"), Role::User);
    }

    #[tokio::test]
    async fn test_role_is_admin() {
        assert!(Role::Admin.is_admin());
        assert!(!Role::TenantAdmin.is_admin());
        assert!(!Role::User.is_admin());
        assert!(!Role::ReadOnly.is_admin());
    }

    #[tokio::test]
    async fn test_role_is_elevated() {
        assert!(Role::Admin.is_elevated());
        assert!(Role::TenantAdmin.is_elevated());
        assert!(!Role::User.is_elevated());
        assert!(!Role::ReadOnly.is_elevated());
    }

    #[tokio::test]
    async fn test_permission_matching() {
        let perm1 = Permission::new(ResourceType::Session, ActionType::Read);
        let perm2 = Permission::new(ResourceType::Session, ActionType::Read);
        let perm3 = Permission::new(ResourceType::Session, ActionType::Write);
        let perm_wildcard = Permission::new(ResourceType::All, ActionType::All);

        assert!(perm1.matches(&perm2));
        assert!(!perm1.matches(&perm3));
        assert!(perm_wildcard.matches(&perm1));
        assert!(perm1.matches(&perm_wildcard));
    }

    #[tokio::test]
    async fn test_default_permissions_admin() {
        let perms = get_default_permissions(&Role::Admin);
        assert_eq!(perms.len(), 1);
        assert_eq!(perms[0].resource, ResourceType::All);
        assert_eq!(perms[0].action, ActionType::All);
    }

    #[tokio::test]
    async fn test_default_permissions_user() {
        let perms = get_default_permissions(&Role::User);
        // User should have CRUD on sessions and turns, search on indexes
        let session_crud: Vec<_> = perms
            .iter()
            .filter(|p| p.resource == ResourceType::Session)
            .collect();
        let turn_crud: Vec<_> = perms
            .iter()
            .filter(|p| p.resource == ResourceType::Turn)
            .collect();

        assert!(session_crud.len() >= 3); // Create, Read, Update, Delete
        assert!(turn_crud.len() >= 3);
    }

    #[tokio::test]
    async fn test_simple_authorizer_admin_has_all_permissions() {
        let authorizer = SimpleAuthorizer::development();
        let claims = Claims::new(
            "admin1".to_string(),
            "tenant1".to_string(),
            "admin".to_string(),
            3600,
            "hippos".to_string(),
            "hippos-api".to_string(),
        );

        let permission = Permission::new(ResourceType::All, ActionType::All);
        let has_permission = authorizer.check_permission(&claims, &permission).await;

        assert!(has_permission);
    }

    #[tokio::test]
    async fn test_simple_authorizer_user_permissions() {
        let authorizer = SimpleAuthorizer::development();
        let claims = Claims::new(
            "user1".to_string(),
            "tenant1".to_string(),
            "user".to_string(),
            3600,
            "hippos".to_string(),
            "hippos-api".to_string(),
        );

        // User should be able to read sessions
        let read_session = Permission::new(ResourceType::Session, ActionType::Read);
        let can_read = authorizer.check_permission(&claims, &read_session).await;
        assert!(can_read);

        // User should NOT be able to manage system
        let manage_system = Permission::new(ResourceType::System, ActionType::Manage);
        let can_manage = authorizer.check_permission(&claims, &manage_system).await;
        assert!(!can_manage);
    }

    #[tokio::test]
    async fn test_claims_ext() {
        let claims = Claims::new(
            "user1".to_string(),
            "tenant1".to_string(),
            "admin".to_string(),
            3600,
            "hippos".to_string(),
            "hippos-api".to_string(),
        );

        assert_eq!(claims.tenant_id(), "tenant1");
        assert_eq!(claims.role(), Role::Admin);
        assert!(claims.is_admin());
        assert!(claims.can_access_tenant("tenant1"));
        assert!(!claims.can_access_tenant("tenant2"));
    }
}

#[cfg(test)]
mod rate_limit_tests {
    use super::super::*;
    use crate::security::rate_limit::*;

    #[tokio::test]
    async fn test_rate_limiter_allow_requests() {
        let limiter = RateLimiter::development();
        let client = RateLimitClient::from_ip("192.168.1.1");

        // First request should be allowed
        let result = limiter.check_rate_limit(&client).await;
        assert!(matches!(result, RateLimitResult::Allowed));
    }

    #[tokio::test]
    async fn test_rate_limiter_enabled_in_production() {
        let limiter = RateLimiter::production();
        let client = RateLimitClient::from_ip("192.168.1.1");

        // Should start with allowed
        let result = limiter.check_rate_limit(&client).await;
        assert!(matches!(result, RateLimitResult::Allowed));
    }

    #[tokio::test]
    async fn test_rate_limiter_disabled_in_development() {
        let limiter = RateLimiter::development();
        let client = RateLimitClient::from_ip("192.168.1.1");

        // Should always return Allowed when disabled
        let result1 = limiter.check_rate_limit(&client).await;
        let result2 = limiter.check_rate_limit(&client).await;
        let result3 = limiter.check_rate_limit(&client).await;

        assert!(matches!(result1, RateLimitResult::Allowed));
        assert!(matches!(result2, RateLimitResult::Allowed));
        assert!(matches!(result3, RateLimitResult::Allowed));
    }

    #[tokio::test]
    async fn test_rate_limiter_record_request() {
        let limiter = RateLimiter::production();
        let client = RateLimitClient::from_ip("192.168.1.1");

        // Record some requests
        limiter.record_request(&client).await;
        limiter.record_request(&client).await;

        // Should still be allowed (high limits)
        let result = limiter.check_rate_limit(&client).await;
        assert!(matches!(result, RateLimitResult::Allowed));
    }

    #[tokio::test]
    async fn test_rate_limiter_usage_stats() {
        let limiter = RateLimiter::development();
        let client = RateLimitClient::from_ip("192.168.1.1");

        // Record some requests
        limiter.record_request(&client).await;
        limiter.record_request(&client).await;

        let stats = limiter.get_usage_stats(&client).await;
        assert!(!stats.is_empty());
        assert!(stats[0].remaining <= stats[0].limit);
    }

    #[tokio::test]
    async fn test_rate_limiter_clear_client() {
        let limiter = RateLimiter::production();
        let client = RateLimitClient::from_ip("192.168.1.1");

        // Record requests
        limiter.record_request(&client).await;

        // Clear
        limiter.clear_client(&client).await;

        // Should still work
        let result = limiter.check_rate_limit(&client).await;
        assert!(matches!(result, RateLimitResult::Allowed));
    }

    #[tokio::test]
    async fn test_rate_limit_client_types() {
        let api_key_client = RateLimitClient::from_api_key("test-key");
        let ip_client = RateLimitClient::from_ip("192.168.1.1");
        let jwt_client = RateLimitClient::from_jwt_subject("user123");

        assert_eq!(api_key_client.as_str(), "test-key");
        assert_eq!(ip_client.as_str(), "192.168.1.1");
        assert_eq!(jwt_client.as_str(), "user123");
    }
}

#[cfg(test)]
mod validation_tests {
    use super::super::*;
    use crate::security::validation::*;

    #[tokio::test]
    async fn test_request_validator_length_valid() {
        let validator = RequestValidator::new();
        let result = validator.validate_length("name", "test", Some(1), Some(255));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_validator_length_too_short() {
        let validator = RequestValidator::new();
        let result = validator.validate_length("name", "", Some(1), Some(255));
        assert!(result.is_err());
        if let Err(ValidationError::TooShort { field, min, got }) = result {
            assert_eq!(field, "name");
            assert_eq!(min, 1);
            assert_eq!(got, 0);
        }
    }

    #[tokio::test]
    async fn test_request_validator_length_too_long() {
        let validator = RequestValidator::new();
        let long_string = "a".repeat(1000);
        let result = validator.validate_length("name", &long_string, Some(1), Some(255));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_validator_uuid_valid() {
        let validator = RequestValidator::new();
        let result = validator.validate_uuid("id", "550e8400-e29b-41d4-a716-446655440000");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_validator_uuid_invalid() {
        let validator = RequestValidator::new();
        let result = validator.validate_uuid("id", "not-a-uuid");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_validator_safe_chars() {
        let validator = RequestValidator::new();
        let result = validator.validate_safe_chars(
            "name",
            "valid-name_123",
            &['a'..='z', 'A'..='Z', '0'..='9', '-', '_'],
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_validator_invalid_chars() {
        let validator = RequestValidator::new();
        let result = validator.validate_safe_chars(
            "name",
            "invalid<script>",
            &['a'..='z', 'A'..='Z', '0'..='9'],
        );
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sanitize_string() {
        let input = "  test\x00string\t\n  ";
        let result = RequestValidator::sanitize_string(input);
        assert_eq!(result, "test string");
    }

    #[tokio::test]
    async fn test_sanitize_for_html() {
        let input = "<script>alert('xss')</script>";
        let result = RequestValidator::sanitize_for_html(&input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
    }

    #[tokio::test]
    async fn test_sanitize_for_sql() {
        let input = "test'; DROP TABLE users;--";
        let result = RequestValidator::sanitize_for_sql(&input);
        assert!(!result.contains("DROP TABLE"));
    }

    #[tokio::test]
    async fn test_validate_content_type_valid() {
        let validator = RequestValidator::new();
        let result = validator.validate_content_type(Some("application/json"));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_content_type_invalid() {
        let validator = RequestValidator::new();
        let result = validator.validate_content_type(Some("text/html"));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_body_size_valid() {
        let validator = RequestValidator::new();
        let result = validator.validate_body_size(1024);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_body_size_too_large() {
        let validator = RequestValidator::new();
        let result = validator.validate_body_size(20 * 1024 * 1024); // 20MB
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validators_session_name() {
        let result = validators::validate_session_name("Valid Session Name");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validators_pagination_valid() {
        let result = validators::validate_pagination(Some(50), Some(100));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validators_pagination_invalid_limit() {
        let result = validators::validate_pagination(Some(0), None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validators_pagination_limit_too_high() {
        let result = validators::validate_pagination(Some(2000), None);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod security_settings_tests {
    use super::super::*;
    use crate::security::config::SecuritySettings;

    #[tokio::test]
    async fn test_development_settings() {
        let settings = SecuritySettings::development();

        assert!(settings.has_jwt_secret());
        assert!(!settings.api_keys.is_empty());
        assert!(!settings.rate_limit_enabled);
        assert!(settings.api_key_auth_enabled);
        assert!(settings.jwt_auth_enabled);
    }

    #[tokio::test]
    async fn test_production_settings() {
        let settings = SecuritySettings::production();

        assert!(settings.rate_limit_enabled);
        assert!(settings.security_headers_enabled);
    }

    #[tokio::test]
    async fn test_security_settings_default() {
        let settings = SecuritySettings::default();

        assert!(settings.jwt_secret.is_empty());
        assert!(settings.api_keys.is_empty());
        assert!(!settings.rate_limit_enabled);
    }
}
