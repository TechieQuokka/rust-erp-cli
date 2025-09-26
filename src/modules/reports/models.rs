use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 보고서 생성을 위한 기간 정의
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom { from: NaiveDate, to: NaiveDate },
}

/// 보고서 출력 형식
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportFormat {
    Console,
    Json,
    Csv,
    Html,
    Pdf,
}

impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportFormat::Console => write!(f, "console"),
            ReportFormat::Json => write!(f, "json"),
            ReportFormat::Csv => write!(f, "csv"),
            ReportFormat::Html => write!(f, "html"),
            ReportFormat::Pdf => write!(f, "pdf"),
        }
    }
}

/// 매출 요약 보고서 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesSummaryReport {
    pub period: ReportPeriod,
    pub generated_at: DateTime<Utc>,
    pub total_orders: u32,
    pub total_revenue: Decimal,
    pub total_items_sold: u32,
    pub average_order_value: Decimal,
    pub top_selling_products: Vec<TopSellingProduct>,
    pub sales_by_status: Vec<SalesByStatus>,
    pub daily_sales: Vec<DailySales>,
}

/// 상위 판매 제품 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopSellingProduct {
    pub product_id: uuid::Uuid,
    pub name: String,
    pub sku: String,
    pub quantity_sold: u32,
    pub total_revenue: Decimal,
}

/// 주문 상태별 매출 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesByStatus {
    pub status: String,
    pub order_count: u32,
    pub total_amount: Decimal,
}

/// 일별 매출 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySales {
    pub date: NaiveDate,
    pub order_count: u32,
    pub total_amount: Decimal,
}

/// 재고 상태 보고서 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryStatusReport {
    pub generated_at: DateTime<Utc>,
    pub total_products: u32,
    pub total_stock_value: Decimal,
    pub low_stock_items: Vec<LowStockItem>,
    pub out_of_stock_items: Vec<OutOfStockItem>,
    pub inventory_by_category: Vec<InventoryByCategory>,
    pub stock_movements: Vec<StockMovement>,
}

/// 저재고 아이템 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowStockItem {
    pub product_id: uuid::Uuid,
    pub name: String,
    pub sku: String,
    pub current_stock: u32,
    pub reorder_level: u32,
    pub suggested_reorder_quantity: u32,
    pub stock_value: Decimal,
}

/// 재고 부족 아이템 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfStockItem {
    pub product_id: uuid::Uuid,
    pub name: String,
    pub sku: String,
    pub last_stock_date: Option<NaiveDate>,
    pub pending_orders: u32,
}

/// 카테고리별 재고 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryByCategory {
    pub category: String,
    pub product_count: u32,
    pub total_stock: u32,
    pub total_value: Decimal,
    pub average_stock_per_product: Decimal,
}

/// 재고 이동 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
    pub date: NaiveDate,
    pub movement_type: String, // "In", "Out", "Adjustment"
    pub total_quantity: i32,
    pub total_value: Decimal,
}

/// 고객 분석 보고서 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAnalysisReport {
    pub analysis_period_months: u32,
    pub generated_at: DateTime<Utc>,
    pub total_customers: u32,
    pub active_customers: u32,
    pub new_customers: u32,
    pub top_customers: Vec<TopCustomer>,
    pub customer_segments: Vec<CustomerSegment>,
    pub geographic_distribution: Vec<GeographicDistribution>,
    pub customer_lifecycle: CustomerLifecycleMetrics,
}

/// 상위 고객 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopCustomer {
    pub customer_id: uuid::Uuid,
    pub name: String,
    pub email: Option<String>,
    pub total_orders: u32,
    pub total_spent: Decimal,
    pub average_order_value: Decimal,
    pub last_order_date: Option<NaiveDate>,
}

/// 고객 세그먼트 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSegment {
    pub segment_name: String,
    pub customer_count: u32,
    pub total_revenue: Decimal,
    pub average_order_frequency: Decimal,
}

/// 지리적 분포 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicDistribution {
    pub country: String,
    pub state_province: Option<String>,
    pub city: Option<String>,
    pub customer_count: u32,
    pub total_revenue: Decimal,
}

/// 고객 생애주기 메트릭
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerLifecycleMetrics {
    pub new_customers: u32,
    pub returning_customers: u32,
    pub churn_rate: Decimal,
    pub customer_lifetime_value: Decimal,
    pub average_customer_lifespan_days: u32,
}

/// 재무 개요 보고서 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialOverviewReport {
    pub period: ReportPeriod,
    pub generated_at: DateTime<Utc>,
    pub revenue_summary: RevenueSummary,
    pub expense_summary: ExpenseSummary,
    pub profit_analysis: ProfitAnalysis,
    pub cash_flow: CashFlowAnalysis,
    pub payment_analytics: PaymentAnalytics,
    pub financial_ratios: FinancialRatios,
}

/// 수익 요약
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueSummary {
    pub total_revenue: Decimal,
    pub product_revenue: Decimal,
    pub service_revenue: Decimal,
    pub recurring_revenue: Decimal,
    pub one_time_revenue: Decimal,
    pub revenue_growth_rate: Decimal,
}

/// 비용 요약
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseSummary {
    pub total_expenses: Decimal,
    pub cost_of_goods_sold: Decimal,
    pub operating_expenses: Decimal,
    pub marketing_expenses: Decimal,
    pub administrative_expenses: Decimal,
}

/// 수익성 분석
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitAnalysis {
    pub gross_profit: Decimal,
    pub net_profit: Decimal,
    pub operating_profit: Decimal,
    pub gross_margin: Decimal,
    pub net_margin: Decimal,
    pub operating_margin: Decimal,
}

/// 현금 흐름 분석
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowAnalysis {
    pub cash_inflow: Decimal,
    pub cash_outflow: Decimal,
    pub net_cash_flow: Decimal,
    pub operating_cash_flow: Decimal,
    pub investing_cash_flow: Decimal,
    pub financing_cash_flow: Decimal,
}

/// 결제 분석
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAnalytics {
    pub payments_received: Decimal,
    pub outstanding_receivables: Decimal,
    pub overdue_payments: Decimal,
    pub average_payment_terms_days: u32,
    pub payment_methods_breakdown: Vec<PaymentMethodBreakdown>,
}

/// 결제 방법별 분석
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodBreakdown {
    pub payment_method: String,
    pub transaction_count: u32,
    pub total_amount: Decimal,
    pub percentage_of_total: Decimal,
}

/// 재무 비율
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialRatios {
    pub current_ratio: Decimal,
    pub quick_ratio: Decimal,
    pub debt_to_equity_ratio: Decimal,
    pub return_on_investment: Decimal,
    pub inventory_turnover: Decimal,
    pub receivables_turnover: Decimal,
}

/// 보고서 생성 요청
#[derive(Debug, Clone)]
pub struct ReportRequest {
    pub report_type: ReportType,
    pub period: ReportPeriod,
    pub format: ReportFormat,
    pub output_path: Option<String>,
    pub filters: ReportFilters,
}

/// 보고서 타입
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportType {
    SalesSummary,
    InventoryStatus,
    CustomerAnalysis { months: u32 },
    FinancialOverview,
}

/// 보고서 필터
#[derive(Debug, Clone, Default)]
pub struct ReportFilters {
    pub customer_ids: Option<Vec<uuid::Uuid>>,
    pub product_ids: Option<Vec<uuid::Uuid>>,
    pub categories: Option<Vec<String>>,
    pub order_statuses: Option<Vec<String>>,
    pub payment_statuses: Option<Vec<String>>,
    pub low_stock_only: bool,
    pub include_inactive: bool,
}

impl ReportPeriod {
    /// 기간을 시작일과 종료일로 변환
    pub fn to_date_range(&self) -> (NaiveDate, NaiveDate) {
        let today = Utc::now().date_naive();
        match self {
            ReportPeriod::Daily => (today, today),
            ReportPeriod::Weekly => {
                let start = today - chrono::Duration::days(7);
                (start, today)
            }
            ReportPeriod::Monthly => {
                let start = today - chrono::Duration::days(30);
                (start, today)
            }
            ReportPeriod::Quarterly => {
                let start = today - chrono::Duration::days(90);
                (start, today)
            }
            ReportPeriod::Yearly => {
                let start = today - chrono::Duration::days(365);
                (start, today)
            }
            ReportPeriod::Custom { from, to } => (*from, *to),
        }
    }
}

impl std::str::FromStr for ReportFormat {
    type Err = crate::utils::error::ErpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "console" => Ok(ReportFormat::Console),
            "json" => Ok(ReportFormat::Json),
            "csv" => Ok(ReportFormat::Csv),
            "html" => Ok(ReportFormat::Html),
            "pdf" => Ok(ReportFormat::Pdf),
            _ => Err(crate::utils::error::ErpError::validation(
                "format",
                "지원되지 않는 보고서 형식입니다. 사용 가능한 형식: console, json, csv, html, pdf",
            )),
        }
    }
}

impl std::str::FromStr for ReportPeriod {
    type Err = crate::utils::error::ErpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(ReportPeriod::Daily),
            "weekly" => Ok(ReportPeriod::Weekly),
            "monthly" => Ok(ReportPeriod::Monthly),
            "quarterly" => Ok(ReportPeriod::Quarterly),
            "yearly" => Ok(ReportPeriod::Yearly),
            _ => Err(crate::utils::error::ErpError::validation(
                "period",
                "지원되지 않는 기간입니다. 사용 가능한 기간: daily, weekly, monthly, quarterly, yearly",
            )),
        }
    }
}

// 테스트용 Mock 데이터 생성 함수들
#[cfg(test)]
impl SalesSummaryReport {
    pub fn mock() -> Self {
        Self {
            period: ReportPeriod::Monthly,
            generated_at: Utc::now(),
            total_orders: 150,
            total_revenue: Decimal::new(50000, 2),
            total_items_sold: 500,
            average_order_value: Decimal::new(33333, 2),
            top_selling_products: vec![TopSellingProduct::mock()],
            sales_by_status: vec![SalesByStatus::mock()],
            daily_sales: vec![DailySales::mock()],
        }
    }
}

#[cfg(test)]
impl TopSellingProduct {
    pub fn mock() -> Self {
        Self {
            product_id: uuid::Uuid::new_v4(),
            name: "Test Product".to_string(),
            sku: "TEST-001".to_string(),
            quantity_sold: 100,
            total_revenue: Decimal::new(10000, 2),
        }
    }
}

#[cfg(test)]
impl SalesByStatus {
    pub fn mock() -> Self {
        Self {
            status: "Delivered".to_string(),
            order_count: 120,
            total_amount: Decimal::new(40000, 2),
        }
    }
}

#[cfg(test)]
impl DailySales {
    pub fn mock() -> Self {
        Self {
            date: Utc::now().date_naive(),
            order_count: 5,
            total_amount: Decimal::new(1000, 2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_period_to_date_range() {
        let period = ReportPeriod::Daily;
        let (start, end) = period.to_date_range();
        assert_eq!(start, end);

        let custom_period = ReportPeriod::Custom {
            from: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        };
        let (start, end) = custom_period.to_date_range();
        assert_eq!(start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    fn test_report_format_from_str() {
        assert_eq!("json".parse::<ReportFormat>().unwrap(), ReportFormat::Json);
        assert_eq!("csv".parse::<ReportFormat>().unwrap(), ReportFormat::Csv);
        assert!("invalid".parse::<ReportFormat>().is_err());
    }

    #[test]
    fn test_report_period_from_str() {
        assert_eq!(
            "monthly".parse::<ReportPeriod>().unwrap(),
            ReportPeriod::Monthly
        );
        assert_eq!(
            "yearly".parse::<ReportPeriod>().unwrap(),
            ReportPeriod::Yearly
        );
        assert!("invalid".parse::<ReportPeriod>().is_err());
    }
}
