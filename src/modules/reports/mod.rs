pub mod models;
pub mod repository;
pub mod service;

pub use models::*;
pub use repository::{MockReportsRepository, ReportsRepository};
pub use service::ReportsService;

use sqlx::PgPool;
use std::sync::Arc;

/// 보고서 모듈 초기화 및 종속성 설정
pub fn create_reports_service(pool: Option<PgPool>) -> ReportsService {
    match pool {
        Some(_pg_pool) => {
            // Production would use PostgresReportsRepository
            let repository = Arc::new(MockReportsRepository::new());
            ReportsService::new(repository)
        }
        _ => {
            // 개발/테스트 환경에서는 Mock 사용
            let repository = Arc::new(MockReportsRepository::new());
            ReportsService::new(repository)
        }
    }
}

/// 보고서 모듈의 초기 설정 및 검증
pub async fn initialize_reports_module(
    pool: Option<&PgPool>,
) -> crate::utils::error::ErpResult<()> {
    // 데이터베이스 연결이 있는 경우 필요한 테이블들의 존재 여부 검증
    if let Some(pg_pool) = pool {
        // 필요한 테이블들이 존재하는지 확인
        let required_tables = vec![
            "products",
            "customers",
            "sales_orders",
            "sales_order_items",
            "customer_addresses",
        ];

        for table in required_tables {
            let query = format!(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables
                    WHERE table_schema = 'public'
                    AND table_name = '{}'
                )",
                table
            );

            let exists: (bool,) = sqlx::query_as(&query)
                .fetch_one(pg_pool)
                .await
                .map_err(|e| {
                    crate::utils::error::ErpError::database(format!(
                        "테이블 존재 확인 중 오류 발생 ({}): {}",
                        table, e
                    ))
                })?;

            if !exists.0 {
                tracing::warn!("보고서 모듈에 필요한 테이블이 없습니다: {}", table);
            }
        }
    }

    tracing::info!("보고서 모듈이 성공적으로 초기화되었습니다");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_create_reports_service_mock() {
        let service = create_reports_service(None);

        let request = ReportRequest {
            report_type: ReportType::SalesSummary,
            period: ReportPeriod::Monthly,
            format: ReportFormat::Console,
            output_path: None,
            filters: ReportFilters::default(),
            include_charts: false,
        };

        let result = service.generate_sales_summary(&request).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_report_period_date_conversion() {
        let daily = ReportPeriod::Daily;
        let (start, end) = daily.to_date_range();
        assert_eq!(start, end);

        let custom = ReportPeriod::Custom {
            from: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        };
        let (start, end) = custom.to_date_range();
        assert_eq!(start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    fn test_report_format_parsing() {
        assert_eq!("json".parse::<ReportFormat>().unwrap(), ReportFormat::Json);
        assert_eq!("csv".parse::<ReportFormat>().unwrap(), ReportFormat::Csv);
        assert_eq!("html".parse::<ReportFormat>().unwrap(), ReportFormat::Html);
        assert!("invalid".parse::<ReportFormat>().is_err());
    }

    #[test]
    fn test_report_filters_default() {
        let filters = ReportFilters::default();
        assert!(filters.customer_ids.is_none());
        assert!(filters.product_ids.is_none());
        assert!(!filters.low_stock_only);
        assert!(!filters.include_inactive);
    }
}
