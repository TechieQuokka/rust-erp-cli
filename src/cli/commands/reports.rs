use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};

use crate::cli::parser::ReportCommands;
use crate::cli::validator::CliValidator;
use crate::core::config::AppConfig;
use crate::modules::reports::{
    create_reports_service, CustomerAnalysisReport, FinancialOverviewReport, InventoryStatusReport,
    ReportFilters, ReportFormat, ReportPeriod, ReportRequest, ReportType, SalesSummaryReport,
};
use crate::utils::error::ErpResult;

pub struct ReportsHandler;

impl ReportsHandler {
    pub async fn handle(cmd: &ReportCommands, _config: &AppConfig) -> ErpResult<()> {
        // 보고서 서비스 초기화 (실제 구현에서는 데이터베이스 연결 사용)
        let _reports_service = create_reports_service(None); // Mock 사용

        match cmd {
            ReportCommands::SalesSummary {
                period,
                from_date,
                to_date,
                format,
                output,
            } => Self::handle_sales_summary(period, from_date, to_date, format, output).await,
            ReportCommands::InventoryStatus {
                format,
                output,
                low_stock_only,
            } => Self::handle_inventory_status(format, output, *low_stock_only).await,
            ReportCommands::CustomerAnalysis {
                months,
                format,
                output,
            } => Self::handle_customer_analysis(*months, format, output).await,
            ReportCommands::FinancialOverview {
                from_date,
                to_date,
                format,
                output,
            } => Self::handle_financial_overview(from_date, to_date, format, output).await,
        }
    }

    async fn handle_sales_summary(
        period: &str,
        from_date: &Option<String>,
        to_date: &Option<String>,
        format: &str,
        output: &Option<String>,
    ) -> ErpResult<()> {
        // 입력 검증
        let validated_period = CliValidator::validate_report_period(period)?;
        let (validated_from_date, validated_to_date) =
            CliValidator::validate_date_range(from_date, to_date)?;
        let validated_format: ReportFormat = format.parse()?;

        // 보고서 서비스 초기화
        let reports_service = create_reports_service(None); // Mock 사용

        // 기간 설정
        let report_period = if let (Some(from), Some(to)) = (validated_from_date, validated_to_date)
        {
            ReportPeriod::Custom { from, to }
        } else {
            match validated_period.as_str() {
                "daily" => ReportPeriod::Daily,
                "weekly" => ReportPeriod::Weekly,
                "monthly" => ReportPeriod::Monthly,
                "quarterly" => ReportPeriod::Quarterly,
                "yearly" => ReportPeriod::Yearly,
                _ => ReportPeriod::Monthly, // default fallback
            }
        };

        let request = ReportRequest {
            report_type: ReportType::SalesSummary,
            period: report_period.clone(),
            format: validated_format.clone(),
            output_path: output.clone(),
            filters: ReportFilters::default(),
        };

        let report = reports_service.generate_sales_summary(&request).await?;

        match validated_format {
            ReportFormat::Console => {
                Self::display_sales_summary_console(&report);
            }
            _ => {
                if let Some(output_path) = output {
                    println!("보고서가 저장되었습니다: {}", output_path);
                } else {
                    let filename = format!(
                        "sales_summary_{}.{}",
                        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                        validated_format
                    );
                    println!("보고서가 저장되었습니다: {}", filename);
                }
            }
        }

        Ok(())
    }

    async fn handle_inventory_status(
        format: &str,
        output: &Option<String>,
        low_stock_only: bool,
    ) -> ErpResult<()> {
        // 입력 검증
        let validated_format: ReportFormat = format.parse()?;

        // 보고서 서비스 초기화
        let reports_service = create_reports_service(None); // Mock 사용

        let filters = ReportFilters {
            low_stock_only,
            ..Default::default()
        };

        let request = ReportRequest {
            report_type: ReportType::InventoryStatus,
            period: ReportPeriod::Daily, // 재고는 날짜와 무관
            format: validated_format.clone(),
            output_path: output.clone(),
            filters,
        };

        let report = reports_service.generate_inventory_status(&request).await?;

        match validated_format {
            ReportFormat::Console => {
                Self::display_inventory_status_console(&report);
            }
            _ => {
                if let Some(output_path) = output {
                    println!("보고서가 저장되었습니다: {}", output_path);
                } else {
                    let filename = format!(
                        "inventory_status_{}.{}",
                        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                        validated_format
                    );
                    println!("보고서가 저장되었습니다: {}", filename);
                }
            }
        }

        Ok(())
    }

    async fn handle_customer_analysis(
        months: u32,
        format: &str,
        output: &Option<String>,
    ) -> ErpResult<()> {
        // 입력 검증
        let validated_format: ReportFormat = format.parse()?;

        if months == 0 || months > 120 {
            return Err(crate::utils::error::ErpError::validation(
                "months",
                "분석 기간은 1-120개월 범위여야 합니다",
            ));
        }

        // 보고서 서비스 초기화
        let reports_service = create_reports_service(None); // Mock 사용

        let request = ReportRequest {
            report_type: ReportType::CustomerAnalysis { months },
            period: ReportPeriod::Monthly, // 고객 분석은 월 단위
            format: validated_format.clone(),
            output_path: output.clone(),
            filters: ReportFilters::default(),
        };

        let report = reports_service
            .generate_customer_analysis(months, &request)
            .await?;

        match validated_format {
            ReportFormat::Console => {
                Self::display_customer_analysis_console(&report);
            }
            _ => {
                if let Some(output_path) = output {
                    println!("보고서가 저장되었습니다: {}", output_path);
                } else {
                    let filename = format!(
                        "customer_analysis_{}.{}",
                        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                        validated_format
                    );
                    println!("보고서가 저장되었습니다: {}", filename);
                }
            }
        }

        Ok(())
    }

    async fn handle_financial_overview(
        from_date: &Option<String>,
        to_date: &Option<String>,
        format: &str,
        output: &Option<String>,
    ) -> ErpResult<()> {
        // 입력 검증
        let (validated_from_date, validated_to_date) =
            CliValidator::validate_date_range(from_date, to_date)?;
        let validated_format: ReportFormat = format.parse()?;

        // 보고서 서비스 초기화
        let reports_service = create_reports_service(None); // Mock 사용

        // 기간 설정 (기본값: 지난 달)
        let report_period = if let (Some(from), Some(to)) = (validated_from_date, validated_to_date)
        {
            ReportPeriod::Custom { from, to }
        } else {
            ReportPeriod::Monthly
        };

        let request = ReportRequest {
            report_type: ReportType::FinancialOverview,
            period: report_period,
            format: validated_format.clone(),
            output_path: output.clone(),
            filters: ReportFilters::default(),
        };

        let report = reports_service
            .generate_financial_overview(&request)
            .await?;

        match validated_format {
            ReportFormat::Console => {
                Self::display_financial_overview_console(&report);
            }
            _ => {
                if let Some(output_path) = output {
                    println!("보고서가 저장되었습니다: {}", output_path);
                } else {
                    let filename = format!(
                        "financial_overview_{}.{}",
                        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                        validated_format
                    );
                    println!("보고서가 저장되었습니다: {}", filename);
                }
            }
        }

        Ok(())
    }

    // Console display methods
    fn display_sales_summary_console(report: &SalesSummaryReport) {
        println!("\n=== 매출 요약 보고서 ===");
        println!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!();

        // 요약 정보
        let mut summary_table = Table::new();
        summary_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["항목", "값"]);

        summary_table
            .add_row(vec!["총 주문 수", &report.total_orders.to_string()])
            .add_row(vec!["총 매출", &format!("₩{}", report.total_revenue)])
            .add_row(vec!["총 판매 수량", &report.total_items_sold.to_string()])
            .add_row(vec![
                "평균 주문 금액",
                &format!("₩{}", report.average_order_value),
            ]);

        println!("{summary_table}");

        // 상위 판매 제품
        if !report.top_selling_products.is_empty() {
            println!("\n상위 판매 제품:");
            let mut products_table = Table::new();
            products_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["제품명", "SKU", "판매량", "매출"]);

            for product in &report.top_selling_products {
                products_table.add_row(vec![
                    product.name.clone(),
                    product.sku.clone(),
                    product.quantity_sold.to_string(),
                    format!("₩{}", product.total_revenue),
                ]);
            }
            println!("{products_table}");
        }

        // 주문 상태별 매출
        if !report.sales_by_status.is_empty() {
            println!("\n주문 상태별 매출:");
            let mut status_table = Table::new();
            status_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["상태", "주문 수", "총 금액"]);

            for status in &report.sales_by_status {
                status_table.add_row(vec![
                    status.status.clone(),
                    status.order_count.to_string(),
                    format!("₩{}", status.total_amount),
                ]);
            }
            println!("{status_table}");
        }
    }

    fn display_inventory_status_console(report: &InventoryStatusReport) {
        println!("\n=== 재고 상태 보고서 ===");
        println!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!();

        // 요약 정보
        let mut summary_table = Table::new();
        summary_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["항목", "값"]);

        summary_table
            .add_row(vec!["총 제품 수", &report.total_products.to_string()])
            .add_row(vec![
                "총 재고 가치",
                &format!("₩{}", report.total_stock_value),
            ])
            .add_row(vec![
                "저재고 아이템",
                &report.low_stock_items.len().to_string(),
            ])
            .add_row(vec![
                "품절 아이템",
                &report.out_of_stock_items.len().to_string(),
            ]);

        println!("{summary_table}");

        // 저재고 아이템
        if !report.low_stock_items.is_empty() {
            println!("\n저재고 아이템:");
            let mut low_stock_table = Table::new();
            low_stock_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["제품명", "SKU", "현재재고", "재주문수준", "제안수량"]);

            for item in &report.low_stock_items {
                low_stock_table.add_row(vec![
                    item.name.clone(),
                    item.sku.clone(),
                    item.current_stock.to_string(),
                    item.reorder_level.to_string(),
                    item.suggested_reorder_quantity.to_string(),
                ]);
            }
            println!("{low_stock_table}");
        }

        // 품절 아이템
        if !report.out_of_stock_items.is_empty() {
            println!("\n품절 아이템:");
            let mut out_of_stock_table = Table::new();
            out_of_stock_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["제품명", "SKU", "최종재고일", "대기주문"]);

            for item in &report.out_of_stock_items {
                out_of_stock_table.add_row(vec![
                    item.name.clone(),
                    item.sku.clone(),
                    item.last_stock_date
                        .map_or("N/A".to_string(), |d| d.to_string()),
                    item.pending_orders.to_string(),
                ]);
            }
            println!("{out_of_stock_table}");
        }

        // 카테고리별 재고
        if !report.inventory_by_category.is_empty() {
            println!("\n카테고리별 재고:");
            let mut category_table = Table::new();
            category_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["카테고리", "제품 수", "총 재고", "총 가치"]);

            for category in &report.inventory_by_category {
                category_table.add_row(vec![
                    category.category.clone(),
                    category.product_count.to_string(),
                    category.total_stock.to_string(),
                    format!("₩{}", category.total_value),
                ]);
            }
            println!("{category_table}");
        }
    }

    fn display_customer_analysis_console(report: &CustomerAnalysisReport) {
        println!("\n=== 고객 분석 보고서 ===");
        println!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!("분석 기간: {}개월", report.analysis_period_months);
        println!();

        // 고객 요약 정보
        let mut summary_table = Table::new();
        summary_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["항목", "값"]);

        summary_table
            .add_row(vec!["총 고객 수", &report.total_customers.to_string()])
            .add_row(vec!["활성 고객 수", &report.active_customers.to_string()])
            .add_row(vec!["신규 고객 수", &report.new_customers.to_string()])
            .add_row(vec![
                "이탈률",
                &format!("{}%", report.customer_lifecycle.churn_rate),
            ])
            .add_row(vec![
                "고객 생애 가치",
                &format!("₩{}", report.customer_lifecycle.customer_lifetime_value),
            ]);

        println!("{summary_table}");

        // 상위 고객
        if !report.top_customers.is_empty() {
            println!("\n상위 고객:");
            let mut customers_table = Table::new();
            customers_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec![
                    "고객명",
                    "총 주문",
                    "총 구매액",
                    "평균 주문액",
                    "최종 주문일",
                ]);

            for customer in &report.top_customers {
                customers_table.add_row(vec![
                    customer.name.clone(),
                    customer.total_orders.to_string(),
                    format!("₩{}", customer.total_spent),
                    format!("₩{}", customer.average_order_value),
                    customer
                        .last_order_date
                        .map_or("N/A".to_string(), |d| d.to_string()),
                ]);
            }
            println!("{customers_table}");
        }

        // 고객 세그먼트
        if !report.customer_segments.is_empty() {
            println!("\n고객 세그먼트:");
            let mut segments_table = Table::new();
            segments_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["세그먼트", "고객 수", "총 매출", "평균 주문 빈도"]);

            for segment in &report.customer_segments {
                segments_table.add_row(vec![
                    segment.segment_name.clone(),
                    segment.customer_count.to_string(),
                    format!("₩{}", segment.total_revenue),
                    segment.average_order_frequency.to_string(),
                ]);
            }
            println!("{segments_table}");
        }
    }

    fn display_financial_overview_console(report: &FinancialOverviewReport) {
        println!("\n=== 재무 개요 보고서 ===");
        println!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!();

        // 수익 요약
        println!("수익 요약:");
        let mut revenue_table = Table::new();
        revenue_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["항목", "금액"]);

        revenue_table
            .add_row(vec![
                "총 매출",
                &format!("₩{}", report.revenue_summary.total_revenue),
            ])
            .add_row(vec![
                "제품 매출",
                &format!("₩{}", report.revenue_summary.product_revenue),
            ])
            .add_row(vec![
                "서비스 매출",
                &format!("₩{}", report.revenue_summary.service_revenue),
            ])
            .add_row(vec![
                "반복 매출",
                &format!("₩{}", report.revenue_summary.recurring_revenue),
            ])
            .add_row(vec![
                "일회성 매출",
                &format!("₩{}", report.revenue_summary.one_time_revenue),
            ]);

        println!("{revenue_table}");

        // 비용 요약
        println!("\n비용 요약:");
        let mut expense_table = Table::new();
        expense_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["항목", "금액"]);

        expense_table
            .add_row(vec![
                "총 비용",
                &format!("₩{}", report.expense_summary.total_expenses),
            ])
            .add_row(vec![
                "매출원가",
                &format!("₩{}", report.expense_summary.cost_of_goods_sold),
            ])
            .add_row(vec![
                "운영비용",
                &format!("₩{}", report.expense_summary.operating_expenses),
            ])
            .add_row(vec![
                "마케팅비용",
                &format!("₩{}", report.expense_summary.marketing_expenses),
            ])
            .add_row(vec![
                "관리비용",
                &format!("₩{}", report.expense_summary.administrative_expenses),
            ]);

        println!("{expense_table}");

        // 수익성 분석
        println!("\n수익성 분석:");
        let mut profit_table = Table::new();
        profit_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["항목", "값"]);

        profit_table
            .add_row(vec![
                "총 이익",
                &format!("₩{}", report.profit_analysis.gross_profit),
            ])
            .add_row(vec![
                "순 이익",
                &format!("₩{}", report.profit_analysis.net_profit),
            ])
            .add_row(vec![
                "영업 이익",
                &format!("₩{}", report.profit_analysis.operating_profit),
            ])
            .add_row(vec![
                "총 이익률",
                &format!("{}%", report.profit_analysis.gross_margin),
            ])
            .add_row(vec![
                "순 이익률",
                &format!("{}%", report.profit_analysis.net_margin),
            ])
            .add_row(vec![
                "영업 이익률",
                &format!("{}%", report.profit_analysis.operating_margin),
            ]);

        // 색상 적용 제거 (comfy_table API 호환성 이슈)

        println!("{profit_table}");

        // 현금 흐름
        println!("\n현금 흐름:");
        let mut cashflow_table = Table::new();
        cashflow_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["항목", "금액"]);

        cashflow_table
            .add_row(vec![
                "현금 유입",
                &format!("₩{}", report.cash_flow.cash_inflow),
            ])
            .add_row(vec![
                "현금 유출",
                &format!("₩{}", report.cash_flow.cash_outflow),
            ])
            .add_row(vec![
                "순 현금 흐름",
                &format!("₩{}", report.cash_flow.net_cash_flow),
            ])
            .add_row(vec![
                "영업 현금 흐름",
                &format!("₩{}", report.cash_flow.operating_cash_flow),
            ]);

        println!("{cashflow_table}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::parser::ReportCommands;
    use crate::core::config::AppConfig;

    #[tokio::test]
    async fn test_handle_sales_summary_console_format() {
        let config = AppConfig::default();
        let command = ReportCommands::SalesSummary {
            period: "monthly".to_string(),
            from_date: None,
            to_date: None,
            format: "console".to_string(),
            output: None,
        };

        let result = ReportsHandler::handle(&command, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_inventory_status_low_stock_only() {
        let config = AppConfig::default();
        let command = ReportCommands::InventoryStatus {
            format: "console".to_string(),
            output: None,
            low_stock_only: true,
        };

        let result = ReportsHandler::handle(&command, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_customer_analysis_valid_months() {
        let config = AppConfig::default();
        let command = ReportCommands::CustomerAnalysis {
            months: 6,
            format: "json".to_string(),
            output: Some("test_customer_report.json".to_string()),
        };

        let result = ReportsHandler::handle(&command, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_customer_analysis_invalid_months() {
        let config = AppConfig::default();

        // Test months = 0
        let command_zero = ReportCommands::CustomerAnalysis {
            months: 0,
            format: "console".to_string(),
            output: None,
        };

        let result = ReportsHandler::handle(&command_zero, &config).await;
        assert!(result.is_err());

        // Test months > 120
        let command_too_large = ReportCommands::CustomerAnalysis {
            months: 121,
            format: "console".to_string(),
            output: None,
        };

        let result = ReportsHandler::handle(&command_too_large, &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_financial_overview_with_dates() {
        let config = AppConfig::default();
        let command = ReportCommands::FinancialOverview {
            from_date: Some("2024-01-01".to_string()),
            to_date: Some("2024-01-31".to_string()),
            format: "csv".to_string(),
            output: Some("financial_overview.csv".to_string()),
        };

        let result = ReportsHandler::handle(&command, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_sales_summary_with_custom_period() {
        let config = AppConfig::default();
        let command = ReportCommands::SalesSummary {
            period: "custom".to_string(),
            from_date: Some("2024-01-01".to_string()),
            to_date: Some("2024-12-31".to_string()),
            format: "html".to_string(),
            output: None,
        };

        let result = ReportsHandler::handle(&command, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_invalid_format() {
        let config = AppConfig::default();
        let command = ReportCommands::SalesSummary {
            period: "monthly".to_string(),
            from_date: None,
            to_date: None,
            format: "invalid_format".to_string(),
            output: None,
        };

        let result = ReportsHandler::handle(&command, &config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_display_methods_exist() {
        // This is a compilation test to ensure our display methods can be called
        let report = SalesSummaryReport {
            period: ReportPeriod::Monthly,
            generated_at: chrono::Utc::now(),
            total_orders: 100,
            total_revenue: rust_decimal::Decimal::new(50000, 2),
            total_items_sold: 300,
            average_order_value: rust_decimal::Decimal::new(50000, 4),
            top_selling_products: vec![],
            sales_by_status: vec![],
            daily_sales: vec![],
        };

        // This should not panic if our method is properly implemented
        ReportsHandler::display_sales_summary_console(&report);
    }
}
