use crate::core::database::connection::DatabasePool;
use crate::core::database::models::product::{
    CreateProductRequest, Product, ProductStatus, StockMovement, UpdateProductRequest,
};
use crate::modules::inventory::models::{
    CategoryValuation, InventoryFilter, InventoryItem, InventoryValuation, LowStockAlert,
    StockAdjustmentRequest, StockMovementResponse,
};
use crate::utils::error::{ErpError, ErpResult};
use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn create_product(&self, request: CreateProductRequest) -> ErpResult<Product>;
    async fn get_product_by_id(&self, id: Uuid) -> ErpResult<Option<Product>>;
    async fn get_product_by_sku(&self, sku: &str) -> ErpResult<Option<Product>>;
    async fn list_products(&self, filter: &InventoryFilter)
        -> ErpResult<(Vec<InventoryItem>, i64)>;
    async fn update_product(&self, id: Uuid, request: UpdateProductRequest) -> ErpResult<Product>;
    async fn delete_product(&self, id: Uuid) -> ErpResult<()>;
    async fn adjust_stock(
        &self,
        request: StockAdjustmentRequest,
        user_id: Uuid,
    ) -> ErpResult<StockMovement>;
    async fn get_stock_movements(
        &self,
        product_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> ErpResult<Vec<StockMovementResponse>>;
    async fn get_low_stock_alerts(&self, threshold: Option<i32>) -> ErpResult<Vec<LowStockAlert>>;
    async fn get_inventory_valuation(&self) -> ErpResult<InventoryValuation>;
    async fn sku_exists(&self, sku: &str, exclude_id: Option<Uuid>) -> ErpResult<bool>;
    async fn get_inventory_by_category(&self) -> ErpResult<HashMap<String, i64>>;
    async fn get_products_by_status(&self, status: ProductStatus) -> ErpResult<Vec<Product>>;
}

pub struct PostgresInventoryRepository {
    pool: DatabasePool,
}

impl PostgresInventoryRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for PostgresInventoryRepository {
    async fn create_product(&self, request: CreateProductRequest) -> ErpResult<Product> {
        // Check if SKU already exists
        if self.sku_exists(&request.sku, None).await? {
            return Err(ErpError::conflict(&format!(
                "SKU '{}' already exists",
                request.sku
            )));
        }

        let product = Product::new(request);

        let query = r#"
            INSERT INTO products (
                id, sku, name, description, category, price, cost, quantity,
                min_stock_level, max_stock_level, status, is_taxable, weight,
                dimensions, barcode, supplier_id, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
        "#;

        sqlx::query(query)
            .bind(product.id)
            .bind(&product.sku)
            .bind(&product.name)
            .bind(&product.description)
            .bind(&product.category)
            .bind(product.price)
            .bind(product.cost)
            .bind(product.quantity)
            .bind(product.min_stock_level)
            .bind(product.max_stock_level)
            .bind(&product.status)
            .bind(product.is_taxable)
            .bind(product.weight)
            .bind(&product.dimensions)
            .bind(&product.barcode)
            .bind(product.supplier_id)
            .bind(product.created_at)
            .bind(product.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to create product: {}", e)))?;

        Ok(product)
    }

    async fn get_product_by_id(&self, id: Uuid) -> ErpResult<Option<Product>> {
        let query = "SELECT * FROM products WHERE id = $1";

        match sqlx::query_as::<_, Product>(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(product) => Ok(product),
            Err(e) => Err(ErpError::internal(format!(
                "Failed to get product by ID: {}",
                e
            ))),
        }
    }

    async fn get_product_by_sku(&self, sku: &str) -> ErpResult<Option<Product>> {
        let query = "SELECT * FROM products WHERE UPPER(sku) = UPPER($1)";

        match sqlx::query_as::<_, Product>(query)
            .bind(sku)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(product) => Ok(product),
            Err(e) => Err(ErpError::internal(format!(
                "Failed to get product by SKU: {}",
                e
            ))),
        }
    }

    async fn list_products(
        &self,
        filter: &InventoryFilter,
    ) -> ErpResult<(Vec<InventoryItem>, i64)> {
        let mut where_conditions = Vec::new();
        let mut query_params: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send + 'static>> =
            Vec::new();
        let mut param_count = 0;

        // Build WHERE clause dynamically
        if let Some(category) = &filter.category {
            param_count += 1;
            where_conditions.push(format!("category = ${}", param_count));
            query_params.push(Box::new(category.clone()));
        }

        if let Some(status) = &filter.status {
            param_count += 1;
            where_conditions.push(format!("status = ${}", param_count));
            query_params.push(Box::new(status.clone()));
        }

        if let Some(sku) = &filter.sku {
            param_count += 1;
            where_conditions.push(format!("UPPER(sku) LIKE UPPER(${})", param_count));
            query_params.push(Box::new(format!("%{}%", sku)));
        }

        if let Some(name) = &filter.name {
            param_count += 1;
            where_conditions.push(format!("UPPER(name) LIKE UPPER(${})", param_count));
            query_params.push(Box::new(format!("%{}%", name)));
        }

        if let Some(true) = filter.low_stock_only {
            where_conditions.push("quantity <= min_stock_level".to_string());
        }

        if let Some(min_quantity) = filter.min_quantity {
            param_count += 1;
            where_conditions.push(format!("quantity >= ${}", param_count));
            query_params.push(Box::new(min_quantity));
        }

        if let Some(max_quantity) = filter.max_quantity {
            param_count += 1;
            where_conditions.push(format!("quantity <= ${}", param_count));
            query_params.push(Box::new(max_quantity));
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Count query
        let count_query = format!("SELECT COUNT(*) FROM products {}", where_clause);

        let total: i64 = sqlx::query(&count_query)
            .execute(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to count products: {}", e)))?
            .rows_affected() as i64;

        // Main query with pagination
        let page = filter.page.unwrap_or(1);
        let limit = filter.limit.unwrap_or(20).min(100); // Cap at 100
        let offset = (page - 1) * limit;

        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        let main_query = format!(
            "SELECT * FROM products {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, limit_param, offset_param
        );

        // This is a simplified version - in practice, you'd need to properly bind all parameters
        let products = sqlx::query_as::<_, Product>(&main_query)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to fetch products: {}", e)))?;

        let inventory_items: Vec<InventoryItem> = products
            .into_iter()
            .map(InventoryItem::from_product)
            .collect();

        Ok((inventory_items, total))
    }

    async fn update_product(&self, id: Uuid, request: UpdateProductRequest) -> ErpResult<Product> {
        // Get current product
        let mut product = self.get_product_by_id(id).await?.ok_or_else(|| {
            ErpError::not_found_simple(format!("Product with ID {} not found", id))
        })?;

        // Update product
        product.update(request);

        // Save to database
        let query = r#"
            UPDATE products SET
                name = $2, description = $3, category = $4, price = $5, cost = $6,
                min_stock_level = $7, max_stock_level = $8, status = $9, is_taxable = $10,
                weight = $11, dimensions = $12, barcode = $13, supplier_id = $14,
                updated_at = $15
            WHERE id = $1
        "#;

        sqlx::query(query)
            .bind(id)
            .bind(&product.name)
            .bind(&product.description)
            .bind(&product.category)
            .bind(product.price)
            .bind(product.cost)
            .bind(product.min_stock_level)
            .bind(product.max_stock_level)
            .bind(&product.status)
            .bind(product.is_taxable)
            .bind(product.weight)
            .bind(&product.dimensions)
            .bind(&product.barcode)
            .bind(product.supplier_id)
            .bind(product.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to update product: {}", e)))?;

        Ok(product)
    }

    async fn delete_product(&self, id: Uuid) -> ErpResult<()> {
        // Check if product exists
        if self.get_product_by_id(id).await?.is_none() {
            return Err(ErpError::not_found_simple(format!(
                "Product with ID {} not found",
                id
            )));
        }

        // TODO: Check for references in orders, etc.
        // For now, we'll do a soft delete by setting status to Discontinued
        let query = "UPDATE products SET status = $1, updated_at = $2 WHERE id = $3";

        sqlx::query(query)
            .bind(ProductStatus::Discontinued)
            .bind(Utc::now())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to delete product: {}", e)))?;

        Ok(())
    }

    async fn adjust_stock(
        &self,
        request: StockAdjustmentRequest,
        user_id: Uuid,
    ) -> ErpResult<StockMovement> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ErpError::internal(format!("Failed to start transaction: {}", e)))?;

        // Get current product
        let mut product = self
            .get_product_by_id(request.product_id)
            .await?
            .ok_or_else(|| {
                ErpError::not_found_simple(format!(
                    "Product with ID {} not found",
                    request.product_id
                ))
            })?;

        let _previous_quantity = product.quantity;

        // Create stock movement with user_id
        let mut movement = product.adjust_quantity(request.quantity_change, request.reason.clone());
        movement.user_id = user_id;
        movement.reference_id = request.reference_id;

        // Update product quantity
        let update_query =
            "UPDATE products SET quantity = $1, status = $2, updated_at = $3 WHERE id = $4";

        sqlx::query(update_query)
            .bind(product.quantity)
            .bind(&product.status)
            .bind(product.updated_at)
            .bind(product.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to update product quantity: {}", e)))?;

        // Insert stock movement record
        let movement_query = r#"
            INSERT INTO stock_movements (
                id, product_id, movement_type, quantity, reason, reference_id, user_id, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#;

        sqlx::query(movement_query)
            .bind(movement.id)
            .bind(movement.product_id)
            .bind(&movement.movement_type)
            .bind(movement.quantity)
            .bind(&movement.reason)
            .bind(movement.reference_id)
            .bind(movement.user_id)
            .bind(movement.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to create stock movement: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| ErpError::internal(format!("Failed to commit transaction: {}", e)))?;

        Ok(movement)
    }

    async fn get_stock_movements(
        &self,
        product_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> ErpResult<Vec<StockMovementResponse>> {
        let limit = limit.unwrap_or(50).min(200); // Cap at 200

        let (_where_clause, query) = if let Some(_pid) = product_id {
            (
                "WHERE sm.product_id = $1",
                format!(
                    r#"
                    SELECT
                        sm.id, sm.product_id, sm.movement_type, sm.quantity, sm.reason,
                        sm.reference_id, sm.user_id, sm.created_at,
                        p.name as product_name, p.sku as product_sku
                    FROM stock_movements sm
                    JOIN products p ON sm.product_id = p.id
                    {} ORDER BY sm.created_at DESC LIMIT {}
                "#,
                    "WHERE sm.product_id = $1", limit
                ),
            )
        } else {
            (
                "",
                format!(
                    r#"
                    SELECT
                        sm.id, sm.product_id, sm.movement_type, sm.quantity, sm.reason,
                        sm.reference_id, sm.user_id, sm.created_at,
                        p.name as product_name, p.sku as product_sku
                    FROM stock_movements sm
                    JOIN products p ON sm.product_id = p.id
                    ORDER BY sm.created_at DESC LIMIT {}
                "#,
                    limit
                ),
            )
        };

        let mut query_builder = sqlx::query(&query);

        if let Some(pid) = product_id {
            query_builder = query_builder.bind(pid);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to fetch stock movements: {}", e)))?;

        let movements: Result<Vec<StockMovementResponse>, _> = rows
            .iter()
            .map(|row| {
                Ok(StockMovementResponse {
                    id: row.try_get("id")?,
                    product_id: row.try_get("product_id")?,
                    product_name: row.try_get("product_name")?,
                    product_sku: row.try_get("product_sku")?,
                    movement_type: row.try_get("movement_type")?,
                    quantity: row.try_get("quantity")?,
                    reason: row.try_get("reason")?,
                    reference_id: row.try_get("reference_id")?,
                    user_id: row.try_get("user_id")?,
                    notes: None,          // Not implemented yet
                    previous_quantity: 0, // Would need to calculate
                    new_quantity: 0,      // Would need to calculate
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect();

        movements.map_err(|e: sqlx::Error| {
            ErpError::internal(format!("Failed to parse stock movements: {}", e))
        })
    }

    async fn get_low_stock_alerts(&self, threshold: Option<i32>) -> ErpResult<Vec<LowStockAlert>> {
        let query = r#"
            SELECT
                id, sku, name, quantity, min_stock_level, category, status, supplier_id, updated_at
            FROM products
            WHERE status = 'active'
            AND (
                CASE
                    WHEN $1::int IS NOT NULL THEN quantity <= $1
                    ELSE quantity <= min_stock_level
                END
            )
            ORDER BY (min_stock_level - quantity) DESC
        "#;

        let rows = sqlx::query(query)
            .bind(threshold)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to fetch low stock alerts: {}", e)))?;

        let alerts: Result<Vec<LowStockAlert>, _> = rows
            .iter()
            .map(|row| {
                let current_quantity: i32 = row.try_get("quantity")?;
                let min_stock_level: i32 = row.try_get("min_stock_level")?;
                let effective_threshold = threshold.unwrap_or(min_stock_level);

                Ok(LowStockAlert {
                    product_id: row.try_get("id")?,
                    sku: row.try_get("sku")?,
                    name: row.try_get("name")?,
                    current_quantity,
                    min_stock_level,
                    shortfall: effective_threshold - current_quantity,
                    category: row.try_get("category")?,
                    status: row.try_get("status")?,
                    last_restock_date: None, // Would need stock movement history
                    supplier_id: row.try_get("supplier_id")?,
                })
            })
            .collect();

        alerts.map_err(|e: sqlx::Error| {
            ErpError::internal(format!("Failed to parse low stock alerts: {}", e))
        })
    }

    async fn get_inventory_valuation(&self) -> ErpResult<InventoryValuation> {
        let summary_query = r#"
            SELECT
                COUNT(*) as total_items,
                SUM(quantity) as total_quantity,
                SUM(cost * quantity) as total_cost_value,
                SUM(price * quantity) as total_sell_value,
                SUM(CASE WHEN quantity <= min_stock_level THEN 1 ELSE 0 END) as low_stock_items,
                SUM(CASE WHEN quantity <= 0 THEN 1 ELSE 0 END) as out_of_stock_items,
                SUM(CASE WHEN max_stock_level IS NOT NULL AND quantity >= max_stock_level THEN 1 ELSE 0 END) as overstocked_items
            FROM products
            WHERE status != 'discontinued'
        "#;

        let summary_row = sqlx::query(summary_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to fetch inventory summary: {}", e)))?;

        let category_query = r#"
            SELECT
                category,
                COUNT(*) as item_count,
                SUM(quantity) as quantity,
                SUM(cost * quantity) as cost_value,
                SUM(price * quantity) as sell_value
            FROM products
            WHERE status != 'discontinued'
            GROUP BY category
            ORDER BY sell_value DESC
        "#;

        let category_rows = sqlx::query(category_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                ErpError::internal(format!("Failed to fetch category valuations: {}", e))
            })?;

        let total_cost_value: Decimal = summary_row
            .try_get("total_cost_value")
            .map_err(|e| ErpError::internal(format!("Failed to parse total cost value: {}", e)))?;
        let total_sell_value: Decimal = summary_row
            .try_get("total_sell_value")
            .map_err(|e| ErpError::internal(format!("Failed to parse total sell value: {}", e)))?;

        let total_margin = total_sell_value - total_cost_value;
        let margin_percentage = if total_cost_value != Decimal::ZERO {
            (total_margin / total_cost_value) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let by_category: Result<Vec<CategoryValuation>, _> = category_rows
            .iter()
            .map(|row| {
                let cost_value: Decimal = row.try_get("cost_value")?;
                let sell_value: Decimal = row.try_get("sell_value")?;
                let margin = sell_value - cost_value;
                let margin_percentage = if cost_value != Decimal::ZERO {
                    (margin / cost_value) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };

                Ok(CategoryValuation {
                    category: row.try_get("category")?,
                    item_count: row.try_get::<i64, _>("item_count")?,
                    quantity: row.try_get::<i64, _>("quantity")?,
                    cost_value,
                    sell_value,
                    margin,
                    margin_percentage,
                })
            })
            .collect();

        Ok(InventoryValuation {
            total_items: summary_row.try_get("total_items")?,
            total_quantity: summary_row.try_get("total_quantity")?,
            total_cost_value,
            total_sell_value,
            total_margin,
            margin_percentage,
            by_category: by_category.map_err(|e: sqlx::Error| {
                ErpError::internal(format!("Failed to parse category valuations: {}", e))
            })?,
            low_stock_items: summary_row.try_get("low_stock_items")?,
            out_of_stock_items: summary_row.try_get("out_of_stock_items")?,
            overstocked_items: summary_row.try_get("overstocked_items")?,
        })
    }

    async fn sku_exists(&self, sku: &str, exclude_id: Option<Uuid>) -> ErpResult<bool> {
        let (_query, count) = if let Some(id) = exclude_id {
            (
                "SELECT COUNT(*) FROM products WHERE UPPER(sku) = UPPER($1) AND id != $2",
                sqlx::query(
                    "SELECT COUNT(*) FROM products WHERE UPPER(sku) = UPPER($1) AND id != $2",
                )
                .bind(sku)
                .bind(id),
            )
        } else {
            (
                "SELECT COUNT(*) FROM products WHERE UPPER(sku) = UPPER($1)",
                sqlx::query("SELECT COUNT(*) FROM products WHERE UPPER(sku) = UPPER($1)").bind(sku),
            )
        };

        let result = count
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to check SKU existence: {}", e)))?;

        let count: i64 = result
            .try_get(0)
            .map_err(|e| ErpError::internal(format!("Failed to parse SKU count: {}", e)))?;

        Ok(count > 0)
    }

    async fn get_inventory_by_category(&self) -> ErpResult<HashMap<String, i64>> {
        let query = "SELECT category, COUNT(*) as count FROM products WHERE status != 'discontinued' GROUP BY category";

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                ErpError::internal(format!("Failed to get inventory by category: {}", e))
            })?;

        let mut result = HashMap::new();
        for row in rows {
            let category: String = row.try_get("category")?;
            let count: i64 = row.try_get("count")?;
            result.insert(category, count);
        }

        Ok(result)
    }

    async fn get_products_by_status(&self, status: ProductStatus) -> ErpResult<Vec<Product>> {
        let query = "SELECT * FROM products WHERE status = $1 ORDER BY created_at DESC";

        let products = sqlx::query_as::<_, Product>(query)
            .bind(status)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ErpError::internal(format!("Failed to get products by status: {}", e)))?;

        Ok(products)
    }
}

// Mock implementation for testing
use std::sync::LazyLock;

static MOCK_PRODUCTS: LazyLock<std::sync::Arc<std::sync::Mutex<HashMap<Uuid, Product>>>> =
    LazyLock::new(|| {
        let products = load_mock_products().unwrap_or_default();
        std::sync::Arc::new(std::sync::Mutex::new(products))
    });

static MOCK_STOCK_MOVEMENTS: LazyLock<std::sync::Arc<std::sync::Mutex<Vec<StockMovement>>>> =
    LazyLock::new(|| std::sync::Arc::new(std::sync::Mutex::new(Vec::new())));

// Helper functions for file-based persistence
fn get_mock_storage_path() -> std::path::PathBuf {
    std::env::temp_dir().join("erp_mock_products.json")
}

fn load_mock_products() -> Option<HashMap<Uuid, Product>> {
    let path = get_mock_storage_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(products) = serde_json::from_str(&content) {
                return serde_json::from_value(products).ok();
            }
        }
    }
    None
}

fn save_mock_products(products: &HashMap<Uuid, Product>) {
    let path = get_mock_storage_path();
    if let Ok(serialized) = serde_json::to_string_pretty(products) {
        if let Err(e) = std::fs::write(&path, serialized) {
            eprintln!("Warning: Failed to save mock products to disk: {}", e);
        }
    }
}

pub struct MockInventoryRepository {
    products: std::sync::Arc<std::sync::Mutex<HashMap<Uuid, Product>>>,
    stock_movements: std::sync::Arc<std::sync::Mutex<Vec<StockMovement>>>,
}

impl MockInventoryRepository {
    pub fn new() -> Self {
        Self {
            products: MOCK_PRODUCTS.clone(),
            stock_movements: MOCK_STOCK_MOVEMENTS.clone(),
        }
    }
}

#[async_trait]
impl InventoryRepository for MockInventoryRepository {
    async fn create_product(&self, request: CreateProductRequest) -> ErpResult<Product> {
        let product = Product::new(request);
        let mut products = self.products.lock().unwrap();
        products.insert(product.id, product.clone());
        save_mock_products(&products);
        Ok(product)
    }

    async fn get_product_by_id(&self, id: Uuid) -> ErpResult<Option<Product>> {
        let products = self.products.lock().unwrap();
        Ok(products.get(&id).cloned())
    }

    async fn get_product_by_sku(&self, sku: &str) -> ErpResult<Option<Product>> {
        let products = self.products.lock().unwrap();
        Ok(products
            .values()
            .find(|p| p.sku.to_uppercase() == sku.to_uppercase())
            .cloned())
    }

    async fn list_products(
        &self,
        _filter: &InventoryFilter,
    ) -> ErpResult<(Vec<InventoryItem>, i64)> {
        let products = self.products.lock().unwrap();
        let items: Vec<InventoryItem> = products
            .values()
            .map(|p| InventoryItem::from_product(p.clone()))
            .collect();
        let total = items.len() as i64;
        Ok((items, total))
    }

    async fn update_product(&self, id: Uuid, request: UpdateProductRequest) -> ErpResult<Product> {
        let mut products = self.products.lock().unwrap();
        if let Some(mut product) = products.get(&id).cloned() {
            product.update(request);
            products.insert(id, product.clone());
            Ok(product)
        } else {
            Err(ErpError::not_found_simple(format!(
                "Product with ID {} not found",
                id
            )))
        }
    }

    async fn delete_product(&self, id: Uuid) -> ErpResult<()> {
        let mut products = self.products.lock().unwrap();
        if products.remove(&id).is_some() {
            save_mock_products(&products);
            Ok(())
        } else {
            Err(ErpError::not_found_simple(format!(
                "Product with ID {} not found",
                id
            )))
        }
    }

    async fn adjust_stock(
        &self,
        request: StockAdjustmentRequest,
        user_id: Uuid,
    ) -> ErpResult<StockMovement> {
        let mut products = self.products.lock().unwrap();
        if let Some(mut product) = products.get(&request.product_id).cloned() {
            let mut movement = product.adjust_quantity(request.quantity_change, request.reason);
            movement.user_id = user_id;
            movement.reference_id = request.reference_id;

            products.insert(request.product_id, product);

            let mut movements = self.stock_movements.lock().unwrap();
            movements.push(movement.clone());

            Ok(movement)
        } else {
            Err(ErpError::not_found_simple(format!(
                "Product with ID {} not found",
                request.product_id
            )))
        }
    }

    async fn get_stock_movements(
        &self,
        _product_id: Option<Uuid>,
        _limit: Option<i32>,
    ) -> ErpResult<Vec<StockMovementResponse>> {
        // Simplified mock implementation
        Ok(Vec::new())
    }

    async fn get_low_stock_alerts(&self, _threshold: Option<i32>) -> ErpResult<Vec<LowStockAlert>> {
        // Simplified mock implementation
        Ok(Vec::new())
    }

    async fn get_inventory_valuation(&self) -> ErpResult<InventoryValuation> {
        // Simplified mock implementation
        Ok(InventoryValuation {
            total_items: 0,
            total_quantity: 0,
            total_cost_value: Decimal::ZERO,
            total_sell_value: Decimal::ZERO,
            total_margin: Decimal::ZERO,
            margin_percentage: Decimal::ZERO,
            by_category: Vec::new(),
            low_stock_items: 0,
            out_of_stock_items: 0,
            overstocked_items: 0,
        })
    }

    async fn sku_exists(&self, sku: &str, exclude_id: Option<Uuid>) -> ErpResult<bool> {
        let products = self.products.lock().unwrap();
        Ok(products.values().any(|p| {
            p.sku.to_uppercase() == sku.to_uppercase()
                && (exclude_id.is_none() || Some(p.id) != exclude_id)
        }))
    }

    async fn get_inventory_by_category(&self) -> ErpResult<HashMap<String, i64>> {
        Ok(HashMap::new())
    }

    async fn get_products_by_status(&self, _status: ProductStatus) -> ErpResult<Vec<Product>> {
        Ok(Vec::new())
    }
}
