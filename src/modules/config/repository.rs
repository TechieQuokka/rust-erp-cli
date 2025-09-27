// Config Module Repository - Data Access Layer

use crate::core::database::DatabaseConnection;
use crate::modules::config::models::*;
use crate::utils::error::{ErpError, ErpResult};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, Row};
use std::sync::Arc;
use uuid::Uuid;

/// Config Repository Trait
#[async_trait]
pub trait ConfigRepositoryTrait: Send + Sync {
    async fn create(&self, config: &CreateConfigRequest) -> ErpResult<ConfigItem>;
    async fn get_by_key(&self, key: &str) -> ErpResult<Option<ConfigItem>>;
    async fn get_by_id(&self, id: &Uuid) -> ErpResult<Option<ConfigItem>>;
    async fn get_by_filter(&self, filter: &ConfigFilter) -> ErpResult<Vec<ConfigItem>>;
    async fn get_all(&self) -> ErpResult<Vec<ConfigItem>>;
    async fn update(&self, id: &Uuid, update_data: &UpdateConfigRequest) -> ErpResult<ConfigItem>;
    async fn delete(&self, id: &Uuid) -> ErpResult<()>;
    async fn delete_by_key(&self, key: &str) -> ErpResult<()>;
    async fn key_exists(&self, key: &str) -> ErpResult<bool>;
    async fn get_categories(&self) -> ErpResult<Vec<String>>;
}

/// Config Repository Implementation
pub struct ConfigRepository {
    db: Arc<DatabaseConnection>,
}

impl ConfigRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// 데이터베이스 테이블 초기화
    pub async fn init_table(&self) -> ErpResult<()> {
        let pool = self.db.pool();

        // PostgreSQL 호환 CREATE TABLE 문
        let create_table_query = r#"
            CREATE TABLE IF NOT EXISTS config_items (
                id UUID PRIMARY KEY,
                key VARCHAR(255) UNIQUE NOT NULL,
                value TEXT NOT NULL,
                description TEXT,
                category VARCHAR(100) NOT NULL DEFAULT 'general',
                is_secret BOOLEAN NOT NULL DEFAULT FALSE,
                is_readonly BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
        "#;

        sqlx::query(create_table_query)
            .execute(pool)
            .await
            .map_err(|e| {
                ErpError::database(format!("Failed to create config_items table: {}", e))
            })?;

        // 인덱스 생성
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_config_key ON config_items(key)")
            .execute(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to create key index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_config_category ON config_items(category)")
            .execute(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to create category index: {}", e)))?;

        Ok(())
    }

    /// SQL 쿼리에서 ConfigItem 생성
    fn row_to_config_item(&self, row: &PgRow) -> ErpResult<ConfigItem> {
        let id: Uuid = row
            .try_get("id")
            .map_err(|e| ErpError::database(format!("Failed to get id from row: {}", e)))?;

        let created_at: chrono::DateTime<chrono::Utc> = row
            .try_get("created_at")
            .map_err(|e| ErpError::database(format!("Failed to get created_at from row: {}", e)))?;

        let updated_at: chrono::DateTime<chrono::Utc> = row
            .try_get("updated_at")
            .map_err(|e| ErpError::database(format!("Failed to get updated_at from row: {}", e)))?;

        Ok(ConfigItem {
            id,
            key: row
                .try_get("key")
                .map_err(|e| ErpError::database(format!("Failed to get key from row: {}", e)))?,
            value: row
                .try_get("value")
                .map_err(|e| ErpError::database(format!("Failed to get value from row: {}", e)))?,
            description: row.try_get("description").unwrap_or(None),
            category: row.try_get("category").map_err(|e| {
                ErpError::database(format!("Failed to get category from row: {}", e))
            })?,
            is_secret: row.try_get("is_secret").unwrap_or(false),
            is_readonly: row.try_get("is_readonly").unwrap_or(false),
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl ConfigRepositoryTrait for ConfigRepository {
    async fn create(&self, config: &CreateConfigRequest) -> ErpResult<ConfigItem> {
        let pool = self.db.pool();
        let config_item = ConfigItem::new(
            config.key.clone(),
            config.value.clone(),
            config.category.clone(),
            config.description.clone(),
            config.is_secret,
            config.is_readonly,
        );

        let query = r#"
            INSERT INTO config_items (id, key, value, description, category, is_secret, is_readonly, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        sqlx::query(query)
            .bind(config_item.id)
            .bind(&config_item.key)
            .bind(&config_item.value)
            .bind(&config_item.description)
            .bind(&config_item.category)
            .bind(config_item.is_secret)
            .bind(config_item.is_readonly)
            .bind(config_item.created_at)
            .bind(config_item.updated_at)
            .execute(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to create config item: {}", e)))?;

        Ok(config_item)
    }

    async fn get_by_key(&self, key: &str) -> ErpResult<Option<ConfigItem>> {
        let pool = self.db.pool();

        let query = "SELECT * FROM config_items WHERE key = $1";

        match sqlx::query(query).bind(key).fetch_one(pool).await {
            Ok(row) => Ok(Some(self.row_to_config_item(&row)?)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(ErpError::database(format!(
                "Failed to get config by key: {}",
                e
            ))),
        }
    }

    async fn get_by_id(&self, id: &Uuid) -> ErpResult<Option<ConfigItem>> {
        let pool = self.db.pool();

        let query = "SELECT * FROM config_items WHERE id = $1";

        match sqlx::query(query).bind(id).fetch_one(pool).await {
            Ok(row) => Ok(Some(self.row_to_config_item(&row)?)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(ErpError::database(format!(
                "Failed to get config by id: {}",
                e
            ))),
        }
    }

    async fn get_by_filter(&self, filter: &ConfigFilter) -> ErpResult<Vec<ConfigItem>> {
        let pool = self.db.pool();
        let mut query_str = "SELECT * FROM config_items WHERE 1=1".to_string();
        let mut bind_params: Vec<String> = Vec::new();

        if let Some(category) = &filter.category {
            query_str.push_str(" AND category = ?");
            bind_params.push(category.clone());
        }

        if let Some(pattern) = &filter.key_pattern {
            query_str.push_str(" AND key LIKE ?");
            bind_params.push(format!("%{}%", pattern));
        }

        if !filter.include_secrets {
            query_str.push_str(" AND is_secret = FALSE");
        }

        if filter.readonly_only {
            query_str.push_str(" AND is_readonly = TRUE");
        }

        query_str.push_str(" ORDER BY category, key");

        let mut query = sqlx::query(&query_str);
        for param in bind_params {
            query = query.bind(param);
        }

        let rows = query
            .fetch_all(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to get configs by filter: {}", e)))?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(self.row_to_config_item(&row)?);
        }

        Ok(configs)
    }

    async fn get_all(&self) -> ErpResult<Vec<ConfigItem>> {
        let pool = self.db.pool();

        let query = "SELECT * FROM config_items ORDER BY category, key";
        let rows = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to get all configs: {}", e)))?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(self.row_to_config_item(&row)?);
        }

        Ok(configs)
    }

    async fn update(&self, id: &Uuid, update_data: &UpdateConfigRequest) -> ErpResult<ConfigItem> {
        let pool = self.db.pool();

        // 현재 설정 조회
        let current = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found("ConfigItem", id.to_string()))?;

        // 읽기 전용 설정 확인
        if current.is_readonly {
            return Err(ErpError::forbidden("Cannot update readonly configuration"));
        }

        let mut query_parts = Vec::new();
        let mut bind_params: Vec<String> = Vec::new();

        if let Some(value) = &update_data.value {
            query_parts.push("value = ?");
            bind_params.push(value.clone());
        }

        if let Some(description) = &update_data.description {
            query_parts.push("description = ?");
            bind_params.push(description.clone());
        }

        if let Some(category) = &update_data.category {
            query_parts.push("category = ?");
            bind_params.push(category.clone());
        }

        if let Some(is_secret) = update_data.is_secret {
            query_parts.push("is_secret = ?");
            bind_params.push(is_secret.to_string());
        }

        if query_parts.is_empty() {
            return Ok(current); // 변경사항이 없으면 현재 값 반환
        }

        query_parts.push("updated_at = ?");
        bind_params.push(chrono::Utc::now().to_rfc3339());

        let query_str = format!(
            "UPDATE config_items SET {} WHERE id = ?",
            query_parts.join(", ")
        );

        let mut query = sqlx::query(&query_str);
        for param in bind_params {
            query = query.bind(param);
        }
        query = query.bind(id.to_string());

        query
            .execute(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to update config: {}", e)))?;

        // 업데이트된 설정 조회 후 반환
        self.get_by_id(id)
            .await?
            .ok_or_else(|| ErpError::internal("Updated config not found"))
    }

    async fn delete(&self, id: &Uuid) -> ErpResult<()> {
        let pool = self.db.pool();

        // 설정이 존재하는지 먼저 확인
        let config = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found("ConfigItem", id.to_string()))?;

        // 읽기 전용 설정은 삭제 불가
        if config.is_readonly {
            return Err(ErpError::forbidden("Cannot delete readonly configuration"));
        }

        let query = "DELETE FROM config_items WHERE id = $1";
        sqlx::query(query)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to delete config: {}", e)))?;

        Ok(())
    }

    async fn delete_by_key(&self, key: &str) -> ErpResult<()> {
        let pool = self.db.pool();

        // 설정이 존재하는지 먼저 확인
        let config = self
            .get_by_key(key)
            .await?
            .ok_or_else(|| ErpError::not_found("ConfigItem", key.to_string()))?;

        // 읽기 전용 설정은 삭제 불가
        if config.is_readonly {
            return Err(ErpError::forbidden("Cannot delete readonly configuration"));
        }

        let query = "DELETE FROM config_items WHERE key = $1";
        sqlx::query(query)
            .bind(key)
            .execute(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to delete config by key: {}", e)))?;

        Ok(())
    }

    async fn key_exists(&self, key: &str) -> ErpResult<bool> {
        let pool = self.db.pool();

        let query = "SELECT COUNT(*) as count FROM config_items WHERE key = $1";
        let row = sqlx::query(query)
            .bind(key)
            .fetch_one(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to check key existence: {}", e)))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| ErpError::database(format!("Failed to get count: {}", e)))?;

        Ok(count > 0)
    }

    async fn get_categories(&self) -> ErpResult<Vec<String>> {
        let pool = self.db.pool();

        let query = "SELECT DISTINCT category FROM config_items ORDER BY category";
        let rows = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to get categories: {}", e)))?;

        let mut categories = Vec::new();
        for row in rows {
            let category: String = row
                .try_get("category")
                .map_err(|e| ErpError::database(format!("Failed to get category: {}", e)))?;
            categories.push(category);
        }

        Ok(categories)
    }
}

// Tests moved to separate test module for compilation
