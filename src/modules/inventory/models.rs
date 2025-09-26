use crate::core::database::models::product::{
    CreateProductRequest, Product, ProductFilter, ProductStatus, StockMovement, StockMovementType,
    StockStatus, UpdateProductRequest,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub product: Product,
    pub stock_level: i32,
    pub available_quantity: i32,
    pub reserved_quantity: i32,
    pub last_movement_date: Option<DateTime<Utc>>,
    pub reorder_point: i32,
    pub reorder_quantity: i32,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInventoryItemRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: Decimal,
    pub cost: Option<Decimal>,
    pub quantity: i32,
    pub min_stock: i32,
    pub max_stock: Option<i32>,
    pub sku: Option<String>,
    pub is_taxable: Option<bool>,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub barcode: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInventoryItemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub price: Option<Decimal>,
    pub cost: Option<Decimal>,
    pub min_stock: Option<i32>,
    pub max_stock: Option<i32>,
    pub is_taxable: Option<bool>,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub barcode: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryFilter {
    pub category: Option<String>,
    pub status: Option<ProductStatus>,
    pub stock_status: Option<StockStatus>,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub low_stock_only: Option<bool>,
    pub location: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryListResponse {
    pub items: Vec<InventoryItemResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub low_stock_count: i64,
    pub out_of_stock_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: Decimal,
    pub cost: Decimal,
    pub quantity: i32,
    pub available_quantity: i32,
    pub reserved_quantity: i32,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub status: ProductStatus,
    pub stock_status: StockStatus,
    pub location: Option<String>,
    pub last_movement_date: Option<DateTime<Utc>>,
    pub margin: Decimal,
    pub margin_percentage: Decimal,
    pub reorder_needed: bool,
    pub days_of_stock: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAdjustmentRequest {
    pub product_id: Uuid,
    pub quantity_change: i32,
    pub reason: String,
    pub reference_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovementResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub product_sku: String,
    pub movement_type: StockMovementType,
    pub quantity: i32,
    pub reason: String,
    pub reference_id: Option<Uuid>,
    pub user_id: Uuid,
    pub notes: Option<String>,
    pub previous_quantity: i32,
    pub new_quantity: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowStockAlert {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub current_quantity: i32,
    pub min_stock_level: i32,
    pub shortfall: i32,
    pub category: String,
    pub status: ProductStatus,
    pub last_restock_date: Option<DateTime<Utc>>,
    pub supplier_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryValuation {
    pub total_items: i64,
    pub total_quantity: i64,
    pub total_cost_value: Decimal,
    pub total_sell_value: Decimal,
    pub total_margin: Decimal,
    pub margin_percentage: Decimal,
    pub by_category: Vec<CategoryValuation>,
    pub low_stock_items: i64,
    pub out_of_stock_items: i64,
    pub overstocked_items: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryValuation {
    pub category: String,
    pub item_count: i64,
    pub quantity: i64,
    pub cost_value: Decimal,
    pub sell_value: Decimal,
    pub margin: Decimal,
    pub margin_percentage: Decimal,
}

impl InventoryItem {
    pub fn from_product(product: Product) -> Self {
        let available_quantity = product.quantity.max(0);
        let reserved_quantity = 0; // This would come from orders/reservations

        Self {
            stock_level: product.quantity,
            available_quantity,
            reserved_quantity,
            last_movement_date: None, // This would be populated from stock movements
            reorder_point: product.min_stock_level,
            reorder_quantity: product.max_stock_level.unwrap_or(100) - product.min_stock_level,
            location: None,
            product,
        }
    }

    pub fn to_response(&self) -> InventoryItemResponse {
        let product_response = self.product.to_response();

        InventoryItemResponse {
            id: self.product.id,
            sku: self.product.sku.clone(),
            name: self.product.name.clone(),
            description: self.product.description.clone(),
            category: self.product.category.clone(),
            price: self.product.price,
            cost: self.product.cost,
            quantity: self.stock_level,
            available_quantity: self.available_quantity,
            reserved_quantity: self.reserved_quantity,
            min_stock_level: self.product.min_stock_level,
            max_stock_level: self.product.max_stock_level,
            status: self.product.status.clone(),
            stock_status: product_response.stock_status,
            location: self.location.clone(),
            last_movement_date: self.last_movement_date,
            margin: product_response.margin,
            margin_percentage: product_response.margin_percentage,
            reorder_needed: self.is_reorder_needed(),
            days_of_stock: self.calculate_days_of_stock(),
            created_at: self.product.created_at,
            updated_at: self.product.updated_at,
        }
    }

    pub fn is_reorder_needed(&self) -> bool {
        self.available_quantity <= self.reorder_point
    }

    pub fn calculate_days_of_stock(&self) -> Option<i32> {
        // This is a simplified calculation
        // In reality, you'd want to use historical sales data
        if self.available_quantity <= 0 {
            Some(0)
        } else {
            // Assume average daily usage of 1 unit per day as a default
            // This should be calculated based on historical data
            Some(self.available_quantity)
        }
    }

    pub fn can_fulfill_quantity(&self, requested_quantity: i32) -> bool {
        self.available_quantity >= requested_quantity
    }

    pub fn reserve_quantity(&mut self, quantity: i32) -> bool {
        if self.can_fulfill_quantity(quantity) {
            self.available_quantity -= quantity;
            self.reserved_quantity += quantity;
            true
        } else {
            false
        }
    }

    pub fn release_reserved_quantity(&mut self, quantity: i32) {
        let release_amount = quantity.min(self.reserved_quantity);
        self.reserved_quantity -= release_amount;
        self.available_quantity += release_amount;
    }

    pub fn fulfill_reservation(&mut self, quantity: i32) -> bool {
        if quantity <= self.reserved_quantity {
            self.reserved_quantity -= quantity;
            self.stock_level -= quantity;
            true
        } else {
            false
        }
    }
}

impl CreateInventoryItemRequest {
    pub fn to_create_product_request(&self) -> CreateProductRequest {
        let generated_sku = self.sku.clone().unwrap_or_else(|| {
            format!(
                "SKU-{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            )
        });

        CreateProductRequest {
            sku: generated_sku,
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            price: self.price,
            cost: self
                .cost
                .unwrap_or_else(|| self.price * Decimal::new(70, 2)), // Default 70% of price
            quantity: self.quantity,
            min_stock_level: self.min_stock,
            max_stock_level: self.max_stock,
            is_taxable: self.is_taxable.unwrap_or(true),
            weight: self.weight,
            dimensions: self.dimensions.clone(),
            barcode: self.barcode.clone(),
            supplier_id: self.supplier_id,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Product name is required".to_string());
        }

        if self.category.trim().is_empty() {
            return Err("Category is required".to_string());
        }

        if self.price <= Decimal::ZERO {
            return Err("Price must be greater than zero".to_string());
        }

        if let Some(cost) = self.cost {
            if cost < Decimal::ZERO {
                return Err("Cost cannot be negative".to_string());
            }
        }

        if self.quantity < 0 {
            return Err("Quantity cannot be negative".to_string());
        }

        if self.min_stock < 0 {
            return Err("Minimum stock cannot be negative".to_string());
        }

        if let Some(max_stock) = self.max_stock {
            if max_stock < self.min_stock {
                return Err("Maximum stock cannot be less than minimum stock".to_string());
            }
        }

        if let Some(sku) = &self.sku {
            if sku.trim().is_empty() {
                return Err("SKU cannot be empty".to_string());
            }
            if sku.len() > 50 {
                return Err("SKU cannot exceed 50 characters".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateInventoryItemRequest {
    pub fn to_update_product_request(&self) -> UpdateProductRequest {
        UpdateProductRequest {
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            price: self.price,
            cost: self.cost,
            min_stock_level: self.min_stock,
            max_stock_level: self.max_stock,
            status: None, // Status updates handled separately
            is_taxable: self.is_taxable,
            weight: self.weight,
            dimensions: self.dimensions.clone(),
            barcode: self.barcode.clone(),
            supplier_id: self.supplier_id,
        }
    }
}

impl InventoryFilter {
    pub fn to_product_filter(&self) -> ProductFilter {
        ProductFilter {
            category: self.category.clone(),
            status: self.status.clone(),
            stock_status: self.stock_status.clone(),
            sku: self.sku.clone(),
            name: self.name.clone(),
            min_price: None,
            max_price: None,
            supplier_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_inventory_item_request_validation() {
        let valid_request = CreateInventoryItemRequest {
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
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateInventoryItemRequest {
            name: "".to_string(), // Empty name
            ..valid_request
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_inventory_item_from_product() {
        let product = create_test_product();
        let inventory_item = InventoryItem::from_product(product.clone());

        assert_eq!(inventory_item.stock_level, product.quantity);
        assert_eq!(inventory_item.available_quantity, product.quantity.max(0));
        assert_eq!(inventory_item.reserved_quantity, 0);
        assert_eq!(inventory_item.reorder_point, product.min_stock_level);
    }

    #[test]
    fn test_inventory_item_reservation() {
        let product = create_test_product();
        let mut inventory_item = InventoryItem::from_product(product);

        assert!(inventory_item.reserve_quantity(10));
        assert_eq!(inventory_item.available_quantity, 90);
        assert_eq!(inventory_item.reserved_quantity, 10);

        assert!(!inventory_item.reserve_quantity(95)); // Not enough available

        inventory_item.release_reserved_quantity(5);
        assert_eq!(inventory_item.available_quantity, 95);
        assert_eq!(inventory_item.reserved_quantity, 5);
    }

    fn create_test_product() -> Product {
        let request = CreateProductRequest {
            sku: "TEST-001".to_string(),
            name: "Test Product".to_string(),
            description: None,
            category: "Test".to_string(),
            price: Decimal::new(1999, 2),
            cost: Decimal::new(1200, 2),
            quantity: 100,
            min_stock_level: 10,
            max_stock_level: Some(1000),
            is_taxable: true,
            weight: None,
            dimensions: None,
            barcode: None,
            supplier_id: None,
        };

        Product::new(request)
    }
}
