//! Request Validation Module
//!
//! Provides request validation and input sanitization for security.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Validation error types
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("Required field '{field}' is missing")]
    MissingField { field: String },

    #[error("Field '{field}' is too long (max: {max}, got: {got})")]
    TooLong {
        field: String,
        max: usize,
        got: usize,
    },

    #[error("Field '{field}' is too short (min: {min}, got: {got})")]
    TooShort {
        field: String,
        min: usize,
        got: usize,
    },

    #[error("Field '{field}' contains invalid characters: {chars}")]
    InvalidCharacters { field: String, chars: String },

    #[error("Field '{field}' is not a valid UUID: {value}")]
    InvalidUuid { field: String, value: String },

    #[error("Field '{field}' is not a valid email: {value}")]
    InvalidEmail { field: String, value: String },

    #[error("Field '{field}' must be a positive number")]
    NotPositive { field: String },

    #[error("Field '{field}' exceeds maximum value: max={max}, got={got}")]
    ExceedsMax { field: String, max: i64, got: i64 },

    #[error("Field '{field}' is below minimum value: min={min}, got={got}")]
    BelowMin { field: String, min: i64, got: i64 },

    #[error("Invalid request body format")]
    InvalidBody,

    #[error("Request body too large: max={max} bytes, got={got} bytes")]
    BodyTooLarge { max: usize, got: usize },

    #[error("Invalid content type: expected={expected}, got={got}")]
    InvalidContentType { expected: String, got: String },

    #[error("Custom validation failed: {message}")]
    Custom { field: String, message: String },
}

impl ValidationError {
    pub fn field(&self) -> &str {
        match self {
            Self::MissingField { field } => field.as_str(),
            Self::TooLong { field, .. } => field.as_str(),
            Self::TooShort { field, .. } => field.as_str(),
            Self::InvalidCharacters { field, .. } => field.as_str(),
            Self::InvalidUuid { field, .. } => field.as_str(),
            Self::InvalidEmail { field, .. } => field.as_str(),
            Self::NotPositive { field } => field.as_str(),
            Self::ExceedsMax { field, .. } => field.as_str(),
            Self::BelowMin { field, .. } => field.as_str(),
            Self::InvalidBody => "body",
            Self::BodyTooLarge { .. } => "body",
            Self::InvalidContentType { .. } => "content_type",
            Self::Custom { field, .. } => field.as_str(),
        }
    }
}

/// Validation result type
pub type ValidationResult<T> = std::result::Result<T, ValidationError>;

/// Request validation trait
#[async_trait]
pub trait Validatable: Send + Sync {
    /// Validate the request data
    fn validate(&self) -> ValidationResult<()>;
}

/// Request sanitizer trait
#[async_trait]
pub trait Sanitizable: Send + Sync {
    /// Sanitize the request data
    fn sanitize(&mut self);
}

/// Request validator implementation
#[derive(Debug, Clone)]
pub struct RequestValidator {
    /// Maximum allowed field length
    max_field_length: usize,
    /// Maximum request body size
    max_body_size: usize,
    /// Allowed content types
    allowed_content_types: Vec<String>,
    /// Whether validation is enabled
    enabled: bool,
}

impl Default for RequestValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self {
            max_field_length: 10_000,
            max_body_size: 10 * 1024 * 1024, // 10MB
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/json; charset=utf-8".to_string(),
            ],
            enabled: true,
        }
    }

    /// Create development validator (permissive)
    pub fn development() -> Self {
        let mut validator = Self::new();
        validator.enabled = false; // Disabled in dev for easier testing
        validator
    }

    /// Create production validator (strict)
    pub fn production() -> Self {
        Self::new()
    }

    /// Set maximum field length
    pub fn with_max_field_length(mut self, length: usize) -> Self {
        self.max_field_length = length;
        self
    }

    /// Set maximum body size
    pub fn with_max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    /// Validate field length
    pub fn validate_length(
        &self,
        field: &str,
        value: &str,
        min: Option<usize>,
        max: Option<usize>,
    ) -> ValidationResult<()> {
        let length = value.chars().count();

        if let Some(min_len) = min {
            if length < min_len {
                return Err(ValidationError::TooShort {
                    field: field.to_string(),
                    min: min_len,
                    got: length,
                });
            }
        }

        if let Some(max_len) = max {
            if length > max_len {
                return Err(ValidationError::TooLong {
                    field: field.to_string(),
                    max: max_len,
                    got: length,
                });
            }
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(&self, field: &str, value: &str) -> ValidationResult<()> {
        match uuid::Uuid::parse_str(value) {
            Ok(_) => Ok(()),
            Err(_) => Err(ValidationError::InvalidUuid {
                field: field.to_string(),
                value: value.to_string(),
            }),
        }
    }

    /// Validate email format
    pub fn validate_email(&self, field: &str, value: &str) -> ValidationResult<()> {
        let email_regex =
            regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if email_regex.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::InvalidEmail {
                field: field.to_string(),
                value: value.to_string(),
            })
        }
    }

    /// Validate that value contains only safe characters
    pub fn validate_safe_chars(
        &self,
        field: &str,
        value: &str,
        allowed: &[char],
    ) -> ValidationResult<()> {
        let invalid: String = value.chars().filter(|c| !allowed.contains(c)).collect();

        if !invalid.is_empty() {
            Err(ValidationError::InvalidCharacters {
                field: field.to_string(),
                chars: invalid,
            })
        } else {
            Ok(())
        }
    }

    /// Sanitize string input
    pub fn sanitize_string(input: &str) -> String {
        // Remove null bytes and control characters
        input
            .trim()
            .chars()
            .filter(|c| !c.is_ascii_control() || c.is_whitespace())
            .collect()
    }

    /// Sanitize for HTML (prevent XSS)
    pub fn sanitize_for_html(input: &str) -> String {
        Self::sanitize_string(input)
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
            .replace("/", "&#x2F;")
    }

    /// Sanitize for SQL (basic protection - use parameterized queries in practice)
    pub fn sanitize_for_sql(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        for c in input.chars() {
            match c {
                '\'' => result.push_str("''"),
                ';' => result.push(' '),
                '-' => result.push_str("--"),
                _ => result.push(c),
            }
        }
        result
    }

    /// Check content type is allowed
    pub fn validate_content_type(&self, content_type: Option<&str>) -> ValidationResult<()> {
        let ct = content_type.ok_or_else(|| ValidationError::InvalidContentType {
            expected: self.allowed_content_types.join(", "),
            got: "none".to_string(),
        })?;

        let ct_base = ct.split(';').next().unwrap_or(ct).trim().to_lowercase();

        if !self.allowed_content_types.iter().any(|allowed| {
            let allowed_base = allowed.split(';').next().unwrap_or(allowed).trim();
            ct_base == allowed_base.to_lowercase()
        }) {
            return Err(ValidationError::InvalidContentType {
                expected: self.allowed_content_types.join(", "),
                got: ct.to_string(),
            });
        }

        Ok(())
    }

    /// Check body size
    pub fn validate_body_size(&self, size: usize) -> ValidationResult<()> {
        if size > self.max_body_size {
            return Err(ValidationError::BodyTooLarge {
                max: self.max_body_size,
                got: size,
            });
        }
        Ok(())
    }
}

/// Generic validated request wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedRequest<T: Validatable + Sanitizable> {
    /// The inner request data
    pub inner: T,
}

impl<T: Validatable + Sanitizable> ValidatedRequest<T> {
    /// Create new validated request
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Validate and create
    pub fn validate(inner: T) -> ValidationResult<Self> {
        inner.validate()?;
        Ok(Self { inner })
    }

    /// Sanitize and validate
    pub fn sanitize_validate(mut inner: T) -> ValidationResult<Self> {
        inner.sanitize();
        inner.validate()?;
        Ok(Self { inner })
    }
}

impl<T: Validatable + Sanitizable> Validatable for ValidatedRequest<T> {
    fn validate(&self) -> ValidationResult<()> {
        self.inner.validate()
    }
}

impl<T: Validatable + Sanitizable> Sanitizable for ValidatedRequest<T> {
    fn sanitize(&mut self) {
        self.inner.sanitize();
    }
}

/// Common validation helpers
pub mod validators {
    use super::*;

    /// Validate session name
    pub fn validate_session_name(name: &str) -> ValidationResult<()> {
        let validator = RequestValidator::new();
        validator.validate_length("name", name, Some(1), Some(255))?;
        validator.validate_safe_chars("name", name, &[' ', '-', '_', '.', ':', '(', ')'])
    }

    /// Validate turn content
    pub fn validate_turn_content(content: &str) -> ValidationResult<()> {
        let validator = RequestValidator::new();
        validator.validate_length("content", content, Some(1), Some(100_000))
    }

    /// Validate search query
    pub fn validate_search_query(query: &str) -> ValidationResult<()> {
        let validator = RequestValidator::new();
        validator.validate_length("query", query, Some(1), Some(1000))
    }

    /// Validate pagination parameters
    pub fn validate_pagination(limit: Option<u32>, offset: Option<u32>) -> ValidationResult<()> {
        if let Some(l) = limit {
            if l == 0 || l > 1000 {
                return Err(ValidationError::Custom {
                    field: "limit".to_string(),
                    message: "Limit must be between 1 and 1000".to_string(),
                });
            }
        }
        if let Some(o) = offset {
            if o > 100_000 {
                return Err(ValidationError::Custom {
                    field: "offset".to_string(),
                    message: "Offset cannot exceed 100000".to_string(),
                });
            }
        }
        Ok(())
    }
}
