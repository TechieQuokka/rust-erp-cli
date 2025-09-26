//! 재고 관리 모듈
//!
//! 제품 등록, 수정, 삭제 및 재고 수준 관리를 담당합니다.
//!
//! ## 주요 기능
//!
//! - 제품 생성, 조회, 수정, 삭제
//! - 재고 수량 조정 및 이력 관리
//! - 저재고 알림 및 재주문 추천
//! - 재고 평가 및 통계
//! - 카테고리별 재고 관리
//!
//! ## 아키텍처
//!
//! - `models`: 데이터 모델 및 요청/응답 구조체
//! - `repository`: 데이터베이스 접근 계층 (PostgreSQL/SQLite)
//! - `service`: 비즈니스 로직 계층
//!
//! ## 사용 예시
//!
//! ```rust
//! use crate::modules::inventory::{
//!     service::{InventoryService, InventoryServiceImpl},
//!     repository::PostgresInventoryRepository,
//!     models::CreateInventoryItemRequest,
//! };
//!
//! // 서비스 초기화
//! let repository = Arc::new(PostgresInventoryRepository::new(pool));
//! let service = InventoryServiceImpl::new(repository);
//!
//! // 제품 생성
//! let request = CreateInventoryItemRequest {
//!     name: "새 제품".to_string(),
//!     category: "전자제품".to_string(),
//!     price: Decimal::new(29999, 2),
//!     quantity: 100,
//!     min_stock: 10,
//!     // ...
//! };
//!
//! let product = service.create_product(request, user_id).await?;
//! ```

pub mod models;
pub mod repository;
pub mod service;

// Re-export commonly used types for convenience
pub use models::{
    CategoryValuation, CreateInventoryItemRequest, InventoryFilter, InventoryItem,
    InventoryItemResponse, InventoryListResponse, InventoryValuation, LowStockAlert,
    StockAdjustmentRequest, StockMovementResponse, UpdateInventoryItemRequest,
};

pub use repository::{InventoryRepository, MockInventoryRepository, PostgresInventoryRepository};

pub use service::{InventoryService, InventoryServiceImpl};

use crate::core::database::connection::DatabasePool;
use crate::utils::error::ErpResult;
use std::sync::Arc;

/// 재고 관리 모듈 팩토리
///
/// 데이터베이스 연결을 받아서 완전히 구성된 인벤토리 서비스를 생성합니다.
pub struct InventoryModule {
    service: Arc<dyn InventoryService>,
}

impl InventoryModule {
    /// PostgreSQL을 사용하여 새로운 재고 관리 모듈을 생성합니다.
    pub fn new_with_postgres(pool: DatabasePool) -> Self {
        let repository = Arc::new(PostgresInventoryRepository::new(pool));
        let service = Arc::new(InventoryServiceImpl::new(repository));

        Self { service }
    }

    /// 테스트용 모의 구현체를 사용하여 새로운 재고 관리 모듈을 생성합니다.
    pub fn new_with_mock() -> Self {
        let repository = Arc::new(MockInventoryRepository::new());
        let service = Arc::new(InventoryServiceImpl::new(repository));

        Self { service }
    }

    /// 재고 관리 서비스에 대한 참조를 반환합니다.
    pub fn service(&self) -> Arc<dyn InventoryService> {
        Arc::clone(&self.service)
    }

    /// 모듈 상태를 확인합니다.
    ///
    /// 데이터베이스 연결 및 기본적인 서비스 가용성을 검증합니다.
    pub async fn health_check(&self) -> ErpResult<()> {
        // 기본적인 서비스 가용성 확인
        let _valuation = self.service.get_inventory_valuation().await?;
        Ok(())
    }

    /// 모듈 초기화 작업을 수행합니다.
    ///
    /// 필요한 데이터베이스 테이블 존재 여부 확인 등의 작업을 수행합니다.
    pub async fn initialize(&self) -> ErpResult<()> {
        // TODO: 필요한 초기화 작업 수행
        // - 데이터베이스 테이블 존재 확인
        // - 기본 카테고리 생성
        // - 인덱스 확인

        tracing::info!("Inventory module initialized successfully");
        Ok(())
    }

    /// 모듈 통계 정보를 반환합니다.
    pub async fn get_statistics(&self) -> ErpResult<InventoryModuleStats> {
        let valuation = self.service.get_inventory_valuation().await?;
        let _low_stock_alerts = self.service.get_low_stock_alerts(None).await?;
        let reorder_items = self.service.get_products_requiring_reorder().await?;

        Ok(InventoryModuleStats {
            total_products: valuation.total_items,
            total_value: valuation.total_sell_value,
            low_stock_count: valuation.low_stock_items,
            out_of_stock_count: valuation.out_of_stock_items,
            reorder_required_count: reorder_items.len() as i64,
            categories_count: valuation.by_category.len() as i64,
        })
    }
}

/// 재고 관리 모듈 통계 정보
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InventoryModuleStats {
    pub total_products: i64,
    pub total_value: rust_decimal::Decimal,
    pub low_stock_count: i64,
    pub out_of_stock_count: i64,
    pub reorder_required_count: i64,
    pub categories_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inventory_module_creation() {
        let module = InventoryModule::new_with_mock();
        let service = module.service();

        // Test that service is accessible
        let filter = models::InventoryFilter::default();
        let result = service.list_products(filter).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let module = InventoryModule::new_with_mock();
        let result = module.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_initialize() {
        let module = InventoryModule::new_with_mock();
        let result = module.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let module = InventoryModule::new_with_mock();
        let result = module.get_statistics().await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_products, 0); // Mock implementation returns 0
    }
}
