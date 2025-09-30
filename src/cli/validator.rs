use crate::utils::error::{ErpError, ErpResult};
use crate::utils::validation::validate_email;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

pub struct CliValidator;

impl CliValidator {
    pub fn validate_price(price: f64) -> ErpResult<Decimal> {
        if price < 0.0 {
            return Err(ErpError::validation(
                "price",
                "가격은 음수일 수 없습니다. 0보다 큰 값을 입력하세요",
            ));
        }

        if price == 0.0 {
            return Err(ErpError::validation("price", "가격은 0보다 커야 합니다"));
        }

        if price > 99999999999999.99 {
            return Err(ErpError::validation(
                "price",
                "가격이 너무 큽니다 (최대: 99,999,999,999,999.99)",
            ));
        }

        Decimal::from_str(&price.to_string())
            .map_err(|_| ErpError::validation("price", "올바르지 않은 가격 형식입니다"))
    }

    pub fn validate_quantity(quantity: i32) -> ErpResult<i32> {
        if quantity <= 0 {
            return Err(ErpError::validation(
                "quantity",
                "수량은 최소 1 이상이어야 합니다",
            ));
        }

        // i32 타입 자체가 최대값을 보장하므로 상한 검증 불필요
        // DB의 INTEGER 타입과 일치 (최대: 2,147,483,647)

        Ok(quantity)
    }

    pub fn validate_product_name(name: &str) -> ErpResult<String> {
        if name.trim().is_empty() {
            return Err(ErpError::validation(
                "name",
                "제품명은 비어있을 수 없습니다",
            ));
        }

        if name.len() > 255 {
            return Err(ErpError::validation(
                "name",
                "제품명이 너무 깁니다 (최대: 255자)",
            ));
        }

        Ok(name.trim().to_string())
    }

    pub fn validate_sku(sku: &str) -> ErpResult<String> {
        if sku.trim().is_empty() {
            return Err(ErpError::validation("sku", "SKU는 비어있을 수 없습니다"));
        }

        if sku.len() > 50 {
            return Err(ErpError::validation(
                "sku",
                "SKU가 너무 깁니다 (최대: 50자)",
            ));
        }

        // SKU는 영문, 숫자, 하이픈, 언더스코어만 허용
        if !sku
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ErpError::validation(
                "sku",
                "SKU는 영문, 숫자, 하이픈(-), 언더스코어(_)만 허용됩니다",
            ));
        }

        Ok(sku.trim().to_uppercase())
    }

    pub fn validate_category(category: &str) -> ErpResult<String> {
        if category.trim().is_empty() {
            return Err(ErpError::validation(
                "category",
                "카테고리는 비어있을 수 없습니다",
            ));
        }

        if category.len() > 100 {
            return Err(ErpError::validation(
                "category",
                "카테고리명이 너무 깁니다 (최대: 100자)",
            ));
        }

        Ok(category.trim().to_string())
    }

    pub fn validate_customer_name(name: &str) -> ErpResult<String> {
        if name.trim().is_empty() {
            return Err(ErpError::validation(
                "name",
                "고객명은 비어있을 수 없습니다",
            ));
        }

        if name.len() > 255 {
            return Err(ErpError::validation(
                "name",
                "고객명이 너무 깁니다 (최대: 255자)",
            ));
        }

        Ok(name.trim().to_string())
    }

    pub fn validate_email_optional(email: &Option<String>) -> ErpResult<Option<String>> {
        match email {
            Some(email_str) => {
                if email_str.trim().is_empty() {
                    Ok(None)
                } else {
                    validate_email(email_str)?;
                    Ok(Some(email_str.trim().to_lowercase()))
                }
            }
            None => Ok(None),
        }
    }

    pub fn validate_phone_optional(phone: &Option<String>) -> ErpResult<Option<String>> {
        match phone {
            Some(phone_str) => {
                if phone_str.trim().is_empty() {
                    return Ok(None);
                }

                let cleaned = phone_str
                    .chars()
                    .filter(|c| {
                        c.is_ascii_digit()
                            || *c == '-'
                            || *c == '+'
                            || *c == '('
                            || *c == ')'
                            || *c == ' '
                    })
                    .collect::<String>()
                    .trim()
                    .to_string();

                if cleaned.len() < 10 || cleaned.len() > 20 {
                    return Err(ErpError::validation(
                        "phone",
                        "전화번호 길이가 올바르지 않습니다 (10-20자)",
                    ));
                }

                Ok(Some(cleaned))
            }
            None => Ok(None),
        }
    }

    pub fn validate_customer_type_optional(
        customer_type: &Option<String>,
    ) -> ErpResult<Option<String>> {
        match customer_type {
            Some(type_str) => {
                let valid_types = ["individual", "business"];
                let normalized = type_str.trim().to_lowercase();

                if !valid_types.contains(&normalized.as_str()) {
                    return Err(ErpError::validation(
                        "customer_type",
                        "고객 타입은 'individual' 또는 'business'만 허용됩니다",
                    ));
                }

                Ok(Some(normalized))
            }
            None => Ok(None),
        }
    }

    pub fn validate_order_status(status: &str) -> ErpResult<String> {
        let valid_statuses = ["pending", "processing", "shipped", "delivered", "cancelled"];
        let normalized = status.trim().to_lowercase();

        if !valid_statuses.contains(&normalized.as_str()) {
            return Err(ErpError::validation("status", "주문 상태는 'pending', 'processing', 'shipped', 'delivered', 'cancelled' 중 하나여야 합니다"));
        }

        Ok(normalized)
    }

    pub fn validate_date_string(date_str: &str) -> ErpResult<NaiveDate> {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            ErpError::validation(
                "date",
                "날짜 형식이 올바르지 않습니다 (YYYY-MM-DD 형식을 사용해주세요)",
            )
        })
    }

    pub fn validate_date_range(
        from_date: &Option<String>,
        to_date: &Option<String>,
    ) -> ErpResult<(Option<NaiveDate>, Option<NaiveDate>)> {
        let from = match from_date {
            Some(date_str) => Some(Self::validate_date_string(date_str)?),
            None => None,
        };

        let to = match to_date {
            Some(date_str) => Some(Self::validate_date_string(date_str)?),
            None => None,
        };

        if let (Some(from_d), Some(to_d)) = (from, to) {
            if from_d > to_d {
                return Err(ErpError::validation(
                    "date_range",
                    "시작 날짜가 종료 날짜보다 늦을 수 없습니다",
                ));
            }

            if from_d > Utc::now().naive_utc().date() {
                return Err(ErpError::validation(
                    "date_range",
                    "시작 날짜가 미래일 수 없습니다",
                ));
            }
        }

        Ok((from, to))
    }

    pub fn validate_discount_percentage(discount: f64) -> ErpResult<f64> {
        if !(0.0..=100.0).contains(&discount) {
            return Err(ErpError::validation(
                "discount",
                "할인율은 0-100% 범위여야 합니다",
            ));
        }

        Ok(discount)
    }

    pub fn validate_report_format(format: &str) -> ErpResult<String> {
        let valid_formats = ["table", "csv", "json", "pdf", "html"];
        let normalized = format.trim().to_lowercase();

        if !valid_formats.contains(&normalized.as_str()) {
            return Err(ErpError::validation(
                "format",
                "출력 형식은 'table', 'csv', 'json', 'pdf', 'html' 중 하나여야 합니다",
            ));
        }

        Ok(normalized)
    }

    pub fn validate_report_period(period: &str) -> ErpResult<String> {
        let valid_periods = [
            "daily",
            "weekly",
            "monthly",
            "quarterly",
            "yearly",
            "custom",
        ];
        let normalized = period.trim().to_lowercase();

        if !valid_periods.contains(&normalized.as_str()) {
            return Err(ErpError::validation(
                "period",
                "기간은 'daily', 'weekly', 'monthly', 'quarterly', 'yearly', 'custom' 중 하나여야 합니다",
            ));
        }

        Ok(normalized)
    }

    pub fn validate_pagination(page: u32, limit: u32) -> ErpResult<(u32, u32)> {
        if page == 0 {
            return Err(ErpError::validation(
                "page",
                "페이지 번호는 1 이상이어야 합니다",
            ));
        }

        if limit == 0 || limit > 1000 {
            return Err(ErpError::validation(
                "limit",
                "페이지당 아이템 수는 1-1000 범위여야 합니다",
            ));
        }

        Ok((page, limit))
    }

    pub fn validate_order_items(items: &[String]) -> ErpResult<Vec<(String, i32)>> {
        if items.is_empty() {
            return Err(ErpError::validation(
                "items",
                "최소 하나의 주문 아이템이 필요합니다",
            ));
        }

        let mut validated_items = Vec::new();

        for item_str in items {
            let parts: Vec<&str> = item_str.split(':').collect();
            if parts.len() != 2 {
                return Err(ErpError::validation(
                    "items",
                    "아이템 형식이 올바르지 않습니다 (product_id:quantity)",
                ));
            }

            let product_id = parts[0].trim();
            if product_id.is_empty() {
                return Err(ErpError::validation(
                    "product_id",
                    "제품 ID는 비어있을 수 없습니다",
                ));
            }

            let quantity = parts[1]
                .trim()
                .parse::<i32>()
                .map_err(|_| ErpError::validation("quantity", "수량은 유효한 정수여야 합니다"))?;

            Self::validate_quantity(quantity)?;

            validated_items.push((product_id.to_string(), quantity));
        }

        Ok(validated_items)
    }

    pub fn validate_search_field(field: &str) -> ErpResult<String> {
        let valid_fields = ["name", "email", "phone", "all"];
        let normalized = field.trim().to_lowercase();

        if !valid_fields.contains(&normalized.as_str()) {
            return Err(ErpError::validation(
                "field",
                "검색 필드는 'name', 'email', 'phone', 'all' 중 하나여야 합니다",
            ));
        }

        Ok(normalized)
    }

    pub fn validate_id_or_sku(id: &str) -> ErpResult<String> {
        let cleaned = id.trim();
        if cleaned.is_empty() {
            return Err(ErpError::validation(
                "id",
                "ID 또는 SKU는 비어있을 수 없습니다",
            ));
        }

        // For Phase 3, just return the cleaned ID without complex validation
        // UUID and SKU validation will be implemented in Phase 4
        Ok(cleaned.to_string())
    }
}
