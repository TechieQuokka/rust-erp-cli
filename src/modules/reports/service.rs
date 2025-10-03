use chrono::Utc;
use printpdf::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

use super::models::*;
use super::repository::ReportsRepository;
use crate::utils::error::{ErpError, ErpResult};

pub struct ReportsService {
    repository: Arc<dyn ReportsRepository>,
}

impl ReportsService {
    pub fn new(repository: Arc<dyn ReportsRepository>) -> Self {
        Self { repository }
    }

    /// 매출 요약 보고서 생성
    pub async fn generate_sales_summary(
        &self,
        request: &ReportRequest,
    ) -> ErpResult<SalesSummaryReport> {
        let (start_date, end_date) = request.period.to_date_range();

        // 기본 매출 요약 데이터 조회
        let mut report = self
            .repository
            .get_sales_summary(start_date, end_date)
            .await?;

        // 필터 적용
        self.apply_sales_filters(&mut report, &request.filters)
            .await?;

        // 출력 형식에 따른 처리
        if let Some(output_path) = &request.output_path {
            self.export_sales_summary(&report, &request.format, output_path)
                .await?;
        }

        Ok(report)
    }

    /// 재고 상태 보고서 생성
    pub async fn generate_inventory_status(
        &self,
        request: &ReportRequest,
    ) -> ErpResult<InventoryStatusReport> {
        let mut report = self.repository.get_inventory_status().await?;

        // 저재고만 표시 옵션 적용
        if request.filters.low_stock_only {
            report.out_of_stock_items.clear(); // 재고 부족 아이템 제거, 저재고 아이템만 유지
        }

        // 카테고리 필터 적용
        if let Some(categories) = &request.filters.categories {
            report
                .inventory_by_category
                .retain(|item| categories.contains(&item.category));
        }

        // 비활성 제품 포함 여부
        if !request.filters.include_inactive {
            // 실제 구현에서는 repository 레벨에서 필터링해야 함
            // 현재는 이미 활성 제품만 포함됨
        }

        // 출력 형식에 따른 처리
        if let Some(output_path) = &request.output_path {
            self.export_inventory_status(&report, &request.format, output_path)
                .await?;
        }

        Ok(report)
    }

    /// 고객 분석 보고서 생성
    pub async fn generate_customer_analysis(
        &self,
        months: u32,
        request: &ReportRequest,
    ) -> ErpResult<CustomerAnalysisReport> {
        // 유효한 기간 검증
        if months == 0 || months > 120 {
            return Err(ErpError::validation(
                "months",
                "분석 기간은 1-120개월 사이여야 합니다",
            ));
        }

        let mut report = self.repository.get_customer_analysis(months).await?;

        // 고객 ID 필터 적용
        if let Some(customer_ids) = &request.filters.customer_ids {
            report
                .top_customers
                .retain(|customer| customer_ids.contains(&customer.customer_id));
        }

        // 출력 형식에 따른 처리
        if let Some(output_path) = &request.output_path {
            self.export_customer_analysis(&report, &request.format, output_path)
                .await?;
        }

        Ok(report)
    }

    /// 재무 개요 보고서 생성
    pub async fn generate_financial_overview(
        &self,
        request: &ReportRequest,
    ) -> ErpResult<FinancialOverviewReport> {
        let (start_date, end_date) = request.period.to_date_range();

        let report = self
            .repository
            .get_financial_overview(start_date, end_date)
            .await?;

        // 출력 형식에 따른 처리
        if let Some(output_path) = &request.output_path {
            self.export_financial_overview(&report, &request.format, output_path)
                .await?;
        }

        Ok(report)
    }

    /// 보고서 내보내기 (통합 메서드)
    pub async fn export_report(
        &self,
        report_type: &ReportType,
        request: &ReportRequest,
    ) -> ErpResult<String> {
        let output_path = match &request.output_path {
            Some(path) => path.clone(),
            None => {
                let filename = self.generate_filename(report_type, &request.format);
                format!("./reports/{}", filename)
            }
        };

        // 출력 디렉토리 생성
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ErpError::io(format!("보고서 디렉토리 생성 실패: {}", e)))?;
        }

        match report_type {
            ReportType::SalesSummary => {
                let report = self.generate_sales_summary(request).await?;
                self.export_sales_summary(&report, &request.format, &output_path)
                    .await?;
            }
            ReportType::InventoryStatus => {
                let report = self.generate_inventory_status(request).await?;
                self.export_inventory_status(&report, &request.format, &output_path)
                    .await?;
            }
            ReportType::CustomerAnalysis { months } => {
                let report = self.generate_customer_analysis(*months, request).await?;
                self.export_customer_analysis(&report, &request.format, &output_path)
                    .await?;
            }
            ReportType::FinancialOverview => {
                let report = self.generate_financial_overview(request).await?;
                self.export_financial_overview(&report, &request.format, &output_path)
                    .await?;
            }
        }

        Ok(output_path)
    }

    /// 보고서 요약 통계 조회
    pub async fn get_report_summary(&self, months: Option<u32>) -> ErpResult<ReportSummary> {
        let months = months.unwrap_or(1);
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days((months * 30) as i64);

        let sales_summary = self
            .repository
            .get_sales_summary(start_date, end_date)
            .await?;
        let inventory_status = self.repository.get_inventory_status().await?;
        let customer_analysis = self.repository.get_customer_analysis(months).await?;

        Ok(ReportSummary {
            period_months: months,
            generated_at: Utc::now(),
            total_revenue: sales_summary.total_revenue,
            total_orders: sales_summary.total_orders,
            total_customers: customer_analysis.total_customers,
            active_customers: customer_analysis.active_customers,
            total_products: inventory_status.total_products,
            total_stock_value: inventory_status.total_stock_value,
            low_stock_items: inventory_status.low_stock_items.len() as u32,
            out_of_stock_items: inventory_status.out_of_stock_items.len() as u32,
        })
    }

    /// 매출 트렌드 분석
    pub async fn analyze_sales_trend(&self, months: u32) -> ErpResult<SalesTrendAnalysis> {
        if !(2..=24).contains(&months) {
            return Err(ErpError::validation(
                "months",
                "트렌드 분석은 2-24개월 기간이 필요합니다",
            ));
        }

        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days((months * 30) as i64);

        let daily_sales = self
            .repository
            .get_daily_sales(start_date, end_date)
            .await?;

        // 트렌드 계산
        let total_days = daily_sales.len() as f64;
        let total_revenue: Decimal = daily_sales.iter().map(|ds| ds.total_amount).sum();
        let average_daily_revenue = if total_days > 0.0 {
            total_revenue / Decimal::from_f64(total_days).unwrap_or(Decimal::ONE)
        } else {
            Decimal::ZERO
        };

        // 성장률 계산 (단순화된 버전)
        let mid_point = daily_sales.len() / 2;
        let first_half_avg: Decimal = if mid_point > 0 {
            daily_sales[..mid_point]
                .iter()
                .map(|ds| ds.total_amount)
                .sum::<Decimal>()
                / Decimal::from(mid_point)
        } else {
            Decimal::ZERO
        };

        let second_half_avg: Decimal = if mid_point < daily_sales.len() {
            daily_sales[mid_point..]
                .iter()
                .map(|ds| ds.total_amount)
                .sum::<Decimal>()
                / Decimal::from(daily_sales.len() - mid_point)
        } else {
            Decimal::ZERO
        };

        let growth_rate = if first_half_avg > Decimal::ZERO {
            ((second_half_avg - first_half_avg) / first_half_avg) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        Ok(SalesTrendAnalysis {
            analysis_period_months: months,
            generated_at: Utc::now(),
            total_revenue,
            average_daily_revenue,
            growth_rate,
            trend_direction: if growth_rate > Decimal::ZERO {
                "상승".to_string()
            } else if growth_rate < Decimal::ZERO {
                "하락".to_string()
            } else {
                "횡보".to_string()
            },
            daily_sales,
        })
    }

    /// 재고 회전율 분석
    pub async fn analyze_inventory_turnover(
        &self,
        months: u32,
    ) -> ErpResult<InventoryTurnoverAnalysis> {
        let _inventory_status = self.repository.get_inventory_status().await?;
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days((months * 30) as i64);

        // 상위 판매 제품 조회
        let top_products = self
            .repository
            .get_top_selling_products(start_date, end_date, 20)
            .await?;

        let mut turnover_items = Vec::new();

        for product in top_products {
            // 평균 재고 가정 (실제로는 재고 이력 필요)
            let average_inventory = product.quantity_sold / 2; // 단순 가정
            let turnover_rate = if average_inventory > 0 {
                Decimal::from(product.quantity_sold) / Decimal::from(average_inventory)
            } else {
                Decimal::ZERO
            };

            turnover_items.push(InventoryTurnoverItem {
                product_id: product.product_id,
                product_name: product.name,
                sku: product.sku,
                quantity_sold: product.quantity_sold,
                average_inventory,
                turnover_rate,
                days_in_stock: if turnover_rate > Decimal::ZERO {
                    Decimal::from(months * 30) / turnover_rate
                } else {
                    Decimal::from(months * 30)
                },
            });
        }

        // 회전율 기준으로 정렬
        turnover_items.sort_by(|a, b| b.turnover_rate.cmp(&a.turnover_rate));

        let average_turnover = if !turnover_items.is_empty() {
            turnover_items
                .iter()
                .map(|item| item.turnover_rate)
                .sum::<Decimal>()
                / Decimal::from(turnover_items.len())
        } else {
            Decimal::ZERO
        };

        Ok(InventoryTurnoverAnalysis {
            analysis_period_months: months,
            generated_at: Utc::now(),
            average_turnover_rate: average_turnover,
            total_products_analyzed: turnover_items.len() as u32,
            turnover_items,
        })
    }

    // Private helper methods

    async fn apply_sales_filters(
        &self,
        report: &mut SalesSummaryReport,
        filters: &ReportFilters,
    ) -> ErpResult<()> {
        // 제품 ID 필터
        if let Some(product_ids) = &filters.product_ids {
            report
                .top_selling_products
                .retain(|product| product_ids.contains(&product.product_id));
        }

        // 주문 상태 필터
        if let Some(order_statuses) = &filters.order_statuses {
            report
                .sales_by_status
                .retain(|status| order_statuses.contains(&status.status));
        }

        Ok(())
    }

    async fn export_sales_summary(
        &self,
        report: &SalesSummaryReport,
        format: &ReportFormat,
        output_path: &str,
    ) -> ErpResult<()> {
        match format {
            ReportFormat::Json => {
                let json = serde_json::to_string_pretty(report)
                    .map_err(|e| ErpError::serialization(format!("JSON 직렬화 실패: {}", e)))?;
                fs::write(output_path, json)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Csv => {
                let csv = self.generate_sales_summary_csv(report)?;
                fs::write(output_path, csv)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Html => {
                let html = self.generate_sales_summary_html(report)?;
                fs::write(output_path, html)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Console => {
                // 콘솔 출력은 별도 처리
                return Ok(());
            }
            ReportFormat::Pdf => {
                let pdf_bytes = self.generate_sales_summary_pdf(report)?;
                fs::write(output_path, pdf_bytes)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
        }
        Ok(())
    }

    async fn export_inventory_status(
        &self,
        report: &InventoryStatusReport,
        format: &ReportFormat,
        output_path: &str,
    ) -> ErpResult<()> {
        match format {
            ReportFormat::Json => {
                let json = serde_json::to_string_pretty(report)
                    .map_err(|e| ErpError::serialization(format!("JSON 직렬화 실패: {}", e)))?;
                fs::write(output_path, json)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Csv => {
                let csv = self.generate_inventory_status_csv(report)?;
                fs::write(output_path, csv)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Html => {
                let html = self.generate_inventory_status_html(report)?;
                fs::write(output_path, html)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Console => return Ok(()),
            ReportFormat::Pdf => {
                let pdf_bytes = self.generate_inventory_status_pdf(report)?;
                fs::write(output_path, pdf_bytes)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
        }
        Ok(())
    }

    async fn export_customer_analysis(
        &self,
        report: &CustomerAnalysisReport,
        format: &ReportFormat,
        output_path: &str,
    ) -> ErpResult<()> {
        match format {
            ReportFormat::Json => {
                let json = serde_json::to_string_pretty(report)
                    .map_err(|e| ErpError::serialization(format!("JSON 직렬화 실패: {}", e)))?;
                fs::write(output_path, json)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Csv => {
                let csv = self.generate_customer_analysis_csv(report)?;
                fs::write(output_path, csv)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Html => {
                let html = self.generate_customer_analysis_html(report)?;
                fs::write(output_path, html)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Console => return Ok(()),
            ReportFormat::Pdf => {
                let pdf_bytes = self.generate_customer_analysis_pdf(report)?;
                fs::write(output_path, pdf_bytes)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
        }
        Ok(())
    }

    async fn export_financial_overview(
        &self,
        report: &FinancialOverviewReport,
        format: &ReportFormat,
        output_path: &str,
    ) -> ErpResult<()> {
        match format {
            ReportFormat::Json => {
                let json = serde_json::to_string_pretty(report)
                    .map_err(|e| ErpError::serialization(format!("JSON 직렬화 실패: {}", e)))?;
                fs::write(output_path, json)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Csv => {
                let csv = self.generate_financial_overview_csv(report)?;
                fs::write(output_path, csv)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Html => {
                let html = self.generate_financial_overview_html(report)?;
                fs::write(output_path, html)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
            ReportFormat::Console => return Ok(()),
            ReportFormat::Pdf => {
                let pdf_bytes = self.generate_financial_overview_pdf(report)?;
                fs::write(output_path, pdf_bytes)
                    .await
                    .map_err(|e| ErpError::io(format!("파일 쓰기 실패: {}", e)))?;
            }
        }
        Ok(())
    }

    fn generate_filename(&self, report_type: &ReportType, format: &ReportFormat) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let type_name = match report_type {
            ReportType::SalesSummary => "sales_summary",
            ReportType::InventoryStatus => "inventory_status",
            ReportType::CustomerAnalysis { .. } => "customer_analysis",
            ReportType::FinancialOverview => "financial_overview",
        };
        let extension = match format {
            ReportFormat::Json => "json",
            ReportFormat::Csv => "csv",
            ReportFormat::Html => "html",
            ReportFormat::Pdf => "pdf",
            ReportFormat::Console => "txt",
        };
        format!("{}_{}.{}", type_name, timestamp, extension)
    }

    // CSV 생성 메서드들
    fn generate_sales_summary_csv(&self, report: &SalesSummaryReport) -> ErpResult<String> {
        let mut csv = String::new();
        csv.push_str("매출 요약 보고서\n");
        csv.push_str(&format!(
            "생성 시간: {}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        csv.push_str(&format!("총 주문 수: {}\n", report.total_orders));
        csv.push_str(&format!("총 매출: {}\n", report.total_revenue));
        csv.push_str(&format!("총 판매 수량: {}\n", report.total_items_sold));
        csv.push_str(&format!("평균 주문 금액: {}\n", report.average_order_value));
        csv.push_str("\n상위 판매 제품:\n");
        csv.push_str("제품명,SKU,판매량,매출\n");

        for product in &report.top_selling_products {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                product.name, product.sku, product.quantity_sold, product.total_revenue
            ));
        }

        Ok(csv)
    }

    fn generate_inventory_status_csv(&self, report: &InventoryStatusReport) -> ErpResult<String> {
        let mut csv = String::new();
        csv.push_str("재고 상태 보고서\n");
        csv.push_str(&format!(
            "생성 시간: {}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        csv.push_str(&format!("총 제품 수: {}\n", report.total_products));
        csv.push_str(&format!("총 재고 가치: {}\n", report.total_stock_value));
        csv.push_str("\n저재고 아이템:\n");
        csv.push_str("제품명,SKU,현재재고,재주문수준,제안수량,재고가치\n");

        for item in &report.low_stock_items {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                item.name,
                item.sku,
                item.current_stock,
                item.reorder_level,
                item.suggested_reorder_quantity,
                item.stock_value
            ));
        }

        Ok(csv)
    }

    fn generate_customer_analysis_csv(&self, report: &CustomerAnalysisReport) -> ErpResult<String> {
        let mut csv = String::new();
        csv.push_str("고객 분석 보고서\n");
        csv.push_str(&format!(
            "생성 시간: {}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        csv.push_str(&format!(
            "분석 기간: {}개월\n",
            report.analysis_period_months
        ));
        csv.push_str(&format!("총 고객 수: {}\n", report.total_customers));
        csv.push_str(&format!("활성 고객 수: {}\n", report.active_customers));
        csv.push_str(&format!("신규 고객 수: {}\n", report.new_customers));
        csv.push_str("\n상위 고객:\n");
        csv.push_str("고객명,이메일,총주문수,총구매액,평균주문금액,최종주문일\n");

        for customer in &report.top_customers {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                customer.name,
                customer.email.as_deref().unwrap_or("N/A"),
                customer.total_orders,
                customer.total_spent,
                customer.average_order_value,
                customer
                    .last_order_date
                    .map(|d| d.to_string())
                    .unwrap_or("N/A".to_string())
            ));
        }

        Ok(csv)
    }

    fn generate_financial_overview_csv(
        &self,
        report: &FinancialOverviewReport,
    ) -> ErpResult<String> {
        let mut csv = String::new();
        csv.push_str("재무 개요 보고서\n");
        csv.push_str(&format!(
            "생성 시간: {}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        csv.push_str("\n수익 요약:\n");
        csv.push_str(&format!(
            "총 매출: {}\n",
            report.revenue_summary.total_revenue
        ));
        csv.push_str(&format!(
            "제품 매출: {}\n",
            report.revenue_summary.product_revenue
        ));
        csv.push_str(&format!(
            "서비스 매출: {}\n",
            report.revenue_summary.service_revenue
        ));
        csv.push_str("\n비용 요약:\n");
        csv.push_str(&format!(
            "총 비용: {}\n",
            report.expense_summary.total_expenses
        ));
        csv.push_str(&format!(
            "매출원가: {}\n",
            report.expense_summary.cost_of_goods_sold
        ));
        csv.push_str(&format!(
            "운영비용: {}\n",
            report.expense_summary.operating_expenses
        ));
        csv.push_str("\n수익성 분석:\n");
        csv.push_str(&format!(
            "총이익: {}\n",
            report.profit_analysis.gross_profit
        ));
        csv.push_str(&format!("순이익: {}\n", report.profit_analysis.net_profit));
        csv.push_str(&format!(
            "총이익률: {}%\n",
            report.profit_analysis.gross_margin
        ));
        csv.push_str(&format!(
            "순이익률: {}%\n",
            report.profit_analysis.net_margin
        ));

        Ok(csv)
    }

    // HTML 생성 메서드들 (기본 구현)
    fn generate_sales_summary_html(&self, report: &SalesSummaryReport) -> ErpResult<String> {
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>매출 요약 보고서</title>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .summary {{ margin-bottom: 20px; }}
        .summary h2 {{ color: #333; }}
    </style>
</head>
<body>
    <h1>매출 요약 보고서</h1>
    <div class="summary">
        <p><strong>생성 시간:</strong> {}</p>
        <p><strong>총 주문 수:</strong> {}</p>
        <p><strong>총 매출:</strong> {}</p>
        <p><strong>평균 주문 금액:</strong> {}</p>
    </div>

    <h2>상위 판매 제품</h2>
    <table>
        <thead>
            <tr>
                <th>제품명</th>
                <th>SKU</th>
                <th>판매량</th>
                <th>매출</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>"#,
            report.generated_at.format("%Y-%m-%d %H:%M:%S"),
            report.total_orders,
            report.total_revenue,
            report.average_order_value,
            report
                .top_selling_products
                .iter()
                .map(|p| format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    p.name, p.sku, p.quantity_sold, p.total_revenue
                ))
                .collect::<Vec<_>>()
                .join("")
        );
        Ok(html)
    }

    fn generate_inventory_status_html(&self, _report: &InventoryStatusReport) -> ErpResult<String> {
        // 기본 HTML 구현
        Ok(
            "<html><body><h1>재고 상태 보고서</h1><p>HTML 형식 구현 중...</p></body></html>"
                .to_string(),
        )
    }

    fn generate_customer_analysis_html(
        &self,
        _report: &CustomerAnalysisReport,
    ) -> ErpResult<String> {
        // 기본 HTML 구현
        Ok(
            "<html><body><h1>고객 분석 보고서</h1><p>HTML 형식 구현 중...</p></body></html>"
                .to_string(),
        )
    }

    fn generate_financial_overview_html(
        &self,
        report: &FinancialOverviewReport,
    ) -> ErpResult<String> {
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>재무 개요 보고서</title>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .summary {{ margin-bottom: 20px; }}
        .summary h2 {{ color: #333; }}
        .chart-placeholder {{
            width: 100%;
            height: 300px;
            background: #f0f0f0;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 20px 0;
            border: 1px solid #ddd;
        }}
    </style>
</head>
<body>
    <h1>재무 개요 보고서</h1>
    <div class="summary">
        <p><strong>생성 시간:</strong> {}</p>
    </div>

    <h2>수익 요약</h2>
    <table>
        <tr><th>항목</th><th>금액</th></tr>
        <tr><td>총 매출</td><td>{}</td></tr>
        <tr><td>제품 매출</td><td>{}</td></tr>
        <tr><td>서비스 매출</td><td>{}</td></tr>
        <tr><td>반복 매출</td><td>{}</td></tr>
        <tr><td>일회성 매출</td><td>{}</td></tr>
    </table>

    <h2>비용 요약</h2>
    <table>
        <tr><th>항목</th><th>금액</th></tr>
        <tr><td>총 비용</td><td>{}</td></tr>
        <tr><td>매출원가</td><td>{}</td></tr>
        <tr><td>운영비용</td><td>{}</td></tr>
        <tr><td>마케팅비용</td><td>{}</td></tr>
        <tr><td>관리비용</td><td>{}</td></tr>
    </table>

    <h2>수익성 분석</h2>
    <table>
        <tr><th>항목</th><th>값</th></tr>
        <tr><td>총 이익</td><td>{}</td></tr>
        <tr><td>순 이익</td><td>{}</td></tr>
        <tr><td>영업 이익</td><td>{}</td></tr>
        <tr><td>총 이익률</td><td>{}%</td></tr>
        <tr><td>순 이익률</td><td>{}%</td></tr>
        <tr><td>영업 이익률</td><td>{}%</td></tr>
    </table>

    <h2>현금 흐름</h2>
    <table>
        <tr><th>항목</th><th>금액</th></tr>
        <tr><td>현금 유입</td><td>{}</td></tr>
        <tr><td>현금 유출</td><td>{}</td></tr>
        <tr><td>순 현금 흐름</td><td>{}</td></tr>
        <tr><td>영업 현금 흐름</td><td>{}</td></tr>
    </table>

    <div class="chart-placeholder">
        <p>차트 영역 (차트 라이브러리 통합 예정)</p>
    </div>
</body>
</html>"#,
            report.generated_at.format("%Y-%m-%d %H:%M:%S"),
            report.revenue_summary.total_revenue,
            report.revenue_summary.product_revenue,
            report.revenue_summary.service_revenue,
            report.revenue_summary.recurring_revenue,
            report.revenue_summary.one_time_revenue,
            report.expense_summary.total_expenses,
            report.expense_summary.cost_of_goods_sold,
            report.expense_summary.operating_expenses,
            report.expense_summary.marketing_expenses,
            report.expense_summary.administrative_expenses,
            report.profit_analysis.gross_profit,
            report.profit_analysis.net_profit,
            report.profit_analysis.operating_profit,
            report.profit_analysis.gross_margin,
            report.profit_analysis.net_margin,
            report.profit_analysis.operating_margin,
            report.cash_flow.cash_inflow,
            report.cash_flow.cash_outflow,
            report.cash_flow.net_cash_flow,
            report.cash_flow.operating_cash_flow,
        );
        Ok(html)
    }

    // PDF 생성 메서드들
    fn generate_sales_summary_pdf(&self, report: &SalesSummaryReport) -> ErpResult<Vec<u8>> {
        let (doc, page1, layer1) =
            PdfDocument::new("매출 요약 보고서", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| ErpError::internal(format!("PDF 폰트 로드 실패: {:?}", e)))?;
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // 제목
        current_layer.use_text("매출 요약 보고서", 24.0, Mm(20.0), Mm(270.0), &font);

        // 생성 시간
        let generated_at = format!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        current_layer.use_text(&generated_at, 12.0, Mm(20.0), Mm(250.0), &font);

        // 요약 정보
        let y_pos = 230.0;
        current_layer.use_text(
            format!("총 주문 수: {}", report.total_orders),
            12.0,
            Mm(20.0),
            Mm(y_pos),
            &font,
        );
        current_layer.use_text(
            format!("총 매출: {}", report.total_revenue),
            12.0,
            Mm(20.0),
            Mm(y_pos - 10.0),
            &font,
        );
        current_layer.use_text(
            format!("총 판매 수량: {}", report.total_items_sold),
            12.0,
            Mm(20.0),
            Mm(y_pos - 20.0),
            &font,
        );
        current_layer.use_text(
            format!("평균 주문 금액: {}", report.average_order_value),
            12.0,
            Mm(20.0),
            Mm(y_pos - 30.0),
            &font,
        );

        // 상위 판매 제품
        let mut y = y_pos - 50.0;
        current_layer.use_text("상위 판매 제품", 14.0, Mm(20.0), Mm(y), &font);
        y -= 15.0;

        for product in &report.top_selling_products {
            let product_line = format!(
                "{} ({}): {} 판매, 매출 {}",
                product.name, product.sku, product.quantity_sold, product.total_revenue
            );
            current_layer.use_text(&product_line, 10.0, Mm(25.0), Mm(y), &font);
            y -= 12.0;
            if y < 30.0 {
                break;
            } // 페이지 하단 근처면 중지
        }

        doc.save_to_bytes()
            .map_err(|e| ErpError::internal(format!("PDF 저장 실패: {:?}", e)))
    }

    fn generate_inventory_status_pdf(&self, report: &InventoryStatusReport) -> ErpResult<Vec<u8>> {
        let (doc, page1, layer1) =
            PdfDocument::new("재고 상태 보고서", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| ErpError::internal(format!("PDF 폰트 로드 실패: {:?}", e)))?;
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // 제목
        current_layer.use_text("재고 상태 보고서", 24.0, Mm(20.0), Mm(270.0), &font);

        // 생성 시간
        let generated_at = format!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        current_layer.use_text(&generated_at, 12.0, Mm(20.0), Mm(250.0), &font);

        // 요약 정보
        let y_pos = 230.0;
        current_layer.use_text(
            format!("총 제품 수: {}", report.total_products),
            12.0,
            Mm(20.0),
            Mm(y_pos),
            &font,
        );
        current_layer.use_text(
            format!("총 재고 가치: {}", report.total_stock_value),
            12.0,
            Mm(20.0),
            Mm(y_pos - 10.0),
            &font,
        );
        current_layer.use_text(
            format!("저재고 아이템: {}", report.low_stock_items.len()),
            12.0,
            Mm(20.0),
            Mm(y_pos - 20.0),
            &font,
        );
        current_layer.use_text(
            format!("품절 아이템: {}", report.out_of_stock_items.len()),
            12.0,
            Mm(20.0),
            Mm(y_pos - 30.0),
            &font,
        );

        // 저재고 아이템
        if !report.low_stock_items.is_empty() {
            let mut y = y_pos - 50.0;
            current_layer.use_text("저재고 아이템", 14.0, Mm(20.0), Mm(y), &font);
            y -= 15.0;

            for item in &report.low_stock_items {
                let item_line = format!(
                    "{} ({}): 현재 {}, 재주문 레벨 {}",
                    item.name, item.sku, item.current_stock, item.reorder_level
                );
                current_layer.use_text(&item_line, 10.0, Mm(25.0), Mm(y), &font);
                y -= 12.0;
                if y < 30.0 {
                    break;
                }
            }
        }

        doc.save_to_bytes()
            .map_err(|e| ErpError::internal(format!("PDF 저장 실패: {:?}", e)))
    }

    fn generate_customer_analysis_pdf(
        &self,
        report: &CustomerAnalysisReport,
    ) -> ErpResult<Vec<u8>> {
        let (doc, page1, layer1) =
            PdfDocument::new("고객 분석 보고서", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| ErpError::internal(format!("PDF 폰트 로드 실패: {:?}", e)))?;
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // 제목
        current_layer.use_text("고객 분석 보고서", 24.0, Mm(20.0), Mm(270.0), &font);

        // 생성 시간
        let generated_at = format!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        current_layer.use_text(&generated_at, 12.0, Mm(20.0), Mm(250.0), &font);

        // 요약 정보
        let y_pos = 230.0;
        current_layer.use_text(
            format!("분석 기간: {}개월", report.analysis_period_months),
            12.0,
            Mm(20.0),
            Mm(y_pos),
            &font,
        );
        current_layer.use_text(
            format!("총 고객 수: {}", report.total_customers),
            12.0,
            Mm(20.0),
            Mm(y_pos - 10.0),
            &font,
        );
        current_layer.use_text(
            format!("활성 고객: {}", report.active_customers),
            12.0,
            Mm(20.0),
            Mm(y_pos - 20.0),
            &font,
        );
        current_layer.use_text(
            format!("신규 고객: {}", report.new_customers),
            12.0,
            Mm(20.0),
            Mm(y_pos - 30.0),
            &font,
        );

        // 상위 고객
        if !report.top_customers.is_empty() {
            let mut y = y_pos - 50.0;
            current_layer.use_text("상위 고객", 14.0, Mm(20.0), Mm(y), &font);
            y -= 15.0;

            for customer in &report.top_customers {
                let customer_line = format!(
                    "{}: {} 주문, 총 {}",
                    customer.name, customer.total_orders, customer.total_spent
                );
                current_layer.use_text(&customer_line, 10.0, Mm(25.0), Mm(y), &font);
                y -= 12.0;
                if y < 30.0 {
                    break;
                }
            }
        }

        doc.save_to_bytes()
            .map_err(|e| ErpError::internal(format!("PDF 저장 실패: {:?}", e)))
    }

    fn generate_financial_overview_pdf(
        &self,
        report: &FinancialOverviewReport,
    ) -> ErpResult<Vec<u8>> {
        let (doc, page1, layer1) =
            PdfDocument::new("재무 개요 보고서", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| ErpError::internal(format!("PDF 폰트 로드 실패: {:?}", e)))?;
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // 제목
        current_layer.use_text("재무 개요 보고서", 24.0, Mm(20.0), Mm(270.0), &font);

        // 생성 시간
        let generated_at = format!(
            "생성 시간: {}",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        );
        current_layer.use_text(&generated_at, 12.0, Mm(20.0), Mm(250.0), &font);

        // 수익 요약
        let y_pos = 230.0;
        current_layer.use_text("수익 요약", 14.0, Mm(20.0), Mm(y_pos), &font);
        current_layer.use_text(
            format!("총 수익: {}", report.revenue_summary.total_revenue),
            12.0,
            Mm(25.0),
            Mm(y_pos - 15.0),
            &font,
        );
        current_layer.use_text(
            format!("제품 수익: {}", report.revenue_summary.product_revenue),
            12.0,
            Mm(25.0),
            Mm(y_pos - 25.0),
            &font,
        );
        current_layer.use_text(
            format!("서비스 수익: {}", report.revenue_summary.service_revenue),
            12.0,
            Mm(25.0),
            Mm(y_pos - 35.0),
            &font,
        );

        // 수익성 분석
        let y = y_pos - 55.0;
        current_layer.use_text("수익성 분석", 14.0, Mm(20.0), Mm(y), &font);
        current_layer.use_text(
            format!("총 이익: {}", report.profit_analysis.gross_profit),
            12.0,
            Mm(25.0),
            Mm(y - 15.0),
            &font,
        );
        current_layer.use_text(
            format!("순이익: {}", report.profit_analysis.net_profit),
            12.0,
            Mm(25.0),
            Mm(y - 25.0),
            &font,
        );
        current_layer.use_text(
            format!("영업 이익: {}", report.profit_analysis.operating_profit),
            12.0,
            Mm(25.0),
            Mm(y - 35.0),
            &font,
        );

        doc.save_to_bytes()
            .map_err(|e| ErpError::internal(format!("PDF 저장 실패: {:?}", e)))
    }
}

// 추가 데이터 구조체들
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReportSummary {
    pub period_months: u32,
    pub generated_at: chrono::DateTime<Utc>,
    pub total_revenue: Decimal,
    pub total_orders: u32,
    pub total_customers: u32,
    pub active_customers: u32,
    pub total_products: u32,
    pub total_stock_value: Decimal,
    pub low_stock_items: u32,
    pub out_of_stock_items: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SalesTrendAnalysis {
    pub analysis_period_months: u32,
    pub generated_at: chrono::DateTime<Utc>,
    pub total_revenue: Decimal,
    pub average_daily_revenue: Decimal,
    pub growth_rate: Decimal,
    pub trend_direction: String,
    pub daily_sales: Vec<DailySales>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InventoryTurnoverAnalysis {
    pub analysis_period_months: u32,
    pub generated_at: chrono::DateTime<Utc>,
    pub average_turnover_rate: Decimal,
    pub total_products_analyzed: u32,
    pub turnover_items: Vec<InventoryTurnoverItem>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InventoryTurnoverItem {
    pub product_id: uuid::Uuid,
    pub product_name: String,
    pub sku: String,
    pub quantity_sold: u32,
    pub average_inventory: u32,
    pub turnover_rate: Decimal,
    pub days_in_stock: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::reports::repository::MockReportsRepository;

    #[tokio::test]
    async fn test_generate_sales_summary() {
        let repo = Arc::new(MockReportsRepository::new());
        let service = ReportsService::new(repo);

        let request = ReportRequest {
            report_type: ReportType::SalesSummary,
            period: ReportPeriod::Monthly,
            format: ReportFormat::Json,
            output_path: None,
            filters: ReportFilters::default(),
            include_charts: false,
        };

        let result = service.generate_sales_summary(&request).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.total_orders > 0);
        assert!(report.total_revenue > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_generate_inventory_status() {
        let repo = Arc::new(MockReportsRepository::new());
        let service = ReportsService::new(repo);

        let request = ReportRequest {
            report_type: ReportType::InventoryStatus,
            period: ReportPeriod::Daily,
            format: ReportFormat::Console,
            output_path: None,
            filters: ReportFilters::default(),
            include_charts: false,
        };

        let result = service.generate_inventory_status(&request).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.total_products > 0);
    }

    #[tokio::test]
    async fn test_analyze_sales_trend() {
        let repo = Arc::new(MockReportsRepository::new());
        let service = ReportsService::new(repo);

        let result = service.analyze_sales_trend(6).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.analysis_period_months, 6);
    }

    #[tokio::test]
    async fn test_invalid_customer_analysis_months() {
        let repo = Arc::new(MockReportsRepository::new());
        let service = ReportsService::new(repo);

        let request = ReportRequest {
            report_type: ReportType::CustomerAnalysis { months: 0 },
            period: ReportPeriod::Monthly,
            format: ReportFormat::Console,
            output_path: None,
            filters: ReportFilters::default(),
            include_charts: false,
        };

        let result = service.generate_customer_analysis(0, &request).await;
        assert!(result.is_err());

        if let Err(ErpError::Validation { field, reason }) = result {
            assert_eq!(field, "months");
            assert!(reason.contains("1-120개월"));
        }
    }
}
