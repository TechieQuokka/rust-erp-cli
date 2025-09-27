use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::models::*;
use crate::utils::error::{ErpError, ErpResult};

#[async_trait]
pub trait SalesRepository: Send + Sync {
    async fn create_order(&self, order: &SalesOrder) -> ErpResult<()>;
    async fn create_order_items(&self, items: &[SalesOrderItem]) -> ErpResult<()>;
    async fn get_order_by_id(&self, id: Uuid) -> ErpResult<Option<SalesOrder>>;
    async fn get_order_by_number(&self, order_number: &str) -> ErpResult<Option<SalesOrder>>;
    async fn get_order_items(&self, order_id: Uuid) -> ErpResult<Vec<SalesOrderItem>>;
    async fn get_order_items_with_products(
        &self,
        order_id: Uuid,
    ) -> ErpResult<Vec<OrderItemWithProduct>>;
    async fn update_order(&self, id: Uuid, updates: &UpdateOrderRequest) -> ErpResult<()>;
    async fn update_order_status(&self, id: Uuid, status: OrderStatus) -> ErpResult<()>;
    async fn update_payment_status(&self, id: Uuid, payment_status: PaymentStatus)
        -> ErpResult<()>;
    async fn delete_order(&self, id: Uuid) -> ErpResult<()>;
    async fn list_orders(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ErpResult<Vec<SalesOrder>>;
    async fn search_orders(
        &self,
        query: &str,
        status: Option<OrderStatus>,
    ) -> ErpResult<Vec<SalesOrder>>;
    async fn get_orders_by_customer(&self, customer_id: Uuid) -> ErpResult<Vec<SalesOrder>>;
    async fn get_orders_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ErpResult<Vec<SalesOrder>>;
    async fn get_sales_statistics(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> ErpResult<SalesStatistics>;
    async fn get_next_order_number(&self) -> ErpResult<String>;
    async fn calculate_order_totals(
        &self,
        order_id: Uuid,
    ) -> ErpResult<(Decimal, Decimal, Decimal)>;
}

pub struct PostgresSalesRepository {
    pool: PgPool,
}

impl PostgresSalesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SalesRepository for PostgresSalesRepository {
    async fn create_order(&self, order: &SalesOrder) -> ErpResult<()> {
        let query = r#"
            INSERT INTO sales_orders (
                id, order_number, customer_id, order_date, status, total_amount,
                tax_amount, discount_amount, shipping_address, billing_address,
                payment_method, payment_status, notes, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#;

        sqlx::query(query)
            .bind(&order.id)
            .bind(&order.order_number)
            .bind(&order.customer_id)
            .bind(&order.order_date)
            .bind(&order.status)
            .bind(&order.total_amount)
            .bind(&order.tax_amount)
            .bind(&order.discount_amount)
            .bind(&order.shipping_address)
            .bind(&order.billing_address)
            .bind(&order.payment_method)
            .bind(&order.payment_status)
            .bind(&order.notes)
            .bind(&order.created_at)
            .bind(&order.updated_at)
            .execute(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(())
    }

    async fn create_order_items(&self, items: &[SalesOrderItem]) -> ErpResult<()> {
        let query = r#"
            INSERT INTO sales_order_items (
                id, order_id, product_id, quantity, unit_price, discount, line_total, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#;

        for item in items {
            sqlx::query(query)
                .bind(&item.id)
                .bind(&item.order_id)
                .bind(&item.product_id)
                .bind(&item.quantity)
                .bind(&item.unit_price)
                .bind(&item.discount)
                .bind(&item.line_total)
                .bind(&item.created_at)
                .execute(&self.pool)
                .await
                .map_err(ErpError::Database)?;
        }

        Ok(())
    }

    async fn get_order_by_id(&self, id: Uuid) -> ErpResult<Option<SalesOrder>> {
        let query = r#"
            SELECT id, order_number, customer_id, order_date, status, total_amount,
                   tax_amount, discount_amount, shipping_address, billing_address,
                   payment_method, payment_status, notes, created_at, updated_at
            FROM sales_orders WHERE id = $1
        "#;

        let order = sqlx::query_as::<_, SalesOrder>(query)
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(order)
    }

    async fn get_order_by_number(&self, order_number: &str) -> ErpResult<Option<SalesOrder>> {
        let query = r#"
            SELECT id, order_number, customer_id, order_date, status, total_amount,
                   tax_amount, discount_amount, shipping_address, billing_address,
                   payment_method, payment_status, notes, created_at, updated_at
            FROM sales_orders WHERE order_number = $1
        "#;

        let order = sqlx::query_as::<_, SalesOrder>(query)
            .bind(order_number)
            .fetch_optional(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(order)
    }

    async fn get_order_items(&self, order_id: Uuid) -> ErpResult<Vec<SalesOrderItem>> {
        let query = r#"
            SELECT id, order_id, product_id, quantity, unit_price, discount, line_total, created_at
            FROM sales_order_items WHERE order_id = $1 ORDER BY created_at
        "#;

        let items = sqlx::query_as::<_, SalesOrderItem>(query)
            .bind(&order_id)
            .fetch_all(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(items)
    }

    async fn get_order_items_with_products(
        &self,
        order_id: Uuid,
    ) -> ErpResult<Vec<OrderItemWithProduct>> {
        let query = r#"
            SELECT soi.id, soi.order_id, soi.product_id, soi.quantity, soi.unit_price,
                   soi.discount, soi.line_total, soi.created_at,
                   p.name as product_name, p.sku as product_sku
            FROM sales_order_items soi
            JOIN products p ON soi.product_id = p.id
            WHERE soi.order_id = $1
            ORDER BY soi.created_at
        "#;

        let rows = sqlx::query(query)
            .bind(&order_id)
            .fetch_all(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        let items = rows
            .into_iter()
            .map(|row| OrderItemWithProduct {
                item: SalesOrderItem {
                    id: row.get("id"),
                    order_id: row.get("order_id"),
                    product_id: row.get("product_id"),
                    quantity: row.get("quantity"),
                    unit_price: row.get("unit_price"),
                    discount: row.get("discount"),
                    line_total: row.get("line_total"),
                    created_at: row.get("created_at"),
                },
                product_name: row.get("product_name"),
                product_sku: row.get("product_sku"),
            })
            .collect();

        Ok(items)
    }

    async fn update_order(&self, id: Uuid, updates: &UpdateOrderRequest) -> ErpResult<()> {
        let mut query_parts = Vec::new();
        let mut bind_count = 1;

        if updates.status.is_some() {
            query_parts.push(format!("status = ${}", bind_count));
            bind_count += 1;
        }
        if updates.payment_status.is_some() {
            query_parts.push(format!("payment_status = ${}", bind_count));
            bind_count += 1;
        }
        if updates.payment_method.is_some() {
            query_parts.push(format!("payment_method = ${}", bind_count));
            bind_count += 1;
        }
        if updates.shipping_address.is_some() {
            query_parts.push(format!("shipping_address = ${}", bind_count));
            bind_count += 1;
        }
        if updates.billing_address.is_some() {
            query_parts.push(format!("billing_address = ${}", bind_count));
            bind_count += 1;
        }
        if updates.notes.is_some() {
            query_parts.push(format!("notes = ${}", bind_count));
            bind_count += 1;
        }

        if query_parts.is_empty() {
            return Ok(());
        }

        query_parts.push(format!("updated_at = ${}", bind_count));
        let query = format!(
            "UPDATE sales_orders SET {} WHERE id = ${}",
            query_parts.join(", "),
            bind_count + 1
        );

        let mut sqlx_query = sqlx::query(&query);

        if let Some(status) = &updates.status {
            sqlx_query = sqlx_query.bind(status);
        }
        if let Some(payment_status) = &updates.payment_status {
            sqlx_query = sqlx_query.bind(payment_status);
        }
        if let Some(payment_method) = &updates.payment_method {
            sqlx_query = sqlx_query.bind(payment_method);
        }
        if let Some(shipping_address) = &updates.shipping_address {
            sqlx_query = sqlx_query.bind(shipping_address);
        }
        if let Some(billing_address) = &updates.billing_address {
            sqlx_query = sqlx_query.bind(billing_address);
        }
        if let Some(notes) = &updates.notes {
            sqlx_query = sqlx_query.bind(notes);
        }

        sqlx_query = sqlx_query.bind(Utc::now()).bind(&id);

        sqlx_query
            .execute(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(())
    }

    async fn update_order_status(&self, id: Uuid, status: OrderStatus) -> ErpResult<()> {
        let query = "UPDATE sales_orders SET status = $1, updated_at = $2 WHERE id = $3";

        sqlx::query(query)
            .bind(&status)
            .bind(Utc::now())
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(())
    }

    async fn update_payment_status(
        &self,
        id: Uuid,
        payment_status: PaymentStatus,
    ) -> ErpResult<()> {
        let query = "UPDATE sales_orders SET payment_status = $1, updated_at = $2 WHERE id = $3";

        sqlx::query(query)
            .bind(&payment_status)
            .bind(Utc::now())
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(())
    }

    async fn delete_order(&self, id: Uuid) -> ErpResult<()> {
        let mut tx = self.pool.begin().await.map_err(ErpError::Database)?;

        sqlx::query("DELETE FROM sales_order_items WHERE order_id = $1")
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(ErpError::Database)?;

        sqlx::query("DELETE FROM sales_orders WHERE id = $1")
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(ErpError::Database)?;

        tx.commit().await.map_err(ErpError::Database)?;

        Ok(())
    }

    async fn list_orders(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ErpResult<Vec<SalesOrder>> {
        let query = format!(
            r#"
            SELECT id, order_number, customer_id, order_date, status, total_amount,
                   tax_amount, discount_amount, shipping_address, billing_address,
                   payment_method, payment_status, notes, created_at, updated_at
            FROM sales_orders
            ORDER BY order_date DESC, created_at DESC
            {}
            "#,
            if limit.is_some() {
                format!(
                    "LIMIT {} OFFSET {}",
                    limit.unwrap_or(100),
                    offset.unwrap_or(0)
                )
            } else {
                String::new()
            }
        );

        let orders = sqlx::query_as::<_, SalesOrder>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(orders)
    }

    async fn search_orders(
        &self,
        query: &str,
        status: Option<OrderStatus>,
    ) -> ErpResult<Vec<SalesOrder>> {
        let search_query = if status.is_some() {
            r#"
            SELECT id, order_number, customer_id, order_date, status, total_amount,
                   tax_amount, discount_amount, shipping_address, billing_address,
                   payment_method, payment_status, notes, created_at, updated_at
            FROM sales_orders
            WHERE (order_number ILIKE $1 OR notes ILIKE $1) AND status = $2
            ORDER BY order_date DESC
            "#
        } else {
            r#"
            SELECT id, order_number, customer_id, order_date, status, total_amount,
                   tax_amount, discount_amount, shipping_address, billing_address,
                   payment_method, payment_status, notes, created_at, updated_at
            FROM sales_orders
            WHERE order_number ILIKE $1 OR notes ILIKE $1
            ORDER BY order_date DESC
            "#
        };

        let search_pattern = format!("%{}%", query);
        let mut sqlx_query = sqlx::query_as::<_, SalesOrder>(search_query).bind(&search_pattern);

        if let Some(status) = status {
            sqlx_query = sqlx_query.bind(status);
        }

        let orders = sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(orders)
    }

    async fn get_orders_by_customer(&self, customer_id: Uuid) -> ErpResult<Vec<SalesOrder>> {
        let query = r#"
            SELECT id, order_number, customer_id, order_date, status, total_amount,
                   tax_amount, discount_amount, shipping_address, billing_address,
                   payment_method, payment_status, notes, created_at, updated_at
            FROM sales_orders
            WHERE customer_id = $1
            ORDER BY order_date DESC
        "#;

        let orders = sqlx::query_as::<_, SalesOrder>(query)
            .bind(&customer_id)
            .fetch_all(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(orders)
    }

    async fn get_orders_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ErpResult<Vec<SalesOrder>> {
        let query = r#"
            SELECT id, order_number, customer_id, order_date, status, total_amount,
                   tax_amount, discount_amount, shipping_address, billing_address,
                   payment_method, payment_status, notes, created_at, updated_at
            FROM sales_orders
            WHERE order_date >= $1 AND order_date <= $2
            ORDER BY order_date DESC
        "#;

        let orders = sqlx::query_as::<_, SalesOrder>(query)
            .bind(&start_date)
            .bind(&end_date)
            .fetch_all(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        Ok(orders)
    }

    async fn get_sales_statistics(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> ErpResult<SalesStatistics> {
        let date_filter = match (start_date, end_date) {
            (Some(_), Some(_)) => "WHERE order_date >= $1 AND order_date <= $2",
            (Some(_), None) => "WHERE order_date >= $1",
            (None, Some(_)) => "WHERE order_date <= $1",
            (None, None) => "",
        };

        let stats_query = format!(
            r#"
            SELECT
                COUNT(*) as total_orders,
                COALESCE(SUM(total_amount), 0) as total_revenue,
                COALESCE(AVG(total_amount), 0) as average_order_value
            FROM sales_orders
            {}
            "#,
            date_filter
        );

        let mut query = sqlx::query(&stats_query);
        if let Some(start) = start_date {
            query = query.bind(start);
            if let Some(end) = end_date {
                query = query.bind(end);
            }
        } else if let Some(end) = end_date {
            query = query.bind(end);
        }

        let stats_row = query
            .fetch_one(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        let total_orders: i64 = stats_row.get("total_orders");
        let total_revenue: Decimal = stats_row.get("total_revenue");
        let average_order_value: Decimal = stats_row.get("average_order_value");

        let status_query = format!(
            r#"
            SELECT status, COUNT(*) as count
            FROM sales_orders
            {}
            GROUP BY status
            "#,
            date_filter
        );

        let mut status_query = sqlx::query(&status_query);
        if let Some(start) = start_date {
            status_query = status_query.bind(start);
            if let Some(end) = end_date {
                status_query = status_query.bind(end);
            }
        } else if let Some(end) = end_date {
            status_query = status_query.bind(end);
        }

        let status_rows = status_query
            .fetch_all(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        let orders_by_status = status_rows
            .into_iter()
            .map(|row| (row.get("status"), row.get("count")))
            .collect();

        Ok(SalesStatistics {
            total_orders,
            total_revenue,
            average_order_value,
            orders_by_status,
            top_customers: Vec::new(),
            top_products: Vec::new(),
        })
    }

    async fn get_next_order_number(&self) -> ErpResult<String> {
        let query = r#"
            SELECT order_number
            FROM sales_orders
            ORDER BY order_number DESC
            LIMIT 1
        "#;

        let result = sqlx::query(query)
            .fetch_optional(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        if let Some(row) = result {
            let last_number: String = row.get("order_number");
            if let Some(number_part) = last_number.strip_prefix("ORD-") {
                if let Ok(num) = number_part.parse::<u32>() {
                    return Ok(format!("ORD-{:06}", num + 1));
                }
            }
        }

        Ok("ORD-000001".to_string())
    }

    async fn calculate_order_totals(
        &self,
        order_id: Uuid,
    ) -> ErpResult<(Decimal, Decimal, Decimal)> {
        let query = r#"
            SELECT
                COALESCE(SUM(line_total), 0) as subtotal,
                COALESCE(SUM(discount), 0) as total_discount
            FROM sales_order_items
            WHERE order_id = $1
        "#;

        let row = sqlx::query(query)
            .bind(&order_id)
            .fetch_one(&self.pool)
            .await
            .map_err(ErpError::Database)?;

        let subtotal: Decimal = row.get("subtotal");
        let total_discount: Decimal = row.get("total_discount");
        let tax_rate = Decimal::new(10, 2);
        let tax_amount = subtotal * tax_rate / Decimal::from(100);

        Ok((subtotal, total_discount, tax_amount))
    }
}

pub struct MockSalesRepository {
    orders: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, SalesOrder>>>,
    items:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Vec<SalesOrderItem>>>>,
    order_counter: std::sync::Arc<tokio::sync::RwLock<u32>>,
}

impl MockSalesRepository {
    pub fn new() -> Self {
        Self {
            orders: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            items: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            order_counter: std::sync::Arc::new(tokio::sync::RwLock::new(1)),
        }
    }
}

impl Default for MockSalesRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SalesRepository for MockSalesRepository {
    async fn create_order(&self, order: &SalesOrder) -> ErpResult<()> {
        let mut orders = self.orders.write().await;
        orders.insert(order.id, order.clone());
        Ok(())
    }

    async fn create_order_items(&self, items: &[SalesOrderItem]) -> ErpResult<()> {
        let mut items_map = self.items.write().await;
        if let Some(first_item) = items.first() {
            items_map.insert(first_item.order_id, items.to_vec());
        }
        Ok(())
    }

    async fn get_order_by_id(&self, id: Uuid) -> ErpResult<Option<SalesOrder>> {
        let orders = self.orders.read().await;
        Ok(orders.get(&id).cloned())
    }

    async fn get_order_by_number(&self, order_number: &str) -> ErpResult<Option<SalesOrder>> {
        let orders = self.orders.read().await;
        Ok(orders
            .values()
            .find(|o| o.order_number == order_number)
            .cloned())
    }

    async fn get_order_items(&self, order_id: Uuid) -> ErpResult<Vec<SalesOrderItem>> {
        let items = self.items.read().await;
        Ok(items.get(&order_id).cloned().unwrap_or_default())
    }

    async fn get_order_items_with_products(
        &self,
        order_id: Uuid,
    ) -> ErpResult<Vec<OrderItemWithProduct>> {
        let items = self.get_order_items(order_id).await?;
        let items_with_products = items
            .into_iter()
            .map(|item| OrderItemWithProduct {
                item,
                product_name: "Mock Product".to_string(),
                product_sku: "MOCK-001".to_string(),
            })
            .collect();
        Ok(items_with_products)
    }

    async fn update_order(&self, id: Uuid, updates: &UpdateOrderRequest) -> ErpResult<()> {
        let mut orders = self.orders.write().await;
        if let Some(order) = orders.get_mut(&id) {
            if let Some(status) = updates.status {
                order.status = status;
            }
            if let Some(payment_status) = updates.payment_status {
                order.payment_status = payment_status;
            }
            if let Some(payment_method) = updates.payment_method {
                order.payment_method = Some(payment_method);
            }
            if let Some(shipping_address) = &updates.shipping_address {
                order.shipping_address = Some(shipping_address.clone());
            }
            if let Some(billing_address) = &updates.billing_address {
                order.billing_address = Some(billing_address.clone());
            }
            if let Some(notes) = &updates.notes {
                order.notes = Some(notes.clone());
            }
            order.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn update_order_status(&self, id: Uuid, status: OrderStatus) -> ErpResult<()> {
        let mut orders = self.orders.write().await;
        if let Some(order) = orders.get_mut(&id) {
            order.status = status;
            order.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn update_payment_status(
        &self,
        id: Uuid,
        payment_status: PaymentStatus,
    ) -> ErpResult<()> {
        let mut orders = self.orders.write().await;
        if let Some(order) = orders.get_mut(&id) {
            order.payment_status = payment_status;
            order.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn delete_order(&self, id: Uuid) -> ErpResult<()> {
        let mut orders = self.orders.write().await;
        let mut items = self.items.write().await;
        orders.remove(&id);
        items.remove(&id);
        Ok(())
    }

    async fn list_orders(
        &self,
        _limit: Option<i64>,
        _offset: Option<i64>,
    ) -> ErpResult<Vec<SalesOrder>> {
        let orders = self.orders.read().await;
        let mut order_list: Vec<SalesOrder> = orders.values().cloned().collect();
        order_list.sort_by(|a, b| b.order_date.cmp(&a.order_date));
        Ok(order_list)
    }

    async fn search_orders(
        &self,
        query: &str,
        status: Option<OrderStatus>,
    ) -> ErpResult<Vec<SalesOrder>> {
        let orders = self.orders.read().await;
        let filtered_orders: Vec<SalesOrder> = orders
            .values()
            .filter(|order| {
                let matches_query = order.order_number.contains(query)
                    || order.notes.as_ref().is_some_and(|n| n.contains(query));
                let matches_status = status.is_none_or(|s| order.status == s);
                matches_query && matches_status
            })
            .cloned()
            .collect();
        Ok(filtered_orders)
    }

    async fn get_orders_by_customer(&self, customer_id: Uuid) -> ErpResult<Vec<SalesOrder>> {
        let orders = self.orders.read().await;
        let customer_orders: Vec<SalesOrder> = orders
            .values()
            .filter(|order| order.customer_id == customer_id)
            .cloned()
            .collect();
        Ok(customer_orders)
    }

    async fn get_orders_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ErpResult<Vec<SalesOrder>> {
        let orders = self.orders.read().await;
        let filtered_orders: Vec<SalesOrder> = orders
            .values()
            .filter(|order| order.order_date >= start_date && order.order_date <= end_date)
            .cloned()
            .collect();
        Ok(filtered_orders)
    }

    async fn get_sales_statistics(
        &self,
        _start_date: Option<DateTime<Utc>>,
        _end_date: Option<DateTime<Utc>>,
    ) -> ErpResult<SalesStatistics> {
        let orders = self.orders.read().await;
        let total_orders = orders.len() as i64;
        let total_revenue: Decimal = orders.values().map(|o| o.total_amount).sum();
        let average_order_value = if total_orders > 0 {
            total_revenue / Decimal::from(total_orders)
        } else {
            Decimal::ZERO
        };

        Ok(SalesStatistics {
            total_orders,
            total_revenue,
            average_order_value,
            orders_by_status: Vec::new(),
            top_customers: Vec::new(),
            top_products: Vec::new(),
        })
    }

    async fn get_next_order_number(&self) -> ErpResult<String> {
        let mut counter = self.order_counter.write().await;
        let number = format!("ORD-{:06}", *counter);
        *counter += 1;
        Ok(number)
    }

    async fn calculate_order_totals(
        &self,
        order_id: Uuid,
    ) -> ErpResult<(Decimal, Decimal, Decimal)> {
        let items = self.get_order_items(order_id).await?;
        let subtotal: Decimal = items.iter().map(|i| i.line_total).sum();
        let total_discount: Decimal = items.iter().map(|i| i.discount).sum();
        let tax_amount = subtotal * Decimal::new(10, 2) / Decimal::from(100);
        Ok((subtotal, total_discount, tax_amount))
    }
}
