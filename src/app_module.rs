//! # Application DI Container Module
//!
//! Shaku 기반 의존성 주입 컨테이너를 정의합니다.
//! 모든 Repository와 Service를 Components로 등록하고 관리합니다.

use shaku::{Component, Interface};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::core::database::connection::DatabaseManager;
use crate::utils::error::ErpResult;
use crate::utils::validation::ValidationService;

/// DatabasePool을 Component로 래핑
///
/// Shaku는 동기 생성만 지원하므로, 미리 초기화된 Pool을 받습니다.
#[derive(Component)]
#[shaku(interface = DatabasePoolProvider)]
pub struct DatabasePoolProviderImpl {
    pool: Arc<Pool<Postgres>>,
}

pub trait DatabasePoolProvider: Interface {
    fn get_pool(&self) -> Arc<Pool<Postgres>>;
}

impl DatabasePoolProvider for DatabasePoolProviderImpl {
    fn get_pool(&self) -> Arc<Pool<Postgres>> {
        self.pool.clone()
    }
}

/// ValidationService Component
#[derive(Component)]
#[shaku(interface = ValidationServiceProvider)]
pub struct ValidationServiceProviderImpl;

pub trait ValidationServiceProvider: Interface {
    fn get_service(&self) -> ValidationService;
}

impl ValidationServiceProvider for ValidationServiceProviderImpl {
    fn get_service(&self) -> ValidationService {
        ValidationService::new()
    }
}

// Application Module 정의는 나중에 각 Repository/Service를
// Component로 변환한 후 추가할 예정

/// AppModule Builder를 위한 헬퍼
///
/// async 초기화가 필요한 DatabasePool을 처리합니다.
pub struct AppModuleBuilder;

impl AppModuleBuilder {
    /// DatabaseManager에서 연결을 가져와 Module을 빌드합니다.
    pub async fn build() -> ErpResult<AppModuleContainer> {
        let connection = DatabaseManager::get_connection().await?;
        let pool = Arc::new(connection.pool().clone());

        // 임시: Module은 나중에 정의
        Ok(AppModuleContainer { pool })
    }
}

/// 임시 Container 구조체 (Module 정의 전까지 사용)
pub struct AppModuleContainer {
    pool: Arc<Pool<Postgres>>,
}

impl AppModuleContainer {
    pub fn get_pool(&self) -> Arc<Pool<Postgres>> {
        self.pool.clone()
    }
}
