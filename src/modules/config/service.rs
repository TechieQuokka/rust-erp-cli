// Config Module Service - Business Logic Layer

use crate::modules::config::models::*;
use crate::modules::config::repository::ConfigRepositoryTrait;
use crate::utils::error::{ErpError, ErpResult};
use crate::utils::validation;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Config Service - Business Logic for Configuration Management
pub struct ConfigService {
    repository: Arc<dyn ConfigRepositoryTrait>,
}

impl ConfigService {
    pub fn new(repository: Arc<dyn ConfigRepositoryTrait>) -> Self {
        Self { repository }
    }

    /// 새로운 설정 생성
    pub async fn create_config(&self, request: CreateConfigRequest) -> ErpResult<ConfigItem> {
        // 입력 검증
        self.validate_config_request(&request)?;

        // 키 중복 확인
        if self.repository.key_exists(&request.key).await? {
            return Err(ErpError::validation(
                "key",
                &format!("Configuration key '{}' already exists", request.key),
            ));
        }

        // 설정 생성
        let config = self.repository.create(&request).await?;

        info!(
            "Created configuration: key={}, category={}",
            config.key, config.category
        );

        Ok(config)
    }

    /// 설정 조회 (키로)
    pub async fn get_config(&self, key: &str) -> ErpResult<Option<ConfigItem>> {
        if key.trim().is_empty() {
            return Err(ErpError::validation("key", "Configuration key cannot be empty"));
        }

        self.repository.get_by_key(key.trim()).await
    }

    /// 설정 조회 (ID로)
    pub async fn get_config_by_id(&self, id: &Uuid) -> ErpResult<Option<ConfigItem>> {
        self.repository.get_by_id(id).await
    }

    /// 설정값만 조회 (비즈니스 로직에서 사용)
    pub async fn get_config_value(&self, key: &str) -> ErpResult<Option<String>> {
        match self.get_config(key).await? {
            Some(config) => Ok(Some(config.value)),
            None => Ok(None),
        }
    }

    /// 설정 목록 조회
    pub async fn list_configs(&self, filter: Option<ConfigFilter>) -> ErpResult<Vec<ConfigItem>> {
        match filter {
            Some(f) => self.repository.get_by_filter(&f).await,
            None => self.repository.get_all().await,
        }
    }

    /// 설정 목록 조회 (표시용 - 비밀 설정은 마스킹)
    pub async fn list_configs_for_display(&self, filter: Option<ConfigFilter>) -> ErpResult<Vec<ConfigItem>> {
        let mut configs = self.list_configs(filter).await?;

        // 비밀 설정의 값을 마스킹
        for config in &mut configs {
            if config.is_secret {
                config.value = config.masked_value();
            }
        }

        Ok(configs)
    }

    /// 설정 업데이트
    pub async fn update_config(&self, key: &str, request: UpdateConfigRequest) -> ErpResult<ConfigItem> {
        if key.trim().is_empty() {
            return Err(ErpError::validation("key", "Configuration key cannot be empty"));
        }

        // 기존 설정 조회
        let existing = self.get_config(key).await?
            .ok_or_else(|| ErpError::not_found("ConfigItem", key))?;

        // 입력 검증
        self.validate_update_request(&request)?;

        // 업데이트 실행
        let updated = self.repository.update(&existing.id, &request).await?;

        info!(
            "Updated configuration: key={}, changes={:?}",
            key, request
        );

        Ok(updated)
    }

    /// 설정 삭제
    pub async fn delete_config(&self, key: &str) -> ErpResult<()> {
        if key.trim().is_empty() {
            return Err(ErpError::validation("key", "Configuration key cannot be empty"));
        }

        // 설정 존재 확인
        let config = self.get_config(key).await?
            .ok_or_else(|| ErpError::not_found("ConfigItem", key))?;

        // 읽기 전용 설정 삭제 방지
        if config.is_readonly {
            return Err(ErpError::forbidden(
                "Cannot delete readonly configuration"
            ));
        }

        // 삭제 실행
        self.repository.delete_by_key(key).await?;

        warn!("Deleted configuration: key={}", key);

        Ok(())
    }

    /// 설정 초기화 (전체 삭제)
    pub async fn reset_configs(&self, force: bool) -> ErpResult<u32> {
        if !force {
            return Err(ErpError::validation(
                "force",
                "Configuration reset requires force flag"
            ));
        }

        // 모든 설정 조회
        let configs = self.repository.get_all().await?;
        let mut deleted_count = 0u32;

        // 읽기 전용이 아닌 설정만 삭제
        for config in configs {
            if !config.is_readonly {
                self.repository.delete(&config.id).await?;
                deleted_count += 1;
            }
        }

        warn!("Reset configurations: deleted {} items", deleted_count);

        Ok(deleted_count)
    }

    /// 카테고리별 설정 조회
    pub async fn get_configs_by_category(&self, category: &str) -> ErpResult<Vec<ConfigItem>> {
        let filter = ConfigFilter::new().with_category(category.to_string());
        self.repository.get_by_filter(&filter).await
    }

    /// 사용 가능한 카테고리 목록 조회
    pub async fn get_categories(&self) -> ErpResult<Vec<String>> {
        self.repository.get_categories().await
    }

    /// 설정 검색
    pub async fn search_configs(&self, pattern: &str) -> ErpResult<Vec<ConfigItem>> {
        if pattern.trim().is_empty() {
            return Err(ErpError::validation("pattern", "Search pattern cannot be empty"));
        }

        let filter = ConfigFilter::new().with_key_pattern(pattern.to_string());
        self.repository.get_by_filter(&filter).await
    }

    /// 설정 통계 조회
    pub async fn get_config_statistics(&self) -> ErpResult<ConfigStatistics> {
        let all_configs = self.repository.get_all().await?;
        let categories = self.repository.get_categories().await?;

        let mut category_counts: HashMap<String, u32> = HashMap::new();
        let mut secret_count = 0u32;
        let mut readonly_count = 0u32;

        for config in &all_configs {
            *category_counts.entry(config.category.clone()).or_insert(0) += 1;

            if config.is_secret {
                secret_count += 1;
            }

            if config.is_readonly {
                readonly_count += 1;
            }
        }

        Ok(ConfigStatistics {
            total_count: all_configs.len() as u32,
            category_count: categories.len() as u32,
            secret_count,
            readonly_count,
            category_distribution: category_counts,
        })
    }

    /// 설정 백업 (JSON 형태로 내보내기)
    pub async fn export_configs(&self, include_secrets: bool) -> ErpResult<String> {
        let filter = if include_secrets {
            Some(ConfigFilter::new().include_secrets())
        } else {
            None
        };

        let configs = self.list_configs(filter).await?;

        serde_json::to_string_pretty(&configs)
            .map_err(|e| ErpError::internal(&format!("Failed to export configs: {}", e)))
    }

    /// 설정 복원 (JSON에서 가져오기)
    pub async fn import_configs(&self, json_data: &str, overwrite: bool) -> ErpResult<ImportResult> {
        let configs: Vec<ConfigItem> = serde_json::from_str(json_data)
            .map_err(|e| ErpError::validation("json_data", &format!("Invalid JSON: {}", e)))?;

        let mut result = ImportResult {
            imported_count: 0,
            skipped_count: 0,
            error_count: 0,
            errors: Vec::new(),
        };

        for config in configs {
            let config_key = config.key.clone();
            match self.import_single_config(config, overwrite).await {
                Ok(imported) => {
                    if imported {
                        result.imported_count += 1;
                    } else {
                        result.skipped_count += 1;
                    }
                }
                Err(e) => {
                    result.error_count += 1;
                    result.errors.push(format!("Key '{}': {}", config_key, e));
                }
            }
        }

        info!(
            "Import completed: imported={}, skipped={}, errors={}",
            result.imported_count, result.skipped_count, result.error_count
        );

        Ok(result)
    }

    /// 단일 설정 가져오기
    async fn import_single_config(&self, config: ConfigItem, overwrite: bool) -> ErpResult<bool> {
        let exists = self.repository.key_exists(&config.key).await?;

        if exists && !overwrite {
            return Ok(false); // 스킵됨
        }

        let request = CreateConfigRequest {
            key: config.key.clone(),
            value: config.value,
            description: config.description,
            category: config.category,
            is_secret: config.is_secret,
            is_readonly: config.is_readonly,
        };

        if exists {
            // 기존 설정 업데이트
            let update_request = UpdateConfigRequest {
                value: Some(request.value),
                description: request.description,
                category: Some(request.category),
                is_secret: Some(request.is_secret),
            };
            self.update_config(&request.key, update_request).await?;
        } else {
            // 새 설정 생성
            self.repository.create(&request).await?;
        }

        Ok(true) // 가져오기 성공
    }

    /// 설정 요청 검증
    fn validate_config_request(&self, request: &CreateConfigRequest) -> ErpResult<()> {
        let mut validation = ConfigValidation::valid();

        // 키 검증
        if request.key.trim().is_empty() {
            validation.add_error("Key cannot be empty".to_string());
        } else if !validation::is_valid_config_key(&request.key) {
            validation.add_error("Key contains invalid characters".to_string());
        }

        // 값 검증
        if request.value.trim().is_empty() {
            validation.add_error("Value cannot be empty".to_string());
        }

        // 카테고리 검증
        if request.category.trim().is_empty() {
            validation.add_error("Category cannot be empty".to_string());
        }

        if !validation.is_valid {
            return Err(ErpError::validation("request", &validation.errors.join(", ")));
        }

        Ok(())
    }

    /// 업데이트 요청 검증
    fn validate_update_request(&self, request: &UpdateConfigRequest) -> ErpResult<()> {
        let mut validation = ConfigValidation::valid();

        // 값 검증 (값이 제공된 경우)
        if let Some(value) = &request.value {
            if value.trim().is_empty() {
                validation.add_error("Value cannot be empty".to_string());
            }
        }

        // 카테고리 검증 (카테고리가 제공된 경우)
        if let Some(category) = &request.category {
            if category.trim().is_empty() {
                validation.add_error("Category cannot be empty".to_string());
            }
        }

        if !validation.is_valid {
            return Err(ErpError::validation("request", &validation.errors.join(", ")));
        }

        Ok(())
    }
}

/// 설정 통계
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigStatistics {
    pub total_count: u32,
    pub category_count: u32,
    pub secret_count: u32,
    pub readonly_count: u32,
    pub category_distribution: HashMap<String, u32>,
}

/// 가져오기 결과
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub imported_count: u32,
    pub skipped_count: u32,
    pub error_count: u32,
    pub errors: Vec<String>,
}

// Tests moved to separate test module for compilation