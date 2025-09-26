use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: Decimal,
    pub cost: Decimal,
    pub quantity: i32,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub status: ProductStatus,
    pub is_taxable: bool,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub barcode: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "product_status", rename_all = "lowercase")]
pub enum ProductStatus {
    Active,
    Inactive,
    Discontinued,
    OutOfStock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: Decimal,
    pub cost: Decimal,
    pub quantity: i32,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub is_taxable: bool,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub barcode: Option<String>,
    pub supplier_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub price: Option<Decimal>,
    pub cost: Option<Decimal>,
    pub min_stock_level: Option<i32>,
    pub max_stock_level: Option<i32>,
    pub status: Option<ProductStatus>,
    pub is_taxable: Option<bool>,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub barcode: Option<String>,
    pub supplier_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: Decimal,
    pub cost: Decimal,
    pub quantity: i32,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub status: ProductStatus,
    pub is_taxable: bool,
    pub weight: Option<Decimal>,
    pub dimensions: Option<String>,
    pub barcode: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub margin: Decimal,
    pub margin_percentage: Decimal,
    pub stock_status: StockStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
    pub id: Uuid,
    pub product_id: Uuid,
    pub movement_type: StockMovementType,
    pub quantity: i32,
    pub reason: String,
    pub reference_id: Option<Uuid>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "stock_movement_type", rename_all = "lowercase")]
pub enum StockMovementType {
    In,
    Out,
    Adjustment,
    Transfer,
    Damaged,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StockStatus {
    InStock,
    LowStock,
    OutOfStock,
    Overstocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductFilter {
    pub category: Option<String>,
    pub status: Option<ProductStatus>,
    pub stock_status: Option<StockStatus>,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub supplier_id: Option<Uuid>,
}

impl Product {
    pub fn new(request: CreateProductRequest) -> Self {
        let now = Utc::now();
        let status = if request.quantity > 0 {
            ProductStatus::Active
        } else {
            ProductStatus::OutOfStock
        };

        Self {
            id: Uuid::new_v4(),
            sku: request.sku.to_uppercase(),
            name: request.name,
            description: request.description,
            category: request.category,
            price: request.price,
            cost: request.cost,
            quantity: request.quantity,
            min_stock_level: request.min_stock_level,
            max_stock_level: request.max_stock_level,
            status,
            is_taxable: request.is_taxable,
            weight: request.weight,
            dimensions: request.dimensions,
            barcode: request.barcode,
            supplier_id: request.supplier_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, request: UpdateProductRequest) {
        if let Some(name) = request.name {
            self.name = name;
        }
        if let Some(description) = request.description {
            self.description = Some(description);
        }
        if let Some(category) = request.category {
            self.category = category;
        }
        if let Some(price) = request.price {
            self.price = price;
        }
        if let Some(cost) = request.cost {
            self.cost = cost;
        }
        if let Some(min_stock_level) = request.min_stock_level {
            self.min_stock_level = min_stock_level;
        }
        if let Some(max_stock_level) = request.max_stock_level {
            self.max_stock_level = Some(max_stock_level);
        }
        if let Some(status) = request.status {
            self.status = status;
        }
        if let Some(is_taxable) = request.is_taxable {
            self.is_taxable = is_taxable;
        }
        if let Some(weight) = request.weight {
            self.weight = Some(weight);
        }
        if let Some(dimensions) = request.dimensions {
            self.dimensions = Some(dimensions);
        }
        if let Some(barcode) = request.barcode {
            self.barcode = Some(barcode);
        }
        if let Some(supplier_id) = request.supplier_id {
            self.supplier_id = Some(supplier_id);
        }
        self.updated_at = Utc::now();
    }

    pub fn adjust_quantity(&mut self, quantity_change: i32, reason: String) -> StockMovement {
        let movement_type = if quantity_change > 0 {
            StockMovementType::In
        } else if quantity_change < 0 {
            StockMovementType::Out
        } else {
            StockMovementType::Adjustment
        };

        self.quantity += quantity_change;
        self.update_status_based_on_quantity();
        self.updated_at = Utc::now();

        StockMovement {
            id: Uuid::new_v4(),
            product_id: self.id,
            movement_type,
            quantity: quantity_change.abs(),
            reason,
            reference_id: None,
            user_id: Uuid::new_v4(), // This should be passed in
            created_at: Utc::now(),
        }
    }

    pub fn calculate_margin(&self) -> Decimal {
        self.price - self.cost
    }

    pub fn calculate_margin_percentage(&self) -> Decimal {
        if self.cost == Decimal::ZERO {
            Decimal::ZERO
        } else {
            ((self.price - self.cost) / self.cost) * Decimal::from(100)
        }
    }

    pub fn get_stock_status(&self) -> StockStatus {
        if self.quantity <= 0 {
            StockStatus::OutOfStock
        } else if self.quantity <= self.min_stock_level {
            StockStatus::LowStock
        } else if let Some(max_level) = self.max_stock_level {
            if self.quantity >= max_level {
                StockStatus::Overstocked
            } else {
                StockStatus::InStock
            }
        } else {
            StockStatus::InStock
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self.status, ProductStatus::Active) && self.quantity > 0
    }

    pub fn is_low_stock(&self) -> bool {
        self.quantity <= self.min_stock_level
    }

    pub fn is_out_of_stock(&self) -> bool {
        self.quantity <= 0
    }

    fn update_status_based_on_quantity(&mut self) {
        if self.quantity <= 0 {
            self.status = ProductStatus::OutOfStock;
        } else if self.status == ProductStatus::OutOfStock {
            self.status = ProductStatus::Active;
        }
    }

    pub fn to_response(&self) -> ProductResponse {
        ProductResponse {
            id: self.id,
            sku: self.sku.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            price: self.price,
            cost: self.cost,
            quantity: self.quantity,
            min_stock_level: self.min_stock_level,
            max_stock_level: self.max_stock_level,
            status: self.status.clone(),
            is_taxable: self.is_taxable,
            weight: self.weight,
            dimensions: self.dimensions.clone(),
            barcode: self.barcode.clone(),
            supplier_id: self.supplier_id,
            margin: self.calculate_margin(),
            margin_percentage: self.calculate_margin_percentage(),
            stock_status: self.get_stock_status(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl ProductStatus {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Active,
            Self::Inactive,
            Self::Discontinued,
            Self::OutOfStock,
        ]
    }

    pub fn is_sellable(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl StockMovementType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::In,
            Self::Out,
            Self::Adjustment,
            Self::Transfer,
            Self::Damaged,
            Self::Expired,
        ]
    }
}

impl std::fmt::Display for ProductStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Discontinued => write!(f, "discontinued"),
            Self::OutOfStock => write!(f, "out_of_stock"),
        }
    }
}

impl std::fmt::Display for StockMovementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::In => write!(f, "in"),
            Self::Out => write!(f, "out"),
            Self::Adjustment => write!(f, "adjustment"),
            Self::Transfer => write!(f, "transfer"),
            Self::Damaged => write!(f, "damaged"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

impl std::fmt::Display for StockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InStock => write!(f, "in_stock"),
            Self::LowStock => write!(f, "low_stock"),
            Self::OutOfStock => write!(f, "out_of_stock"),
            Self::Overstocked => write!(f, "overstocked"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation() {
        let request = CreateProductRequest {
            sku: "prod-001".to_string(),
            name: "Test Product".to_string(),
            description: Some("A test product".to_string()),
            category: "Electronics".to_string(),
            price: Decimal::new(1999, 2), // 19.99
            cost: Decimal::new(1200, 2),  // 12.00
            quantity: 100,
            min_stock_level: 10,
            max_stock_level: Some(1000),
            is_taxable: true,
            weight: Some(Decimal::new(150, 2)), // 1.50 kg
            dimensions: Some("10x5x2".to_string()),
            barcode: Some("123456789012".to_string()),
            supplier_id: None,
        };

        let product = Product::new(request);

        assert_eq!(product.sku, "PROD-001");
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.status, ProductStatus::Active);
        assert_eq!(product.quantity, 100);
    }

    #[test]
    fn test_product_margin_calculation() {
        let mut product = create_test_product();
        product.price = Decimal::new(2000, 2); // 20.00
        product.cost = Decimal::new(1200, 2); // 12.00

        let margin = product.calculate_margin();
        let margin_percentage = product.calculate_margin_percentage();

        assert_eq!(margin, Decimal::new(800, 2)); // 8.00
        assert_eq!(margin_percentage.round_dp(2), Decimal::new(6667, 2)); // 66.67%
    }

    #[test]
    fn test_stock_status() {
        let mut product = create_test_product();

        product.quantity = 0;
        assert_eq!(product.get_stock_status(), StockStatus::OutOfStock);

        product.quantity = 5; // Below min_stock_level (10)
        assert_eq!(product.get_stock_status(), StockStatus::LowStock);

        product.quantity = 50;
        assert_eq!(product.get_stock_status(), StockStatus::InStock);

        product.quantity = 1500; // Above max_stock_level (1000)
        assert_eq!(product.get_stock_status(), StockStatus::Overstocked);
    }

    #[test]
    fn test_quantity_adjustment() {
        let mut product = create_test_product();
        let initial_quantity = product.quantity;

        let movement = product.adjust_quantity(10, "Restock".to_string());

        assert_eq!(product.quantity, initial_quantity + 10);
        assert_eq!(movement.movement_type, StockMovementType::In);
        assert_eq!(movement.quantity, 10);
        assert_eq!(movement.reason, "Restock");

        let movement = product.adjust_quantity(-5, "Sale".to_string());
        assert_eq!(movement.movement_type, StockMovementType::Out);
        assert_eq!(movement.quantity, 5);
    }

    #[test]
    fn test_product_availability() {
        let mut product = create_test_product();

        assert!(product.is_available());

        product.quantity = 0;
        assert!(!product.is_available());

        product.quantity = 10;
        product.status = ProductStatus::Inactive;
        assert!(!product.is_available());
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
