// Config Module Models - Data Structures for Configuration Management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn display_option(opt: &Option<String>) -> String {
    opt.as_ref().map(|s| s.as_str()).unwrap_or("").to_string()
}

/// 설정 항목 모델
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(sqlx::FromRow)]
#[derive(tabled::Tabled)]
pub struct ConfigItem {
    pub id: Uuid,
    pub key: String,
    pub value: String,
    #[tabled(display_with = "display_option")]
    pub description: Option<String>,
    pub category: String,
    pub is_secret: bool,
    pub is_readonly: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConfigItem {
    pub fn new(
        key: String,
        value: String,
        category: String,
        description: Option<String>,
        is_secret: bool,
        is_readonly: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            key,
            value,
            description,
            category,
            is_secret,
            is_readonly,
            created_at: now,
            updated_at: now,
        }
    }

    /// 설정값이 민감한 정보인지 확인
    pub fn is_sensitive(&self) -> bool {
        self.is_secret
    }

    /// 설정값이 읽기 전용인지 확인
    pub fn is_read_only(&self) -> bool {
        self.is_readonly
    }

    /// 마스킹된 값 반환 (비밀 설정인 경우)
    pub fn masked_value(&self) -> String {
        if self.is_secret {
            "*".repeat(8)
        } else {
            self.value.clone()
        }
    }
}

/// 설정 생성을 위한 데이터 구조
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConfigRequest {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub category: String,
    pub is_secret: bool,
    pub is_readonly: bool,
}

impl CreateConfigRequest {
    pub fn new(key: String, value: String, category: String) -> Self {
        Self {
            key,
            value,
            category,
            description: None,
            is_secret: false,
            is_readonly: false,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn as_secret(mut self) -> Self {
        self.is_secret = true;
        self
    }

    pub fn as_readonly(mut self) -> Self {
        self.is_readonly = true;
        self
    }
}

/// 설정 업데이트를 위한 데이터 구조
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_secret: Option<bool>,
}

/// 설정 검색 필터
#[derive(Debug, Clone, Default)]
pub struct ConfigFilter {
    pub category: Option<String>,
    pub key_pattern: Option<String>,
    pub include_secrets: bool,
    pub readonly_only: bool,
}

impl ConfigFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_key_pattern(mut self, pattern: String) -> Self {
        self.key_pattern = Some(pattern);
        self
    }

    pub fn include_secrets(mut self) -> Self {
        self.include_secrets = true;
        self
    }

    pub fn readonly_only(mut self) -> Self {
        self.readonly_only = true;
        self
    }
}

/// 설정 카테고리
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigCategory {
    Database,
    Logging,
    Authentication,
    Cache,
    Api,
    Reports,
    System,
    Custom(String),
}

impl ConfigCategory {
    pub fn as_str(&self) -> &str {
        match self {
            ConfigCategory::Database => "database",
            ConfigCategory::Logging => "logging",
            ConfigCategory::Authentication => "authentication",
            ConfigCategory::Cache => "cache",
            ConfigCategory::Api => "api",
            ConfigCategory::Reports => "reports",
            ConfigCategory::System => "system",
            ConfigCategory::Custom(s) => s,
        }
    }
}

impl From<&str> for ConfigCategory {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "database" => ConfigCategory::Database,
            "logging" => ConfigCategory::Logging,
            "authentication" | "auth" => ConfigCategory::Authentication,
            "cache" => ConfigCategory::Cache,
            "api" => ConfigCategory::Api,
            "reports" => ConfigCategory::Reports,
            "system" => ConfigCategory::System,
            _ => ConfigCategory::Custom(s.to_string()),
        }
    }
}

impl From<String> for ConfigCategory {
    fn from(s: String) -> Self {
        ConfigCategory::from(s.as_str())
    }
}

/// 설정 검증 결과
#[derive(Debug, Clone)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ConfigValidation {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_item_creation() {
        let config = ConfigItem::new(
            "test.key".to_string(),
            "test_value".to_string(),
            "test".to_string(),
            Some("Test configuration".to_string()),
            false,
            false,
        );

        assert_eq!(config.key, "test.key");
        assert_eq!(config.value, "test_value");
        assert_eq!(config.category, "test");
        assert_eq!(config.description, Some("Test configuration".to_string()));
        assert!(!config.is_secret);
        assert!(!config.is_readonly);
    }

    #[test]
    fn test_config_item_masked_value() {
        let secret_config = ConfigItem::new(
            "secret.key".to_string(),
            "secret_value".to_string(),
            "auth".to_string(),
            None,
            true,
            false,
        );

        let normal_config = ConfigItem::new(
            "normal.key".to_string(),
            "normal_value".to_string(),
            "system".to_string(),
            None,
            false,
            false,
        );

        assert_eq!(secret_config.masked_value(), "********");
        assert_eq!(normal_config.masked_value(), "normal_value");
    }

    #[test]
    fn test_create_config_request_builder() {
        let request = CreateConfigRequest::new(
            "test.key".to_string(),
            "test_value".to_string(),
            "test".to_string(),
        )
        .with_description("Test description".to_string())
        .as_secret()
        .as_readonly();

        assert_eq!(request.key, "test.key");
        assert_eq!(request.description, Some("Test description".to_string()));
        assert!(request.is_secret);
        assert!(request.is_readonly);
    }

    #[test]
    fn test_config_category_conversion() {
        assert_eq!(ConfigCategory::from("database"), ConfigCategory::Database);
        assert_eq!(ConfigCategory::from("auth"), ConfigCategory::Authentication);
        assert_eq!(ConfigCategory::from("custom"), ConfigCategory::Custom("custom".to_string()));
    }
}