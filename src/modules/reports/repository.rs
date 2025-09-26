use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::models::*;
use crate::utils::error::{ErpError, ErpResult};

#[async_trait]
pub trait ReportsRepository: Send + Sync {
    // 매출 요약 관련
    async fn get_sales_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<SalesSummaryReport>;

    async fn get_top_selling_products(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        limit: u32,
    ) -> ErpResult<Vec<TopSellingProduct>>;

    async fn get_sales_by_status(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<Vec<SalesByStatus>>;

    async fn get_daily_sales(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<Vec<DailySales>>;

    // 재고 상태 관련
    async fn get_inventory_status(&self) -> ErpResult<InventoryStatusReport>;

    async fn get_low_stock_items(&self, threshold_multiplier: f64) -> ErpResult<Vec<LowStockItem>>;

    async fn get_out_of_stock_items(&self) -> ErpResult<Vec<OutOfStockItem>>;

    async fn get_inventory_by_category(&self) -> ErpResult<Vec<InventoryByCategory>>;

    async fn get_stock_movements(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<Vec<StockMovement>>;

    // 고객 분석 관련
    async fn get_customer_analysis(&self, months: u32) -> ErpResult<CustomerAnalysisReport>;

    async fn get_top_customers(&self, months: u32, limit: u32) -> ErpResult<Vec<TopCustomer>>;

    async fn get_customer_segments(&self, months: u32) -> ErpResult<Vec<CustomerSegment>>;

    async fn get_geographic_distribution(
        &self,
        months: u32,
    ) -> ErpResult<Vec<GeographicDistribution>>;

    async fn get_customer_lifecycle_metrics(
        &self,
        months: u32,
    ) -> ErpResult<CustomerLifecycleMetrics>;

    // 재무 개요 관련
    async fn get_financial_overview(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<FinancialOverviewReport>;

    async fn get_revenue_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<RevenueSummary>;

    async fn get_expense_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<ExpenseSummary>;

    async fn get_payment_analytics(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<PaymentAnalytics>;
}

#[cfg(feature = "database")]
pub struct PostgresReportsRepository {
    pool: PgPool,
}

#[cfg(feature = "database")]
impl PostgresReportsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "database")]
#[async_trait]
impl ReportsRepository for PostgresReportsRepository {
    async fn get_sales_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<SalesSummaryReport> {
        let total_orders_query = sqlx::query!(
            "SELECT COUNT(*) as count FROM sales_orders WHERE DATE(created_at) BETWEEN $1 AND $2",
            start_date,
            end_date
        );

        let revenue_query = sqlx::query!(
            "SELECT COALESCE(SUM(total_amount), 0) as total FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2 AND status IN ('delivered', 'completed')",
            start_date,
            end_date
        );

        let items_sold_query = sqlx::query!(
            "SELECT COALESCE(SUM(soi.quantity), 0) as total
             FROM sales_order_items soi
             JOIN sales_orders so ON soi.order_id = so.id
             WHERE DATE(so.created_at) BETWEEN $1 AND $2",
            start_date,
            end_date
        );

        let total_orders = total_orders_query.fetch_one(&self.pool).await?;
        let revenue = revenue_query.fetch_one(&self.pool).await?;
        let items_sold = items_sold_query.fetch_one(&self.pool).await?;

        let total_orders = total_orders.count.unwrap_or(0) as u32;
        let total_revenue = revenue.total.unwrap_or(Decimal::ZERO);
        let total_items_sold = items_sold.total.unwrap_or(0) as u32;

        let average_order_value = if total_orders > 0 {
            total_revenue / Decimal::from(total_orders)
        } else {
            Decimal::ZERO
        };

        let top_selling_products = self
            .get_top_selling_products(start_date, end_date, 10)
            .await?;
        let sales_by_status = self.get_sales_by_status(start_date, end_date).await?;
        let daily_sales = self.get_daily_sales(start_date, end_date).await?;

        Ok(SalesSummaryReport {
            period: ReportPeriod::Custom {
                from: start_date,
                to: end_date,
            },
            generated_at: Utc::now(),
            total_orders,
            total_revenue,
            total_items_sold,
            average_order_value,
            top_selling_products,
            sales_by_status,
            daily_sales,
        })
    }

    async fn get_top_selling_products(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        limit: u32,
    ) -> ErpResult<Vec<TopSellingProduct>> {
        let query = sqlx::query!(
            "SELECT p.id, p.name, p.sku,
                    COALESCE(SUM(soi.quantity), 0) as quantity_sold,
                    COALESCE(SUM(soi.unit_price * soi.quantity), 0) as total_revenue
             FROM products p
             LEFT JOIN sales_order_items soi ON p.id = soi.product_id
             LEFT JOIN sales_orders so ON soi.order_id = so.id
             WHERE DATE(so.created_at) BETWEEN $1 AND $2
             GROUP BY p.id, p.name, p.sku
             ORDER BY quantity_sold DESC
             LIMIT $3",
            start_date,
            end_date,
            limit as i64
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut products = Vec::new();

        for row in rows {
            products.push(TopSellingProduct {
                product_id: row.id,
                name: row.name,
                sku: row.sku,
                quantity_sold: row.quantity_sold.unwrap_or(0) as u32,
                total_revenue: row.total_revenue.unwrap_or(Decimal::ZERO),
            });
        }

        Ok(products)
    }

    async fn get_sales_by_status(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<Vec<SalesByStatus>> {
        let query = sqlx::query!(
            "SELECT status, COUNT(*) as order_count, COALESCE(SUM(total_amount), 0) as total_amount
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2
             GROUP BY status
             ORDER BY order_count DESC",
            start_date,
            end_date
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut sales_by_status = Vec::new();

        for row in rows {
            sales_by_status.push(SalesByStatus {
                status: row.status,
                order_count: row.order_count.unwrap_or(0) as u32,
                total_amount: row.total_amount.unwrap_or(Decimal::ZERO),
            });
        }

        Ok(sales_by_status)
    }

    async fn get_daily_sales(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<Vec<DailySales>> {
        let query = sqlx::query!(
            "SELECT DATE(created_at) as date,
                    COUNT(*) as order_count,
                    COALESCE(SUM(total_amount), 0) as total_amount
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2
             GROUP BY DATE(created_at)
             ORDER BY date",
            start_date,
            end_date
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut daily_sales = Vec::new();

        for row in rows {
            daily_sales.push(DailySales {
                date: row.date.unwrap_or(start_date),
                order_count: row.order_count.unwrap_or(0) as u32,
                total_amount: row.total_amount.unwrap_or(Decimal::ZERO),
            });
        }

        Ok(daily_sales)
    }

    async fn get_inventory_status(&self) -> ErpResult<InventoryStatusReport> {
        let total_products_query =
            sqlx::query!("SELECT COUNT(*) as count FROM products WHERE active = true");
        let total_value_query = sqlx::query!(
            "SELECT COALESCE(SUM(current_stock * unit_price), 0) as total_value FROM products WHERE active = true"
        );

        let total_products = total_products_query.fetch_one(&self.pool).await?;
        let total_value = total_value_query.fetch_one(&self.pool).await?;

        let low_stock_items = self.get_low_stock_items(1.0).await?;
        let out_of_stock_items = self.get_out_of_stock_items().await?;
        let inventory_by_category = self.get_inventory_by_category().await?;

        // 최근 30일간의 재고 이동
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days(30);
        let stock_movements = self.get_stock_movements(start_date, end_date).await?;

        Ok(InventoryStatusReport {
            generated_at: Utc::now(),
            total_products: total_products.count.unwrap_or(0) as u32,
            total_stock_value: total_value.total_value.unwrap_or(Decimal::ZERO),
            low_stock_items,
            out_of_stock_items,
            inventory_by_category,
            stock_movements,
        })
    }

    async fn get_low_stock_items(&self, threshold_multiplier: f64) -> ErpResult<Vec<LowStockItem>> {
        let query = sqlx::query!(
            "SELECT id, name, sku, current_stock, reorder_level, reorder_quantity, unit_price
             FROM products
             WHERE active = true AND current_stock <= (reorder_level * $1)
             ORDER BY (current_stock::float / NULLIF(reorder_level, 0)) ASC",
            threshold_multiplier
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut low_stock_items = Vec::new();

        for row in rows {
            let stock_value = Decimal::from(row.current_stock) * row.unit_price;

            low_stock_items.push(LowStockItem {
                product_id: row.id,
                name: row.name,
                sku: row.sku,
                current_stock: row.current_stock as u32,
                reorder_level: row.reorder_level as u32,
                suggested_reorder_quantity: row.reorder_quantity as u32,
                stock_value,
            });
        }

        Ok(low_stock_items)
    }

    async fn get_out_of_stock_items(&self) -> ErpResult<Vec<OutOfStockItem>> {
        let query = sqlx::query!(
            "SELECT id, name, sku, updated_at
             FROM products
             WHERE active = true AND current_stock = 0"
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut out_of_stock_items = Vec::new();

        for row in rows {
            // 해당 제품에 대한 보류 중인 주문 수 계산
            let pending_orders_query = sqlx::query!(
                "SELECT COALESCE(SUM(soi.quantity), 0) as pending_quantity
                 FROM sales_order_items soi
                 JOIN sales_orders so ON soi.order_id = so.id
                 WHERE soi.product_id = $1 AND so.status IN ('draft', 'pending', 'confirmed')",
                row.id
            );

            let pending_result = pending_orders_query.fetch_one(&self.pool).await?;

            out_of_stock_items.push(OutOfStockItem {
                product_id: row.id,
                name: row.name,
                sku: row.sku,
                last_stock_date: row.updated_at.map(|dt| dt.date_naive()),
                pending_orders: pending_result.pending_quantity.unwrap_or(0) as u32,
            });
        }

        Ok(out_of_stock_items)
    }

    async fn get_inventory_by_category(&self) -> ErpResult<Vec<InventoryByCategory>> {
        let query = sqlx::query!(
            "SELECT category,
                    COUNT(*) as product_count,
                    COALESCE(SUM(current_stock), 0) as total_stock,
                    COALESCE(SUM(current_stock * unit_price), 0) as total_value
             FROM products
             WHERE active = true
             GROUP BY category
             ORDER BY total_value DESC"
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut inventory_by_category = Vec::new();

        for row in rows {
            let product_count = row.product_count.unwrap_or(0) as u32;
            let total_stock = row.total_stock.unwrap_or(0) as u32;
            let average_stock_per_product = if product_count > 0 {
                Decimal::from(total_stock) / Decimal::from(product_count)
            } else {
                Decimal::ZERO
            };

            inventory_by_category.push(InventoryByCategory {
                category: row.category,
                product_count,
                total_stock,
                total_value: row.total_value.unwrap_or(Decimal::ZERO),
                average_stock_per_product,
            });
        }

        Ok(inventory_by_category)
    }

    async fn get_stock_movements(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<Vec<StockMovement>> {
        // 주문으로 인한 재고 감소 (Out)
        let outgoing_query = sqlx::query!(
            "SELECT DATE(so.created_at) as date,
                    'Out' as movement_type,
                    -SUM(soi.quantity) as total_quantity,
                    -SUM(soi.quantity * soi.unit_price) as total_value
             FROM sales_orders so
             JOIN sales_order_items soi ON so.id = soi.order_id
             WHERE DATE(so.created_at) BETWEEN $1 AND $2
             GROUP BY DATE(so.created_at)
             ORDER BY date",
            start_date,
            end_date
        );

        let outgoing_rows = outgoing_query.fetch_all(&self.pool).await?;
        let mut stock_movements = Vec::new();

        for row in outgoing_rows {
            stock_movements.push(StockMovement {
                date: row.date.unwrap_or(start_date),
                movement_type: row.movement_type,
                total_quantity: row.total_quantity.unwrap_or(0),
                total_value: row.total_value.unwrap_or(Decimal::ZERO),
            });
        }

        // TODO: 실제 구현에서는 재고 입고, 조정 등의 이력을 별도 테이블에서 추적해야 함
        // 현재는 주문에 의한 출고만 추적

        Ok(stock_movements)
    }

    async fn get_customer_analysis(&self, months: u32) -> ErpResult<CustomerAnalysisReport> {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days((months * 30) as i64);

        let total_customers_query = sqlx::query!("SELECT COUNT(*) as count FROM customers");
        let active_customers_query = sqlx::query!(
            "SELECT COUNT(DISTINCT customer_id) as count FROM sales_orders WHERE DATE(created_at) >= $1",
            start_date
        );
        let new_customers_query = sqlx::query!(
            "SELECT COUNT(*) as count FROM customers WHERE DATE(created_at) >= $1",
            start_date
        );

        let total_customers = total_customers_query.fetch_one(&self.pool).await?;
        let active_customers = active_customers_query.fetch_one(&self.pool).await?;
        let new_customers = new_customers_query.fetch_one(&self.pool).await?;

        let top_customers = self.get_top_customers(months, 10).await?;
        let customer_segments = self.get_customer_segments(months).await?;
        let geographic_distribution = self.get_geographic_distribution(months).await?;
        let customer_lifecycle = self.get_customer_lifecycle_metrics(months).await?;

        Ok(CustomerAnalysisReport {
            analysis_period_months: months,
            generated_at: Utc::now(),
            total_customers: total_customers.count.unwrap_or(0) as u32,
            active_customers: active_customers.count.unwrap_or(0) as u32,
            new_customers: new_customers.count.unwrap_or(0) as u32,
            top_customers,
            customer_segments,
            geographic_distribution,
            customer_lifecycle,
        })
    }

    async fn get_top_customers(&self, months: u32, limit: u32) -> ErpResult<Vec<TopCustomer>> {
        let start_date = Utc::now().date_naive() - chrono::Duration::days((months * 30) as i64);

        let query = sqlx::query!(
            "SELECT c.id, c.name, c.email,
                    COUNT(so.id) as total_orders,
                    COALESCE(SUM(so.total_amount), 0) as total_spent,
                    MAX(so.created_at) as last_order_date
             FROM customers c
             LEFT JOIN sales_orders so ON c.id = so.customer_id
             WHERE DATE(so.created_at) >= $1
             GROUP BY c.id, c.name, c.email
             ORDER BY total_spent DESC
             LIMIT $2",
            start_date,
            limit as i64
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut top_customers = Vec::new();

        for row in rows {
            let total_orders = row.total_orders.unwrap_or(0) as u32;
            let total_spent = row.total_spent.unwrap_or(Decimal::ZERO);
            let average_order_value = if total_orders > 0 {
                total_spent / Decimal::from(total_orders)
            } else {
                Decimal::ZERO
            };

            top_customers.push(TopCustomer {
                customer_id: row.id,
                name: row.name,
                email: row.email,
                total_orders,
                total_spent,
                average_order_value,
                last_order_date: row.last_order_date.map(|dt| dt.date_naive()),
            });
        }

        Ok(top_customers)
    }

    async fn get_customer_segments(&self, months: u32) -> ErpResult<Vec<CustomerSegment>> {
        // 간단한 세그멘테이션: 고객 유형별
        let start_date = Utc::now().date_naive() - chrono::Duration::days((months * 30) as i64);

        let query = sqlx::query!(
            "SELECT c.customer_type as segment_name,
                    COUNT(DISTINCT c.id) as customer_count,
                    COALESCE(SUM(so.total_amount), 0) as total_revenue,
                    COALESCE(AVG(order_frequency.freq), 0) as avg_frequency
             FROM customers c
             LEFT JOIN sales_orders so ON c.id = so.customer_id AND DATE(so.created_at) >= $1
             LEFT JOIN (
                 SELECT customer_id, COUNT(*) as freq
                 FROM sales_orders
                 WHERE DATE(created_at) >= $1
                 GROUP BY customer_id
             ) order_frequency ON c.id = order_frequency.customer_id
             GROUP BY c.customer_type",
            start_date
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut segments = Vec::new();

        for row in rows {
            segments.push(CustomerSegment {
                segment_name: row.segment_name,
                customer_count: row.customer_count.unwrap_or(0) as u32,
                total_revenue: row.total_revenue.unwrap_or(Decimal::ZERO),
                average_order_frequency: row.avg_frequency.unwrap_or(0.0).into(),
            });
        }

        Ok(segments)
    }

    async fn get_geographic_distribution(
        &self,
        months: u32,
    ) -> ErpResult<Vec<GeographicDistribution>> {
        let start_date = Utc::now().date_naive() - chrono::Duration::days((months * 30) as i64);

        let query = sqlx::query!(
            "SELECT ca.country, ca.state_province, ca.city,
                    COUNT(DISTINCT c.id) as customer_count,
                    COALESCE(SUM(so.total_amount), 0) as total_revenue
             FROM customers c
             LEFT JOIN customer_addresses ca ON c.id = ca.customer_id AND ca.address_type = 'billing'
             LEFT JOIN sales_orders so ON c.id = so.customer_id AND DATE(so.created_at) >= $1
             GROUP BY ca.country, ca.state_province, ca.city
             ORDER BY total_revenue DESC",
            start_date
        );

        let rows = query.fetch_all(&self.pool).await?;
        let mut distribution = Vec::new();

        for row in rows {
            distribution.push(GeographicDistribution {
                country: row.country.unwrap_or_else(|| "Unknown".to_string()),
                state_province: row.state_province,
                city: row.city,
                customer_count: row.customer_count.unwrap_or(0) as u32,
                total_revenue: row.total_revenue.unwrap_or(Decimal::ZERO),
            });
        }

        Ok(distribution)
    }

    async fn get_customer_lifecycle_metrics(
        &self,
        months: u32,
    ) -> ErpResult<CustomerLifecycleMetrics> {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days((months * 30) as i64);

        let new_customers_query = sqlx::query!(
            "SELECT COUNT(*) as count FROM customers WHERE DATE(created_at) >= $1",
            start_date
        );

        let returning_customers_query = sqlx::query!(
            "SELECT COUNT(DISTINCT customer_id) as count
             FROM sales_orders
             WHERE DATE(created_at) >= $1
             AND customer_id IN (
                 SELECT customer_id FROM sales_orders WHERE DATE(created_at) < $1
             )",
            start_date
        );

        let new_customers = new_customers_query.fetch_one(&self.pool).await?;
        let returning_customers = returning_customers_query.fetch_one(&self.pool).await?;

        // 단순화된 메트릭 계산
        let new_count = new_customers.count.unwrap_or(0) as u32;
        let returning_count = returning_customers.count.unwrap_or(0) as u32;

        let total_active = new_count + returning_count;
        let churn_rate = if total_active > 0 {
            Decimal::from(100)
                - (Decimal::from(returning_count * 100) / Decimal::from(total_active))
        } else {
            Decimal::ZERO
        };

        Ok(CustomerLifecycleMetrics {
            new_customers: new_count,
            returning_customers: returning_count,
            churn_rate,
            customer_lifetime_value: Decimal::new(50000, 2), // 가상 값 - 실제로는 복잡한 계산 필요
            average_customer_lifespan_days: 365,             // 가상 값
        })
    }

    async fn get_financial_overview(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<FinancialOverviewReport> {
        let revenue_summary = self.get_revenue_summary(start_date, end_date).await?;
        let expense_summary = self.get_expense_summary(start_date, end_date).await?;
        let payment_analytics = self.get_payment_analytics(start_date, end_date).await?;

        let profit_analysis = ProfitAnalysis {
            gross_profit: revenue_summary.total_revenue - expense_summary.cost_of_goods_sold,
            net_profit: revenue_summary.total_revenue - expense_summary.total_expenses,
            operating_profit: revenue_summary.total_revenue - expense_summary.operating_expenses,
            gross_margin: if revenue_summary.total_revenue > Decimal::ZERO {
                ((revenue_summary.total_revenue - expense_summary.cost_of_goods_sold)
                    * Decimal::from(100))
                    / revenue_summary.total_revenue
            } else {
                Decimal::ZERO
            },
            net_margin: if revenue_summary.total_revenue > Decimal::ZERO {
                ((revenue_summary.total_revenue - expense_summary.total_expenses)
                    * Decimal::from(100))
                    / revenue_summary.total_revenue
            } else {
                Decimal::ZERO
            },
            operating_margin: if revenue_summary.total_revenue > Decimal::ZERO {
                ((revenue_summary.total_revenue - expense_summary.operating_expenses)
                    * Decimal::from(100))
                    / revenue_summary.total_revenue
            } else {
                Decimal::ZERO
            },
        };

        let cash_flow = CashFlowAnalysis {
            cash_inflow: payment_analytics.payments_received,
            cash_outflow: expense_summary.total_expenses,
            net_cash_flow: payment_analytics.payments_received - expense_summary.total_expenses,
            operating_cash_flow: revenue_summary.total_revenue - expense_summary.operating_expenses,
            investing_cash_flow: Decimal::ZERO, // 투자 활동으로 인한 현금흐름 - 별도 추적 필요
            financing_cash_flow: Decimal::ZERO, // 재무 활동으로 인한 현금흐름 - 별도 추적 필요
        };

        let financial_ratios = FinancialRatios {
            current_ratio: Decimal::new(150, 2),       // 1.5 - 가상 값
            quick_ratio: Decimal::new(125, 2),         // 1.25 - 가상 값
            debt_to_equity_ratio: Decimal::new(50, 2), // 0.5 - 가상 값
            return_on_investment: if revenue_summary.total_revenue > Decimal::ZERO {
                (profit_analysis.net_profit * Decimal::from(100)) / revenue_summary.total_revenue
            } else {
                Decimal::ZERO
            },
            inventory_turnover: Decimal::new(6, 0), // 6 - 가상 값
            receivables_turnover: Decimal::new(12, 0), // 12 - 가상 값
        };

        Ok(FinancialOverviewReport {
            period: ReportPeriod::Custom {
                from: start_date,
                to: end_date,
            },
            generated_at: Utc::now(),
            revenue_summary,
            expense_summary,
            profit_analysis,
            cash_flow,
            payment_analytics,
            financial_ratios,
        })
    }

    async fn get_revenue_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<RevenueSummary> {
        let total_revenue_query = sqlx::query!(
            "SELECT COALESCE(SUM(total_amount), 0) as total
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2
             AND status IN ('delivered', 'completed')",
            start_date,
            end_date
        );

        let total_revenue = total_revenue_query.fetch_one(&self.pool).await?;
        let total_revenue = total_revenue.total.unwrap_or(Decimal::ZERO);

        // 간단화된 수익 분류 - 실제로는 더 복잡한 분류가 필요
        Ok(RevenueSummary {
            total_revenue,
            product_revenue: total_revenue * Decimal::new(80, 2), // 80%
            service_revenue: total_revenue * Decimal::new(20, 2), // 20%
            recurring_revenue: total_revenue * Decimal::new(30, 2), // 30%
            one_time_revenue: total_revenue * Decimal::new(70, 2), // 70%
            revenue_growth_rate: Decimal::new(5, 0),              // 5% 성장률 가정
        })
    }

    async fn get_expense_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<ExpenseSummary> {
        // 비용 데이터는 별도 테이블에서 관리되어야 하지만,
        // 현재 스키마에서는 매출 데이터를 기반으로 추정
        let revenue_query = sqlx::query!(
            "SELECT COALESCE(SUM(total_amount), 0) as total
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2",
            start_date,
            end_date
        );

        let revenue = revenue_query.fetch_one(&self.pool).await?;
        let total_revenue = revenue.total.unwrap_or(Decimal::ZERO);

        // 일반적인 비용 비율로 추정
        let cost_of_goods_sold = total_revenue * Decimal::new(60, 2); // 60%
        let operating_expenses = total_revenue * Decimal::new(25, 2); // 25%
        let marketing_expenses = total_revenue * Decimal::new(10, 2); // 10%
        let administrative_expenses = total_revenue * Decimal::new(5, 2); // 5%
        let total_expenses =
            cost_of_goods_sold + operating_expenses + marketing_expenses + administrative_expenses;

        Ok(ExpenseSummary {
            total_expenses,
            cost_of_goods_sold,
            operating_expenses,
            marketing_expenses,
            administrative_expenses,
        })
    }

    async fn get_payment_analytics(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<PaymentAnalytics> {
        let payments_received_query = sqlx::query!(
            "SELECT COALESCE(SUM(total_amount), 0) as total
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2
             AND payment_status = 'paid'",
            start_date,
            end_date
        );

        let outstanding_query = sqlx::query!(
            "SELECT COALESCE(SUM(total_amount), 0) as total
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2
             AND payment_status IN ('pending', 'partially_paid')",
            start_date,
            end_date
        );

        let overdue_query = sqlx::query!(
            "SELECT COALESCE(SUM(total_amount), 0) as total
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2
             AND payment_status = 'overdue'",
            start_date,
            end_date
        );

        let payment_methods_query = sqlx::query!(
            "SELECT payment_method,
                    COUNT(*) as transaction_count,
                    COALESCE(SUM(total_amount), 0) as total_amount
             FROM sales_orders
             WHERE DATE(created_at) BETWEEN $1 AND $2
             AND payment_status = 'paid'
             GROUP BY payment_method",
            start_date,
            end_date
        );

        let payments_received = payments_received_query.fetch_one(&self.pool).await?;
        let outstanding = outstanding_query.fetch_one(&self.pool).await?;
        let overdue = overdue_query.fetch_one(&self.pool).await?;
        let payment_methods = payment_methods_query.fetch_all(&self.pool).await?;

        let payments_received_amount = payments_received.total.unwrap_or(Decimal::ZERO);
        let total_transactions: i64 = payment_methods
            .iter()
            .map(|row| row.transaction_count.unwrap_or(0))
            .sum();

        let mut payment_methods_breakdown = Vec::new();
        for row in payment_methods {
            let amount = row.total_amount.unwrap_or(Decimal::ZERO);
            let percentage = if payments_received_amount > Decimal::ZERO {
                (amount * Decimal::from(100)) / payments_received_amount
            } else {
                Decimal::ZERO
            };

            payment_methods_breakdown.push(PaymentMethodBreakdown {
                payment_method: row.payment_method,
                transaction_count: row.transaction_count.unwrap_or(0) as u32,
                total_amount: amount,
                percentage_of_total: percentage,
            });
        }

        Ok(PaymentAnalytics {
            payments_received: payments_received_amount,
            outstanding_receivables: outstanding.total.unwrap_or(Decimal::ZERO),
            overdue_payments: overdue.total.unwrap_or(Decimal::ZERO),
            average_payment_terms_days: 30, // 가정값
            payment_methods_breakdown,
        })
    }
}

// Mock 구현체
pub struct MockReportsRepository;

impl MockReportsRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ReportsRepository for MockReportsRepository {
    async fn get_sales_summary(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> ErpResult<SalesSummaryReport> {
        Ok(SalesSummaryReport {
            period: ReportPeriod::Custom {
                from: start_date,
                to: end_date,
            },
            generated_at: Utc::now(),
            total_orders: 100,
            total_revenue: Decimal::new(25000, 2),
            total_items_sold: 300,
            average_order_value: Decimal::new(25000, 4),
            top_selling_products: vec![TopSellingProduct {
                product_id: uuid::Uuid::new_v4(),
                name: "Mock Product".to_string(),
                sku: "MOCK-001".to_string(),
                quantity_sold: 50,
                total_revenue: Decimal::new(5000, 2),
            }],
            sales_by_status: vec![SalesByStatus {
                status: "delivered".to_string(),
                order_count: 80,
                total_amount: Decimal::new(20000, 2),
            }],
            daily_sales: vec![DailySales {
                date: Utc::now().date_naive(),
                order_count: 5,
                total_amount: Decimal::new(1250, 2),
            }],
        })
    }

    // Mock 구현체의 나머지 메서드들은 간단한 더미 데이터 반환
    async fn get_top_selling_products(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
        _limit: u32,
    ) -> ErpResult<Vec<TopSellingProduct>> {
        Ok(vec![TopSellingProduct {
            product_id: uuid::Uuid::new_v4(),
            name: "Mock Product".to_string(),
            sku: "MOCK-001".to_string(),
            quantity_sold: 50,
            total_revenue: Decimal::new(5000, 2),
        }])
    }

    async fn get_sales_by_status(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> ErpResult<Vec<SalesByStatus>> {
        Ok(vec![SalesByStatus {
            status: "delivered".to_string(),
            order_count: 80,
            total_amount: Decimal::new(20000, 2),
        }])
    }

    async fn get_daily_sales(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> ErpResult<Vec<DailySales>> {
        Ok(vec![DailySales {
            date: Utc::now().date_naive(),
            order_count: 5,
            total_amount: Decimal::new(1250, 2),
        }])
    }

    async fn get_inventory_status(&self) -> ErpResult<InventoryStatusReport> {
        Ok(InventoryStatusReport {
            generated_at: Utc::now(),
            total_products: 50,
            total_stock_value: Decimal::new(125000, 2),
            low_stock_items: vec![],
            out_of_stock_items: vec![],
            inventory_by_category: vec![],
            stock_movements: vec![],
        })
    }

    // 나머지 메서드들도 유사하게 더미 데이터 반환
    // (간결성을 위해 생략하고 필요시 구현)
    async fn get_low_stock_items(
        &self,
        _threshold_multiplier: f64,
    ) -> ErpResult<Vec<LowStockItem>> {
        Ok(vec![])
    }

    async fn get_out_of_stock_items(&self) -> ErpResult<Vec<OutOfStockItem>> {
        Ok(vec![])
    }

    async fn get_inventory_by_category(&self) -> ErpResult<Vec<InventoryByCategory>> {
        Ok(vec![])
    }

    async fn get_stock_movements(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> ErpResult<Vec<StockMovement>> {
        Ok(vec![])
    }

    async fn get_customer_analysis(&self, _months: u32) -> ErpResult<CustomerAnalysisReport> {
        Ok(CustomerAnalysisReport {
            analysis_period_months: 3,
            generated_at: Utc::now(),
            total_customers: 25,
            active_customers: 20,
            new_customers: 5,
            top_customers: vec![],
            customer_segments: vec![],
            geographic_distribution: vec![],
            customer_lifecycle: CustomerLifecycleMetrics {
                new_customers: 5,
                returning_customers: 15,
                churn_rate: Decimal::new(10, 0),
                customer_lifetime_value: Decimal::new(50000, 2),
                average_customer_lifespan_days: 365,
            },
        })
    }

    async fn get_top_customers(&self, _months: u32, _limit: u32) -> ErpResult<Vec<TopCustomer>> {
        Ok(vec![])
    }

    async fn get_customer_segments(&self, _months: u32) -> ErpResult<Vec<CustomerSegment>> {
        Ok(vec![])
    }

    async fn get_geographic_distribution(
        &self,
        _months: u32,
    ) -> ErpResult<Vec<GeographicDistribution>> {
        Ok(vec![])
    }

    async fn get_customer_lifecycle_metrics(
        &self,
        _months: u32,
    ) -> ErpResult<CustomerLifecycleMetrics> {
        Ok(CustomerLifecycleMetrics {
            new_customers: 5,
            returning_customers: 15,
            churn_rate: Decimal::new(10, 0),
            customer_lifetime_value: Decimal::new(50000, 2),
            average_customer_lifespan_days: 365,
        })
    }

    async fn get_financial_overview(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> ErpResult<FinancialOverviewReport> {
        Ok(FinancialOverviewReport {
            period: ReportPeriod::Monthly,
            generated_at: Utc::now(),
            revenue_summary: RevenueSummary {
                total_revenue: Decimal::new(100000, 2),
                product_revenue: Decimal::new(80000, 2),
                service_revenue: Decimal::new(20000, 2),
                recurring_revenue: Decimal::new(30000, 2),
                one_time_revenue: Decimal::new(70000, 2),
                revenue_growth_rate: Decimal::new(5, 0),
            },
            expense_summary: ExpenseSummary {
                total_expenses: Decimal::new(75000, 2),
                cost_of_goods_sold: Decimal::new(50000, 2),
                operating_expenses: Decimal::new(15000, 2),
                marketing_expenses: Decimal::new(7000, 2),
                administrative_expenses: Decimal::new(3000, 2),
            },
            profit_analysis: ProfitAnalysis {
                gross_profit: Decimal::new(50000, 2),
                net_profit: Decimal::new(25000, 2),
                operating_profit: Decimal::new(35000, 2),
                gross_margin: Decimal::new(50, 0),
                net_margin: Decimal::new(25, 0),
                operating_margin: Decimal::new(35, 0),
            },
            cash_flow: CashFlowAnalysis {
                cash_inflow: Decimal::new(95000, 2),
                cash_outflow: Decimal::new(75000, 2),
                net_cash_flow: Decimal::new(20000, 2),
                operating_cash_flow: Decimal::new(30000, 2),
                investing_cash_flow: Decimal::new(-5000, 2),
                financing_cash_flow: Decimal::new(-5000, 2),
            },
            payment_analytics: PaymentAnalytics {
                payments_received: Decimal::new(95000, 2),
                outstanding_receivables: Decimal::new(15000, 2),
                overdue_payments: Decimal::new(5000, 2),
                average_payment_terms_days: 30,
                payment_methods_breakdown: vec![],
            },
            financial_ratios: FinancialRatios {
                current_ratio: Decimal::new(150, 2),
                quick_ratio: Decimal::new(125, 2),
                debt_to_equity_ratio: Decimal::new(50, 2),
                return_on_investment: Decimal::new(15, 0),
                inventory_turnover: Decimal::new(6, 0),
                receivables_turnover: Decimal::new(12, 0),
            },
        })
    }

    async fn get_revenue_summary(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> ErpResult<RevenueSummary> {
        Ok(RevenueSummary {
            total_revenue: Decimal::new(100000, 2),
            product_revenue: Decimal::new(80000, 2),
            service_revenue: Decimal::new(20000, 2),
            recurring_revenue: Decimal::new(30000, 2),
            one_time_revenue: Decimal::new(70000, 2),
            revenue_growth_rate: Decimal::new(5, 0),
        })
    }

    async fn get_expense_summary(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> ErpResult<ExpenseSummary> {
        Ok(ExpenseSummary {
            total_expenses: Decimal::new(75000, 2),
            cost_of_goods_sold: Decimal::new(50000, 2),
            operating_expenses: Decimal::new(15000, 2),
            marketing_expenses: Decimal::new(7000, 2),
            administrative_expenses: Decimal::new(3000, 2),
        })
    }

    async fn get_payment_analytics(
        &self,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> ErpResult<PaymentAnalytics> {
        Ok(PaymentAnalytics {
            payments_received: Decimal::new(95000, 2),
            outstanding_receivables: Decimal::new(15000, 2),
            overdue_payments: Decimal::new(5000, 2),
            average_payment_terms_days: 30,
            payment_methods_breakdown: vec![],
        })
    }
}
