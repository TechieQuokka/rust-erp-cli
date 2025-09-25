use anyhow::{bail, Result};
use regex::Regex;
use std::str::FromStr;

#[derive(Default)]
pub struct InputValidator;

impl InputValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_email(email: &str) -> Result<()> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?;

        if email_regex.is_match(email) {
            Ok(())
        } else {
            bail!("Invalid email format: {}", email)
        }
    }

    pub fn validate_phone(phone: &str) -> Result<()> {
        let phone_regex = Regex::new(r"^[\+]?[0-9\s\-\(\)]{10,15}$")?;

        if phone_regex.is_match(phone) {
            Ok(())
        } else {
            bail!("Invalid phone number format: {}", phone)
        }
    }

    pub fn validate_price(price: f64) -> Result<()> {
        if price < 0.0 {
            bail!("Price cannot be negative: {}", price)
        }
        if price > 999999999.99 {
            bail!("Price too large: {}", price)
        }
        Ok(())
    }

    pub fn validate_quantity(quantity: i32) -> Result<()> {
        if quantity < 0 {
            bail!("Quantity cannot be negative: {}", quantity)
        }
        if quantity > 1_000_000 {
            bail!("Quantity too large: {}", quantity)
        }
        Ok(())
    }

    pub fn validate_uuid(uuid_str: &str) -> Result<uuid::Uuid> {
        match uuid::Uuid::from_str(uuid_str) {
            Ok(uuid) => Ok(uuid),
            Err(_) => bail!("Invalid UUID format: {}", uuid_str),
        }
    }

    pub fn validate_username(username: &str) -> Result<()> {
        if username.len() < 3 {
            bail!("Username must be at least 3 characters long");
        }
        if username.len() > 50 {
            bail!("Username cannot exceed 50 characters");
        }

        let username_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")?;
        if !username_regex.is_match(username) {
            bail!("Username can only contain letters, numbers, underscores, and hyphens");
        }

        Ok(())
    }

    pub fn validate_product_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            bail!("Product name cannot be empty");
        }
        if name.len() > 255 {
            bail!("Product name cannot exceed 255 characters");
        }
        Ok(())
    }

    pub fn validate_customer_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            bail!("Customer name cannot be empty");
        }
        if name.len() > 255 {
            bail!("Customer name cannot exceed 255 characters");
        }
        Ok(())
    }

    pub fn validate_category(category: &str) -> Result<()> {
        if category.trim().is_empty() {
            bail!("Category cannot be empty");
        }
        if category.len() > 100 {
            bail!("Category cannot exceed 100 characters");
        }
        Ok(())
    }

    pub fn validate_sku(sku: &str) -> Result<()> {
        if sku.trim().is_empty() {
            bail!("SKU cannot be empty");
        }
        if sku.len() > 100 {
            bail!("SKU cannot exceed 100 characters");
        }

        let sku_regex = Regex::new(r"^[A-Z0-9\-_]+$")?;
        if !sku_regex.is_match(sku) {
            bail!("SKU can only contain uppercase letters, numbers, hyphens, and underscores");
        }

        Ok(())
    }

    pub fn validate_order_status(status: &str) -> Result<()> {
        match status.to_lowercase().as_str() {
            "pending" | "processing" | "shipped" | "delivered" | "cancelled" => Ok(()),
            _ => bail!("Invalid order status. Valid statuses: pending, processing, shipped, delivered, cancelled"),
        }
    }

    pub fn validate_user_role(role: &str) -> Result<()> {
        match role.to_lowercase().as_str() {
            "admin" | "manager" | "user" | "readonly" => Ok(()),
            _ => bail!("Invalid user role. Valid roles: admin, manager, user, readonly"),
        }
    }

    pub fn validate_output_format(format: &str) -> Result<()> {
        match format.to_lowercase().as_str() {
            "table" | "json" | "csv" => Ok(()),
            _ => bail!("Invalid output format. Valid formats: table, json, csv"),
        }
    }

    pub fn validate_report_period(period: &str) -> Result<()> {
        match period.to_lowercase().as_str() {
            "daily" | "weekly" | "monthly" | "quarterly" | "yearly" => Ok(()),
            _ => bail!(
                "Invalid report period. Valid periods: daily, weekly, monthly, quarterly, yearly"
            ),
        }
    }

    pub fn validate_date_range(date_range: &str) -> Result<(chrono::NaiveDate, chrono::NaiveDate)> {
        let parts: Vec<&str> = date_range.split(',').collect();
        if parts.len() != 2 {
            bail!("Date range must be in format: YYYY-MM-DD,YYYY-MM-DD");
        }

        let start_date = chrono::NaiveDate::parse_from_str(parts[0].trim(), "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid start date format: {}", parts[0]))?;

        let end_date = chrono::NaiveDate::parse_from_str(parts[1].trim(), "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("Invalid end date format: {}", parts[1]))?;

        if start_date > end_date {
            bail!("Start date cannot be after end date");
        }

        Ok((start_date, end_date))
    }

    pub fn validate_pagination(page: usize, limit: usize) -> Result<()> {
        if page == 0 {
            bail!("Page number must be greater than 0");
        }
        if limit == 0 {
            bail!("Limit must be greater than 0");
        }
        if limit > 1000 {
            bail!("Limit cannot exceed 1000");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(InputValidator::validate_email("test@example.com").is_ok());
        assert!(InputValidator::validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_validate_phone() {
        assert!(InputValidator::validate_phone("123-456-7890").is_ok());
        assert!(InputValidator::validate_phone("invalid").is_err());
    }

    #[test]
    fn test_validate_price() {
        assert!(InputValidator::validate_price(19.99).is_ok());
        assert!(InputValidator::validate_price(-1.0).is_err());
    }

    #[test]
    fn test_validate_quantity() {
        assert!(InputValidator::validate_quantity(100).is_ok());
        assert!(InputValidator::validate_quantity(-1).is_err());
    }

    #[test]
    fn test_validate_username() {
        assert!(InputValidator::validate_username("user123").is_ok());
        assert!(InputValidator::validate_username("ab").is_err()); // too short
    }

    #[test]
    fn test_validate_order_status() {
        assert!(InputValidator::validate_order_status("pending").is_ok());
        assert!(InputValidator::validate_order_status("invalid").is_err());
    }
}
