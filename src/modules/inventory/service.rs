use crate::core::database::models::product::{Product, ProductStatus, StockMovement, StockStatus};
use crate::modules::inventory::models::{
    CreateInventoryItemRequest, InventoryFilter, InventoryItem, InventoryItemResponse,
    InventoryListResponse, InventoryValuation, LowStockAlert, StockAdjustmentRequest,
    StockMovementResponse, UpdateInventoryItemRequest,
};
use crate::modules::inventory::repository::InventoryRepository;
use crate::utils::error::{ErpError, ErpResult};
use crate::utils::validation::ValidationService;
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[async_trait]
pub trait InventoryService: Send + Sync {
    async fn create_product(
        &self,
        request: CreateInventoryItemRequest,
        user_id: Uuid,
    ) -> ErpResult<InventoryItemResponse>;
    async fn get_product(&self, id_or_sku: &str) -> ErpResult<InventoryItemResponse>;
    async fn list_products(&self, filter: InventoryFilter) -> ErpResult<InventoryListResponse>;
    async fn update_product(
        &self,
        id_or_sku: &str,
        request: UpdateInventoryItemRequest,
        user_id: Uuid,
    ) -> ErpResult<InventoryItemResponse>;
    async fn delete_product(&self, id_or_sku: &str, force: bool, user_id: Uuid) -> ErpResult<()>;
    async fn adjust_stock(
        &self,
        id_or_sku: &str,
        quantity_change: i32,
        reason: String,
        user_id: Uuid,
    ) -> ErpResult<StockMovementResponse>;
    async fn get_stock_movements(
        &self,
        product_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> ErpResult<Vec<StockMovementResponse>>;
    async fn get_low_stock_alerts(&self, threshold: Option<i32>) -> ErpResult<Vec<LowStockAlert>>;
    async fn get_inventory_valuation(&self) -> ErpResult<InventoryValuation>;
    async fn reserve_stock(
        &self,
        id_or_sku: &str,
        quantity: i32,
        reference_id: Uuid,
    ) -> ErpResult<bool>;
    async fn release_reservation(
        &self,
        id_or_sku: &str,
        quantity: i32,
        reference_id: Uuid,
    ) -> ErpResult<bool>;
    async fn get_products_requiring_reorder(&self) -> ErpResult<Vec<InventoryItemResponse>>;
    async fn bulk_update_prices(
        &self,
        category: Option<String>,
        price_adjustment: Decimal,
        user_id: Uuid,
    ) -> ErpResult<i64>;
}

pub struct InventoryServiceImpl {
    repository: Arc<dyn InventoryRepository>,
    validation_service: ValidationService,
}

impl InventoryServiceImpl {
    pub fn new(repository: Arc<dyn InventoryRepository>) -> Self {
        Self {
            repository,
            validation_service: ValidationService::new(),
        }
    }

    async fn get_product_by_id_or_sku(&self, id_or_sku: &str) -> ErpResult<Product> {
        // Try to parse as UUID first
        if let Ok(id) = Uuid::parse_str(id_or_sku) {
            if let Some(product) = self.repository.get_product_by_id(id).await? {
                return Ok(product);
            }
        }

        // Try to find by SKU
        if let Some(product) = self.repository.get_product_by_sku(id_or_sku).await? {
            return Ok(product);
        }

        Err(ErpError::not_found_simple(format!(
            "Product not found: {}",
            id_or_sku
        )))
    }

    fn validate_stock_adjustment(&self, product: &Product, quantity_change: i32) -> ErpResult<()> {
        if quantity_change == 0 {
            return Err(ErpError::validation_simple(
                "Quantity change cannot be zero".to_string(),
            ));
        }

        let new_quantity = product.quantity + quantity_change;
        if new_quantity < 0 {
            return Err(ErpError::validation_simple(format!(
                "Insufficient stock. Current: {}, Requested: {}",
                product.quantity,
                quantity_change.abs()
            )));
        }

        if new_quantity > i32::MAX / 2 {
            return Err(ErpError::validation_simple(
                "Quantity would exceed maximum allowed".to_string(),
            ));
        }

        Ok(())
    }

    async fn check_references_before_delete(&self, _product_id: Uuid) -> ErpResult<bool> {
        // TODO: Check if product is referenced in orders, invoices, etc.
        // For now, return false (no references)
        Ok(false)
    }

    fn calculate_reorder_recommendations(
        &self,
        items: &[InventoryItem],
    ) -> Vec<InventoryItemResponse> {
        items
            .iter()
            .filter(|item| item.is_reorder_needed())
            .map(|item| item.to_response())
            .collect()
    }

    fn format_stock_movement_response(
        &self,
        movement: StockMovement,
        product: &Product,
    ) -> StockMovementResponse {
        StockMovementResponse {
            id: movement.id,
            product_id: movement.product_id,
            product_name: product.name.clone(),
            product_sku: product.sku.clone(),
            movement_type: movement.movement_type,
            quantity: movement.quantity,
            reason: movement.reason,
            reference_id: movement.reference_id,
            user_id: movement.user_id,
            notes: None, // Could be extended
            previous_quantity: product.quantity - movement.quantity,
            new_quantity: product.quantity,
            created_at: movement.created_at,
        }
    }
}

#[async_trait]
impl InventoryService for InventoryServiceImpl {
    async fn create_product(
        &self,
        request: CreateInventoryItemRequest,
        _user_id: Uuid,
    ) -> ErpResult<InventoryItemResponse> {
        info!("Creating new product: {}", request.name);

        // Validate request
        request.validate().map_err(ErpError::validation_simple)?;

        // Validate product name
        self.validation_service
            .validate_name(&request.name, "product_name")?;

        // Validate category
        self.validation_service
            .validate_name(&request.category, "category")?;

        // Validate price
        if request.price <= Decimal::ZERO {
            return Err(ErpError::validation_simple(
                "Price must be greater than zero".to_string(),
            ));
        }

        // Convert to create product request
        let create_request = request.to_create_product_request();

        // Check if SKU exists
        if self
            .repository
            .sku_exists(&create_request.sku, None)
            .await?
        {
            return Err(ErpError::conflict(&format!(
                "SKU '{}' already exists",
                create_request.sku
            )));
        }

        // Create product
        let product = self.repository.create_product(create_request).await?;

        info!(
            "Product created successfully: {} ({})",
            product.name, product.sku
        );

        // Convert to inventory item response
        let inventory_item = InventoryItem::from_product(product);
        Ok(inventory_item.to_response())
    }

    async fn get_product(&self, id_or_sku: &str) -> ErpResult<InventoryItemResponse> {
        let product = self.get_product_by_id_or_sku(id_or_sku).await?;
        let inventory_item = InventoryItem::from_product(product);
        Ok(inventory_item.to_response())
    }

    async fn list_products(&self, filter: InventoryFilter) -> ErpResult<InventoryListResponse> {
        // Validate pagination
        let page = filter.page.unwrap_or(1);
        let limit = filter.limit.unwrap_or(20);

        if page == 0 {
            return Err(ErpError::validation_simple(
                "Page must be greater than 0".to_string(),
            ));
        }

        if limit == 0 || limit > 100 {
            return Err(ErpError::validation_simple(
                "Limit must be between 1 and 100".to_string(),
            ));
        }

        // Get products from repository
        let (inventory_items, total) = self.repository.list_products(&filter).await?;

        // Convert to responses
        let items: Vec<InventoryItemResponse> = inventory_items
            .iter()
            .map(|item| item.to_response())
            .collect();

        // Calculate additional statistics
        let low_stock_count = items
            .iter()
            .filter(|item| item.stock_status == StockStatus::LowStock)
            .count() as i64;

        let out_of_stock_count = items
            .iter()
            .filter(|item| item.stock_status == StockStatus::OutOfStock)
            .count() as i64;

        Ok(InventoryListResponse {
            items,
            total,
            page,
            per_page: limit,
            low_stock_count,
            out_of_stock_count,
        })
    }

    async fn update_product(
        &self,
        id_or_sku: &str,
        request: UpdateInventoryItemRequest,
        _user_id: Uuid,
    ) -> ErpResult<InventoryItemResponse> {
        info!("Updating product: {}", id_or_sku);

        // Get current product
        let product = self.get_product_by_id_or_sku(id_or_sku).await?;

        // Validate update request
        if let Some(name) = &request.name {
            self.validation_service
                .validate_name(name, "product_name")?;
        }

        if let Some(category) = &request.category {
            self.validation_service
                .validate_name(category, "category")?;
        }

        if let Some(price) = request.price {
            if price <= Decimal::ZERO {
                return Err(ErpError::validation_simple(
                    "Price must be greater than zero".to_string(),
                ));
            }
        }

        // Convert to update product request
        let update_request = request.to_update_product_request();

        // Update product
        let updated_product = self
            .repository
            .update_product(product.id, update_request)
            .await?;

        info!(
            "Product updated successfully: {} ({})",
            updated_product.name, updated_product.sku
        );

        let inventory_item = InventoryItem::from_product(updated_product);
        Ok(inventory_item.to_response())
    }

    async fn delete_product(&self, id_or_sku: &str, force: bool, _user_id: Uuid) -> ErpResult<()> {
        info!("Deleting product: {} (force: {})", id_or_sku, force);

        let product = self.get_product_by_id_or_sku(id_or_sku).await?;

        // Check for references if not forced
        if !force {
            let has_references = self.check_references_before_delete(product.id).await?;
            if has_references {
                return Err(ErpError::validation_simple(
                    "Cannot delete product that is referenced in orders or other records. Use --force to override.".to_string()
                ));
            }
        }

        self.repository.delete_product(product.id).await?;

        info!(
            "Product deleted successfully: {} ({})",
            product.name, product.sku
        );
        Ok(())
    }

    async fn adjust_stock(
        &self,
        id_or_sku: &str,
        quantity_change: i32,
        reason: String,
        user_id: Uuid,
    ) -> ErpResult<StockMovementResponse> {
        info!(
            "Adjusting stock for product: {} by {}",
            id_or_sku, quantity_change
        );

        let product = self.get_product_by_id_or_sku(id_or_sku).await?;

        // Validate stock adjustment
        self.validate_stock_adjustment(&product, quantity_change)?;

        // Validate reason
        if reason.trim().is_empty() {
            return Err(ErpError::validation_simple(
                "Reason is required for stock adjustments".to_string(),
            ));
        }

        let adjustment_request = StockAdjustmentRequest {
            product_id: product.id,
            quantity_change,
            reason: reason.clone(),
            reference_id: None,
            notes: None,
        };

        let movement = self
            .repository
            .adjust_stock(adjustment_request, user_id)
            .await?;

        // Get updated product for response
        let updated_product = self
            .repository
            .get_product_by_id(product.id)
            .await?
            .ok_or_else(|| {
                ErpError::not_found_simple("Product not found after stock adjustment".to_string())
            })?;

        let response = self.format_stock_movement_response(movement, &updated_product);

        info!(
            "Stock adjusted successfully for {}: {} -> {}",
            updated_product.sku, response.previous_quantity, response.new_quantity
        );

        // Log warning for low stock
        if updated_product.is_low_stock() {
            warn!(
                "Product {} is now at low stock level: {} (min: {})",
                updated_product.sku, updated_product.quantity, updated_product.min_stock_level
            );
        }

        Ok(response)
    }

    async fn get_stock_movements(
        &self,
        product_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> ErpResult<Vec<StockMovementResponse>> {
        let limit = limit.unwrap_or(50);
        if limit <= 0 || limit > 200 {
            return Err(ErpError::validation_simple(
                "Limit must be between 1 and 200".to_string(),
            ));
        }

        self.repository
            .get_stock_movements(product_id, Some(limit))
            .await
    }

    async fn get_low_stock_alerts(&self, threshold: Option<i32>) -> ErpResult<Vec<LowStockAlert>> {
        if let Some(threshold) = threshold {
            if threshold < 0 {
                return Err(ErpError::validation_simple(
                    "Threshold cannot be negative".to_string(),
                ));
            }
        }

        let alerts = self.repository.get_low_stock_alerts(threshold).await?;

        if !alerts.is_empty() {
            info!("Found {} low stock alerts", alerts.len());
        }

        Ok(alerts)
    }

    async fn get_inventory_valuation(&self) -> ErpResult<InventoryValuation> {
        self.repository.get_inventory_valuation().await
    }

    async fn reserve_stock(
        &self,
        id_or_sku: &str,
        quantity: i32,
        _reference_id: Uuid,
    ) -> ErpResult<bool> {
        info!("Reserving {} units for product: {}", quantity, id_or_sku);

        if quantity <= 0 {
            return Err(ErpError::validation_simple(
                "Quantity must be greater than zero".to_string(),
            ));
        }

        let product = self.get_product_by_id_or_sku(id_or_sku).await?;

        // Check availability
        if product.quantity < quantity {
            return Ok(false);
        }

        // TODO: Implement reservation system
        // This would involve creating a reservations table and updating available quantities
        warn!("Stock reservation not fully implemented - returning success");
        Ok(true)
    }

    async fn release_reservation(
        &self,
        id_or_sku: &str,
        quantity: i32,
        _reference_id: Uuid,
    ) -> ErpResult<bool> {
        info!(
            "Releasing {} units reservation for product: {}",
            quantity, id_or_sku
        );

        if quantity <= 0 {
            return Err(ErpError::validation_simple(
                "Quantity must be greater than zero".to_string(),
            ));
        }

        // TODO: Implement reservation release
        warn!("Stock reservation release not fully implemented - returning success");
        Ok(true)
    }

    async fn get_products_requiring_reorder(&self) -> ErpResult<Vec<InventoryItemResponse>> {
        let filter = InventoryFilter {
            low_stock_only: Some(true),
            status: Some(ProductStatus::Active),
            ..Default::default()
        };

        let (inventory_items, _) = self.repository.list_products(&filter).await?;
        let reorder_items = self.calculate_reorder_recommendations(&inventory_items);

        info!("Found {} products requiring reorder", reorder_items.len());
        Ok(reorder_items)
    }

    async fn bulk_update_prices(
        &self,
        category: Option<String>,
        price_adjustment: Decimal,
        _user_id: Uuid,
    ) -> ErpResult<i64> {
        info!(
            "Bulk updating prices - category: {:?}, adjustment: {}",
            category, price_adjustment
        );

        if price_adjustment == Decimal::ZERO {
            return Err(ErpError::validation_simple(
                "Price adjustment cannot be zero".to_string(),
            ));
        }

        // TODO: Implement bulk price update
        // This would involve getting all products in category and updating their prices
        warn!("Bulk price update not fully implemented");
        Ok(0)
    }
}

// Implement Default for InventoryFilter
impl Default for InventoryFilter {
    fn default() -> Self {
        Self {
            category: None,
            status: None,
            stock_status: None,
            sku: None,
            name: None,
            min_quantity: None,
            max_quantity: None,
            low_stock_only: None,
            location: None,
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::inventory::repository::MockInventoryRepository;
    use rust_decimal::Decimal;

    fn create_test_service() -> InventoryServiceImpl {
        let repository = Arc::new(MockInventoryRepository::new());
        InventoryServiceImpl::new(repository)
    }

    fn create_test_request() -> CreateInventoryItemRequest {
        CreateInventoryItemRequest {
            name: "Test Product".to_string(),
            description: Some("A test product".to_string()),
            category: "Electronics".to_string(),
            price: Decimal::new(1999, 2),
            cost: Some(Decimal::new(1200, 2)),
            quantity: 100,
            min_stock: 10,
            max_stock: Some(1000),
            sku: Some("TEST-001".to_string()),
            is_taxable: Some(true),
            weight: None,
            dimensions: None,
            barcode: None,
            supplier_id: None,
            location: None,
        }
    }

    #[tokio::test]
    async fn test_create_product() {
        let service = create_test_service();
        let request = create_test_request();
        let user_id = Uuid::new_v4();

        let result = service.create_product(request, user_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.name, "Test Product");
        assert_eq!(response.sku, "TEST-001");
    }

    #[tokio::test]
    async fn test_create_product_validation_error() {
        let service = create_test_service();
        let mut request = create_test_request();
        request.name = "".to_string(); // Invalid empty name
        let user_id = Uuid::new_v4();

        let result = service.create_product(request, user_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErpError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_list_products() {
        let service = create_test_service();
        let filter = InventoryFilter::default();

        let result = service.list_products(filter).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.page, 1);
        assert_eq!(response.per_page, 20);
    }

    #[tokio::test]
    async fn test_stock_adjustment_validation() {
        let service = create_test_service();

        // First create a product
        let request = create_test_request();
        let user_id = Uuid::new_v4();
        let product_response = service.create_product(request, user_id).await.unwrap();

        // Test invalid quantity change (zero)
        let result = service
            .adjust_stock(&product_response.sku, 0, "Test".to_string(), user_id)
            .await;
        assert!(result.is_err());

        // Test invalid reason (empty)
        let result = service
            .adjust_stock(&product_response.sku, 10, "".to_string(), user_id)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_low_stock_alerts() {
        let service = create_test_service();

        let result = service.get_low_stock_alerts(Some(5)).await;
        assert!(result.is_ok());

        let alerts = result.unwrap();
        // Mock implementation returns empty list
        assert_eq!(alerts.len(), 0);
    }

    #[tokio::test]
    async fn test_inventory_valuation() {
        let service = create_test_service();

        let result = service.get_inventory_valuation().await;
        assert!(result.is_ok());

        let valuation = result.unwrap();
        assert_eq!(valuation.total_items, 0); // Mock returns zero
    }
}
