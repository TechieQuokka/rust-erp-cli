use crate::utils::error::{ErpError, ErpResult};
use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashSet;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^(\+?[\d\s\-\(\)\.]{7,20}|[\+]?[1-9][\d]{0,15})$").unwrap();
    static ref SKU_REGEX: Regex = Regex::new(r"^[A-Z0-9\-]{3,20}$").unwrap();
    static ref CONFIG_KEY_REGEX: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9._-]*$").unwrap();
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub min_password_length: usize,
    pub max_name_length: usize,
    pub max_description_length: usize,
    pub require_password_special_chars: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_password_length: 8,
            max_name_length: 100,
            max_description_length: 1000,
            require_password_special_chars: true,
        }
    }
}

pub fn validate_email(email: &str) -> ErpResult<()> {
    if email.trim().is_empty() {
        return Err(ErpError::validation("email", "cannot be empty"));
    }

    if email.len() > 254 {
        return Err(ErpError::validation(
            "email",
            "exceeds maximum length of 254",
        ));
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err(ErpError::validation("email", "invalid format"));
    }

    Ok(())
}

pub fn validate_phone(phone: &str) -> ErpResult<()> {
    if phone.trim().is_empty() {
        return Ok(());
    }

    let cleaned = phone.replace(&[' ', '-', '(', ')', '.'][..], "");

    if !PHONE_REGEX.is_match(&cleaned) {
        return Err(ErpError::validation("phone", "invalid format"));
    }

    Ok(())
}

pub fn validate_password(password: &str, config: &ValidationConfig) -> ErpResult<()> {
    if password.len() < config.min_password_length {
        return Err(ErpError::validation(
            "password",
            format!(
                "must be at least {} characters long",
                config.min_password_length
            ),
        ));
    }

    if password.len() > 128 {
        return Err(ErpError::validation(
            "password",
            "exceeds maximum length of 128",
        ));
    }

    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_uppercase {
        return Err(ErpError::validation(
            "password",
            "must contain at least one uppercase letter",
        ));
    }

    if !has_lowercase {
        return Err(ErpError::validation(
            "password",
            "must contain at least one lowercase letter",
        ));
    }

    if !has_digit {
        return Err(ErpError::validation(
            "password",
            "must contain at least one digit",
        ));
    }

    if config.require_password_special_chars {
        let has_special = password.chars().any(|c| !c.is_alphanumeric());
        if !has_special {
            return Err(ErpError::validation(
                "password",
                "must contain at least one special character",
            ));
        }
    }

    Ok(())
}

pub fn validate_name(name: &str, field_name: &str, config: &ValidationConfig) -> ErpResult<()> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(ErpError::validation(field_name, "cannot be empty"));
    }

    if trimmed.len() > config.max_name_length {
        return Err(ErpError::validation(
            field_name,
            format!("exceeds maximum length of {}", config.max_name_length),
        ));
    }

    if trimmed.chars().all(|c| c.is_whitespace()) {
        return Err(ErpError::validation(
            field_name,
            "cannot contain only whitespace",
        ));
    }

    Ok(())
}

pub fn validate_description(description: &str, config: &ValidationConfig) -> ErpResult<()> {
    if description.len() > config.max_description_length {
        return Err(ErpError::validation(
            "description",
            format!(
                "exceeds maximum length of {}",
                config.max_description_length
            ),
        ));
    }

    Ok(())
}

pub fn validate_sku(sku: &str) -> ErpResult<()> {
    if sku.trim().is_empty() {
        return Err(ErpError::validation("sku", "cannot be empty"));
    }

    let trimmed = sku.trim().to_uppercase();

    if !SKU_REGEX.is_match(&trimmed) {
        return Err(ErpError::validation(
            "sku",
            "must be 3-20 characters, containing only letters, numbers, and hyphens",
        ));
    }

    Ok(())
}

pub fn validate_quantity(quantity: i32) -> ErpResult<()> {
    if quantity < 0 {
        return Err(ErpError::validation("quantity", "cannot be negative"));
    }

    if quantity > 1_000_000 {
        return Err(ErpError::validation(
            "quantity",
            "exceeds maximum allowed value",
        ));
    }

    Ok(())
}

pub fn validate_price(price: Decimal) -> ErpResult<()> {
    if price.is_sign_negative() {
        return Err(ErpError::validation("price", "cannot be negative"));
    }

    if price > Decimal::from(999_999_999) {
        return Err(ErpError::validation(
            "price",
            "exceeds maximum allowed value",
        ));
    }

    if price.scale() > 2 {
        return Err(ErpError::validation(
            "price",
            "cannot have more than 2 decimal places",
        ));
    }

    Ok(())
}

pub fn validate_uuid(uuid_str: &str, field_name: &str) -> ErpResult<Uuid> {
    Uuid::parse_str(uuid_str).map_err(|_| ErpError::validation(field_name, "invalid UUID format"))
}

pub fn is_valid_config_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 100 && CONFIG_KEY_REGEX.is_match(key)
}

pub fn validate_config_key(key: &str) -> ErpResult<()> {
    if key.trim().is_empty() {
        return Err(ErpError::validation("key", "cannot be empty"));
    }

    if key.len() > 100 {
        return Err(ErpError::validation(
            "key",
            "exceeds maximum length of 100 characters",
        ));
    }

    if !CONFIG_KEY_REGEX.is_match(key) {
        return Err(ErpError::validation(
            "key",
            "must start with a letter and contain only letters, numbers, dots, underscores, and hyphens",
        ));
    }

    Ok(())
}

pub fn validate_enum_value<T>(value: &str, allowed_values: &[T], field_name: &str) -> ErpResult<()>
where
    T: AsRef<str>,
{
    if allowed_values.iter().any(|v| v.as_ref() == value) {
        Ok(())
    } else {
        let allowed_list = allowed_values
            .iter()
            .map(|v| v.as_ref())
            .collect::<Vec<_>>()
            .join(", ");

        Err(ErpError::validation(
            field_name,
            format!("must be one of: {}", allowed_list),
        ))
    }
}

pub fn validate_unique_values<T>(values: &[T], field_name: &str) -> ErpResult<()>
where
    T: std::hash::Hash + Eq + Clone,
{
    let unique_values: HashSet<_> = values.iter().cloned().collect();

    if unique_values.len() != values.len() {
        return Err(ErpError::validation(
            field_name,
            "contains duplicate values",
        ));
    }

    Ok(())
}

pub fn validate_string_length(
    value: &str,
    field_name: &str,
    min_length: Option<usize>,
    max_length: Option<usize>,
) -> ErpResult<()> {
    let length = value.len();

    if let Some(min) = min_length {
        if length < min {
            return Err(ErpError::validation(
                field_name,
                format!("must be at least {} characters long", min),
            ));
        }
    }

    if let Some(max) = max_length {
        if length > max {
            return Err(ErpError::validation(
                field_name,
                format!("must be no more than {} characters long", max),
            ));
        }
    }

    Ok(())
}

pub fn validate_required_field<T>(value: &Option<T>, field_name: &str) -> ErpResult<()> {
    if value.is_none() {
        return Err(ErpError::validation(field_name, "is required"));
    }

    Ok(())
}

pub fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect()
}

#[derive(Debug, Clone)]
pub struct ValidationService {
    config: ValidationConfig,
}

impl ValidationService {
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn validate_email(&self, email: &str) -> ErpResult<()> {
        validate_email(email)
    }

    pub fn validate_phone(&self, phone: &str) -> ErpResult<()> {
        validate_phone(phone)
    }

    pub fn validate_password(&self, password: &str) -> ErpResult<()> {
        validate_password(password, &self.config)
    }

    pub fn validate_username(&self, username: &str) -> ErpResult<()> {
        validate_name(username, "username", &self.config)
    }

    pub fn validate_name(&self, name: &str, field_name: &str) -> ErpResult<()> {
        validate_name(name, field_name, &self.config)
    }

    pub fn validate_description(&self, description: &str) -> ErpResult<()> {
        validate_description(description, &self.config)
    }

    pub fn validate_sku(&self, sku: &str) -> ErpResult<()> {
        validate_sku(sku)
    }

    pub fn validate_quantity(&self, quantity: i32) -> ErpResult<()> {
        validate_quantity(quantity)
    }

    pub fn validate_price(&self, price: Decimal) -> ErpResult<()> {
        validate_price(price)
    }

    pub fn validate_uuid(&self, uuid_str: &str, field_name: &str) -> ErpResult<Uuid> {
        validate_uuid(uuid_str, field_name)
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        sanitize_input(input)
    }
}

impl Default for ValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name@domain.co.uk").is_ok());

        assert!(validate_email("").is_err());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@domain.com").is_err());
    }

    #[test]
    fn test_validate_phone() {
        assert!(validate_phone("+1234567890").is_ok());
        assert!(validate_phone("123-456-7890").is_ok());
        assert!(validate_phone("").is_ok()); // Empty is allowed

        assert!(validate_phone("abc123").is_err());
        assert!(validate_phone("+123456789012345678901").is_err()); // 21 digits exceeds max of 20
    }

    #[test]
    fn test_validate_password() {
        let config = ValidationConfig::default();

        assert!(validate_password("Password123!", &config).is_ok());
        assert!(validate_password("short", &config).is_err());
        assert!(validate_password("nouppercase123!", &config).is_err());
        assert!(validate_password("NOLOWERCASE123!", &config).is_err());
        assert!(validate_password("NoDigits!", &config).is_err());
    }

    #[test]
    fn test_validate_sku() {
        assert!(validate_sku("PROD-001").is_ok());
        assert!(validate_sku("ABC123").is_ok());

        assert!(validate_sku("").is_err());
        assert!(validate_sku("ab").is_err()); // Too short
        assert!(validate_sku("prod_001").is_err()); // Invalid characters
    }

    #[test]
    fn test_validate_price() {
        assert!(validate_price(Decimal::new(1999, 2)).is_ok()); // 19.99
        assert!(validate_price(Decimal::new(0, 0)).is_ok()); // 0.00

        assert!(validate_price(Decimal::new(-100, 2)).is_err()); // -1.00
        assert!(validate_price(Decimal::new(1999, 3)).is_err()); // Too many decimals
    }

    #[test]
    fn test_validate_quantity() {
        assert!(validate_quantity(0).is_ok());
        assert!(validate_quantity(100).is_ok());
        assert!(validate_quantity(1_000_000).is_ok());

        assert!(validate_quantity(-1).is_err());
        assert!(validate_quantity(1_000_001).is_err());
    }

    #[test]
    fn test_validate_name() {
        let config = ValidationConfig::default();

        assert!(validate_name("Valid Name", "name", &config).is_ok());
        assert!(validate_name("John Doe", "name", &config).is_ok());

        assert!(validate_name("", "name", &config).is_err());
        assert!(validate_name("   ", "name", &config).is_err());
        assert!(validate_name(&"a".repeat(101), "name", &config).is_err());
    }

    #[test]
    fn test_validate_description() {
        let config = ValidationConfig::default();

        assert!(validate_description("Valid description", &config).is_ok());
        assert!(validate_description("", &config).is_ok()); // Empty is allowed

        assert!(validate_description(&"a".repeat(1001), &config).is_err());
    }

    #[test]
    fn test_validate_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let invalid_uuid = "invalid-uuid";

        assert!(validate_uuid(valid_uuid, "id").is_ok());
        assert!(validate_uuid(invalid_uuid, "id").is_err());
    }

    #[test]
    fn test_validate_config_key() {
        assert!(validate_config_key("valid_key").is_ok());
        assert!(validate_config_key("config.setting").is_ok());
        assert!(validate_config_key("app-setting").is_ok());

        assert!(validate_config_key("").is_err());
        assert!(validate_config_key("1invalid").is_err()); // Starts with number
        assert!(validate_config_key("invalid@key").is_err()); // Invalid character
        assert!(validate_config_key(&"a".repeat(101)).is_err()); // Too long
    }

    #[test]
    fn test_validate_enum_value() {
        let allowed_values = ["active", "inactive", "pending"];

        assert!(validate_enum_value("active", &allowed_values, "status").is_ok());
        assert!(validate_enum_value("invalid", &allowed_values, "status").is_err());
    }

    #[test]
    fn test_validate_unique_values() {
        let unique_values = vec![1, 2, 3, 4];
        let duplicate_values = vec![1, 2, 2, 3];

        assert!(validate_unique_values(&unique_values, "list").is_ok());
        assert!(validate_unique_values(&duplicate_values, "list").is_err());
    }

    #[test]
    fn test_validate_string_length() {
        assert!(validate_string_length("hello", "field", Some(3), Some(10)).is_ok());
        assert!(validate_string_length("hi", "field", Some(3), None).is_err()); // Too short
        assert!(validate_string_length("very long text", "field", None, Some(10)).is_err());
        // Too long
    }

    #[test]
    fn test_validate_required_field() {
        let some_value = Some("value");
        let none_value: Option<String> = None;

        assert!(validate_required_field(&some_value, "field").is_ok());
        assert!(validate_required_field(&none_value, "field").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("  hello world  "), "hello world");
        assert_eq!(sanitize_input("hello\nworld\t"), "hello\nworld"); // Tab is trimmed but newline is preserved
        assert_eq!(sanitize_input("hello\x00world"), "helloworld"); // Control character removed
        assert_eq!(sanitize_input("test\r\nline"), "test\r\nline"); // Both CR and LF are preserved as whitespace
    }

    #[test]
    fn test_is_valid_config_key() {
        assert!(is_valid_config_key("valid_key"));
        assert!(is_valid_config_key("config.setting"));
        assert!(!is_valid_config_key(""));
        assert!(!is_valid_config_key("1invalid"));
    }

    #[test]
    fn test_validation_service() {
        let service = ValidationService::new();

        assert!(service.validate_email("test@example.com").is_ok());
        assert!(service.validate_password("Password123!").is_ok());
        assert!(service.validate_sku("PROD-001").is_ok());
        assert!(service.validate_quantity(100).is_ok());

        let custom_config = ValidationConfig {
            min_password_length: 6,
            require_password_special_chars: false,
            ..Default::default()
        };
        let custom_service = ValidationService::with_config(custom_config);

        assert!(custom_service.validate_password("Pass123").is_ok());
        assert_eq!(custom_service.sanitize_input("  hello  "), "hello");
    }
}
