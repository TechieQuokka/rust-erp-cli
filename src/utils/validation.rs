use crate::utils::error::{ErpError, ErpResult};
use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashSet;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^[\+]?[1-9][\d]{0,15}$").unwrap();
    static ref SKU_REGEX: Regex = Regex::new(r"^[A-Z0-9\-]{3,20}$").unwrap();
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
            &format!(
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
            &format!("exceeds maximum length of {}", config.max_name_length),
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
            &format!(
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
            &format!("must be one of: {}", allowed_list),
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
                &format!("must be at least {} characters long", min),
            ));
        }
    }

    if let Some(max) = max_length {
        if length > max {
            return Err(ErpError::validation(
                field_name,
                &format!("must be no more than {} characters long", max),
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
        assert!(validate_phone("+12345678901234567890").is_err());
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
}
