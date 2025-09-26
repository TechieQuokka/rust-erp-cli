use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub order_number: String,
    pub customer_id: Uuid,
    pub user_id: Uuid,
    pub status: OrderStatus,
    pub payment_status: PaymentStatus,
    pub shipping_status: ShippingStatus,
    pub order_date: DateTime<Utc>,
    pub required_date: Option<DateTime<Utc>>,
    pub shipped_date: Option<DateTime<Utc>>,
    pub shipping_address: String,
    pub billing_address: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub shipping_cost: Decimal,
    pub discount_amount: Decimal,
    pub total_amount: Decimal,
    pub notes: Option<String>,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount_percentage: Decimal,
    pub discount_amount: Decimal,
    pub tax_percentage: Decimal,
    pub tax_amount: Decimal,
    pub line_total: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Draft,
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Returned,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Paid,
    PartiallyPaid,
    Overdue,
    Failed,
    Refunded,
    PartiallyRefunded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "shipping_status", rename_all = "lowercase")]
pub enum ShippingStatus {
    NotShipped,
    Processing,
    Shipped,
    InTransit,
    Delivered,
    Failed,
    Returned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: Uuid,
    pub required_date: Option<DateTime<Utc>>,
    pub shipping_address: String,
    pub billing_address: String,
    pub notes: Option<String>,
    pub items: Vec<CreateOrderItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderItemRequest {
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Option<Decimal>,
    pub discount_percentage: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderRequest {
    pub status: Option<OrderStatus>,
    pub payment_status: Option<PaymentStatus>,
    pub shipping_status: Option<ShippingStatus>,
    pub required_date: Option<DateTime<Utc>>,
    pub shipped_date: Option<DateTime<Utc>>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub order_number: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub status: OrderStatus,
    pub payment_status: PaymentStatus,
    pub shipping_status: ShippingStatus,
    pub order_date: DateTime<Utc>,
    pub required_date: Option<DateTime<Utc>>,
    pub shipped_date: Option<DateTime<Utc>>,
    pub shipping_address: String,
    pub billing_address: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub shipping_cost: Decimal,
    pub discount_amount: Decimal,
    pub total_amount: Decimal,
    pub notes: Option<String>,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub items: Vec<OrderItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount_percentage: Decimal,
    pub discount_amount: Decimal,
    pub tax_percentage: Decimal,
    pub tax_amount: Decimal,
    pub line_total: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFilter {
    pub status: Option<OrderStatus>,
    pub payment_status: Option<PaymentStatus>,
    pub shipping_status: Option<ShippingStatus>,
    pub customer_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub order_number: Option<String>,
    pub order_date_from: Option<DateTime<Utc>>,
    pub order_date_to: Option<DateTime<Utc>>,
    pub min_total: Option<Decimal>,
    pub max_total: Option<Decimal>,
}

impl Order {
    pub fn new(request: CreateOrderRequest, user_id: Uuid) -> Self {
        let now = Utc::now();
        let order_number = Self::generate_order_number();

        Self {
            id: Uuid::new_v4(),
            order_number,
            customer_id: request.customer_id,
            user_id,
            status: OrderStatus::Draft,
            payment_status: PaymentStatus::Pending,
            shipping_status: ShippingStatus::NotShipped,
            order_date: now,
            required_date: request.required_date,
            shipped_date: None,
            shipping_address: request.shipping_address,
            billing_address: request.billing_address,
            subtotal: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            shipping_cost: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            total_amount: Decimal::ZERO,
            notes: request.notes,
            payment_method: None,
            payment_reference: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, request: UpdateOrderRequest) {
        if let Some(status) = request.status {
            self.status = status;
        }
        if let Some(payment_status) = request.payment_status {
            self.payment_status = payment_status;
        }
        if let Some(shipping_status) = request.shipping_status {
            self.shipping_status = shipping_status;
        }
        if let Some(required_date) = request.required_date {
            self.required_date = Some(required_date);
        }
        if let Some(shipped_date) = request.shipped_date {
            self.shipped_date = Some(shipped_date);
        }
        if let Some(shipping_address) = request.shipping_address {
            self.shipping_address = shipping_address;
        }
        if let Some(billing_address) = request.billing_address {
            self.billing_address = billing_address;
        }
        if let Some(notes) = request.notes {
            self.notes = Some(notes);
        }
        if let Some(payment_method) = request.payment_method {
            self.payment_method = Some(payment_method);
        }
        if let Some(payment_reference) = request.payment_reference {
            self.payment_reference = Some(payment_reference);
        }
        self.updated_at = Utc::now();
    }

    pub fn calculate_totals(&mut self, items: &[OrderItem]) {
        self.subtotal = items.iter().map(|item| item.line_total).sum();
        self.discount_amount = items.iter().map(|item| item.discount_amount).sum();
        self.tax_amount = items.iter().map(|item| item.tax_amount).sum();
        self.total_amount =
            self.subtotal + self.tax_amount + self.shipping_cost - self.discount_amount;
        self.updated_at = Utc::now();
    }

    pub fn confirm(&mut self) {
        if self.status == OrderStatus::Draft {
            self.status = OrderStatus::Confirmed;
            self.updated_at = Utc::now();
        }
    }

    pub fn cancel(&mut self, reason: Option<String>) {
        if matches!(
            self.status,
            OrderStatus::Draft | OrderStatus::Pending | OrderStatus::Confirmed
        ) {
            self.status = OrderStatus::Cancelled;
            if let Some(reason) = reason {
                self.notes = Some(match &self.notes {
                    Some(existing) => format!("{}\nCancellation reason: {}", existing, reason),
                    None => format!("Cancellation reason: {}", reason),
                });
            }
            self.updated_at = Utc::now();
        }
    }

    pub fn ship(&mut self, shipping_date: DateTime<Utc>) {
        if matches!(
            self.status,
            OrderStatus::Confirmed | OrderStatus::Processing
        ) {
            self.status = OrderStatus::Shipped;
            self.shipping_status = ShippingStatus::Shipped;
            self.shipped_date = Some(shipping_date);
            self.updated_at = Utc::now();
        }
    }

    pub fn deliver(&mut self) {
        if self.status == OrderStatus::Shipped {
            self.status = OrderStatus::Delivered;
            self.shipping_status = ShippingStatus::Delivered;
            self.updated_at = Utc::now();
        }
    }

    pub fn is_editable(&self) -> bool {
        matches!(self.status, OrderStatus::Draft | OrderStatus::Pending)
    }

    pub fn is_cancellable(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Draft | OrderStatus::Pending | OrderStatus::Confirmed
        )
    }

    pub fn is_paid(&self) -> bool {
        self.payment_status == PaymentStatus::Paid
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(required_date) = self.required_date {
            Utc::now() > required_date && !self.is_paid()
        } else {
            false
        }
    }

    fn generate_order_number() -> String {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%d%H%M%S");
        let random_suffix: u32 = rand::random::<u32>() % 1000;
        format!("ORD-{}-{:03}", timestamp, random_suffix)
    }

    pub fn to_response(
        &self,
        customer_name: String,
        user_name: String,
        items: Vec<OrderItemResponse>,
    ) -> OrderResponse {
        OrderResponse {
            id: self.id,
            order_number: self.order_number.clone(),
            customer_id: self.customer_id,
            customer_name,
            user_id: self.user_id,
            user_name,
            status: self.status.clone(),
            payment_status: self.payment_status.clone(),
            shipping_status: self.shipping_status.clone(),
            order_date: self.order_date,
            required_date: self.required_date,
            shipped_date: self.shipped_date,
            shipping_address: self.shipping_address.clone(),
            billing_address: self.billing_address.clone(),
            subtotal: self.subtotal,
            tax_amount: self.tax_amount,
            shipping_cost: self.shipping_cost,
            discount_amount: self.discount_amount,
            total_amount: self.total_amount,
            notes: self.notes.clone(),
            payment_method: self.payment_method.clone(),
            payment_reference: self.payment_reference.clone(),
            items,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl OrderItem {
    pub fn new(
        order_id: Uuid,
        request: CreateOrderItemRequest,
        product: &crate::core::database::models::product::Product,
    ) -> Self {
        let now = Utc::now();
        let unit_price = request.unit_price.unwrap_or(product.price);
        let discount_percentage = request.discount_percentage.unwrap_or(Decimal::ZERO);
        let line_total_before_discount = unit_price * Decimal::from(request.quantity);
        let discount_amount =
            line_total_before_discount * (discount_percentage / Decimal::from(100));
        let line_total_after_discount = line_total_before_discount - discount_amount;

        let tax_percentage = if product.is_taxable {
            Decimal::from(10) // 10% tax rate - this should be configurable
        } else {
            Decimal::ZERO
        };

        let tax_amount = line_total_after_discount * (tax_percentage / Decimal::from(100));
        let line_total = line_total_after_discount + tax_amount;

        Self {
            id: Uuid::new_v4(),
            order_id,
            product_id: request.product_id,
            sku: product.sku.clone(),
            product_name: product.name.clone(),
            quantity: request.quantity,
            unit_price,
            discount_percentage,
            discount_amount,
            tax_percentage,
            tax_amount,
            line_total,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_quantity(&mut self, new_quantity: i32) {
        let _old_line_total = self.line_total;
        self.quantity = new_quantity;
        self.recalculate_totals();
        self.updated_at = Utc::now();
    }

    pub fn update_price(&mut self, new_price: Decimal) {
        self.unit_price = new_price;
        self.recalculate_totals();
        self.updated_at = Utc::now();
    }

    pub fn apply_discount(&mut self, discount_percentage: Decimal) {
        self.discount_percentage = discount_percentage;
        self.recalculate_totals();
        self.updated_at = Utc::now();
    }

    fn recalculate_totals(&mut self) {
        let line_total_before_discount = self.unit_price * Decimal::from(self.quantity);
        self.discount_amount =
            line_total_before_discount * (self.discount_percentage / Decimal::from(100));
        let line_total_after_discount = line_total_before_discount - self.discount_amount;
        self.tax_amount = line_total_after_discount * (self.tax_percentage / Decimal::from(100));
        self.line_total = line_total_after_discount + self.tax_amount;
    }

    pub fn to_response(&self) -> OrderItemResponse {
        OrderItemResponse {
            id: self.id,
            product_id: self.product_id,
            sku: self.sku.clone(),
            product_name: self.product_name.clone(),
            quantity: self.quantity,
            unit_price: self.unit_price,
            discount_percentage: self.discount_percentage,
            discount_amount: self.discount_amount,
            tax_percentage: self.tax_percentage,
            tax_amount: self.tax_amount,
            line_total: self.line_total,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl OrderStatus {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Draft,
            Self::Pending,
            Self::Confirmed,
            Self::Processing,
            Self::Shipped,
            Self::Delivered,
            Self::Cancelled,
            Self::Returned,
        ]
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Self::Delivered | Self::Cancelled | Self::Returned)
    }

    pub fn can_transition_to(&self, new_status: &OrderStatus) -> bool {
        match (self, new_status) {
            (Self::Draft, Self::Pending | Self::Confirmed | Self::Cancelled) => true,
            (Self::Pending, Self::Confirmed | Self::Cancelled) => true,
            (Self::Confirmed, Self::Processing | Self::Cancelled) => true,
            (Self::Processing, Self::Shipped | Self::Cancelled) => true,
            (Self::Shipped, Self::Delivered | Self::Returned) => true,
            (Self::Delivered, Self::Returned) => true,
            _ => false,
        }
    }
}

impl PaymentStatus {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Pending,
            Self::Paid,
            Self::PartiallyPaid,
            Self::Overdue,
            Self::Failed,
            Self::Refunded,
            Self::PartiallyRefunded,
        ]
    }

    pub fn is_paid(&self) -> bool {
        matches!(self, Self::Paid | Self::PartiallyPaid)
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Paid | Self::Failed | Self::Refunded | Self::PartiallyRefunded
        )
    }
}

impl ShippingStatus {
    pub fn all() -> Vec<Self> {
        vec![
            Self::NotShipped,
            Self::Processing,
            Self::Shipped,
            Self::InTransit,
            Self::Delivered,
            Self::Failed,
            Self::Returned,
        ]
    }

    pub fn is_shipped(&self) -> bool {
        matches!(self, Self::Shipped | Self::InTransit | Self::Delivered)
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Self::Delivered | Self::Failed | Self::Returned)
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Pending => write!(f, "pending"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::Processing => write!(f, "processing"),
            Self::Shipped => write!(f, "shipped"),
            Self::Delivered => write!(f, "delivered"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Returned => write!(f, "returned"),
        }
    }
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Paid => write!(f, "paid"),
            Self::PartiallyPaid => write!(f, "partially_paid"),
            Self::Overdue => write!(f, "overdue"),
            Self::Failed => write!(f, "failed"),
            Self::Refunded => write!(f, "refunded"),
            Self::PartiallyRefunded => write!(f, "partially_refunded"),
        }
    }
}

impl std::fmt::Display for ShippingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotShipped => write!(f, "not_shipped"),
            Self::Processing => write!(f, "processing"),
            Self::Shipped => write!(f, "shipped"),
            Self::InTransit => write!(f, "in_transit"),
            Self::Delivered => write!(f, "delivered"),
            Self::Failed => write!(f, "failed"),
            Self::Returned => write!(f, "returned"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::models::product::{CreateProductRequest, Product, ProductStatus};

    #[test]
    fn test_order_creation() {
        let customer_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let request = CreateOrderRequest {
            customer_id,
            required_date: None,
            shipping_address: "123 Main St, City".to_string(),
            billing_address: "123 Main St, City".to_string(),
            notes: Some("Test order".to_string()),
            items: vec![],
        };

        let order = Order::new(request, user_id);

        assert_eq!(order.customer_id, customer_id);
        assert_eq!(order.user_id, user_id);
        assert_eq!(order.status, OrderStatus::Draft);
        assert_eq!(order.payment_status, PaymentStatus::Pending);
        assert_eq!(order.shipping_status, ShippingStatus::NotShipped);
        assert!(order.order_number.starts_with("ORD-"));
    }

    #[test]
    fn test_order_status_transitions() {
        let draft = OrderStatus::Draft;
        let pending = OrderStatus::Pending;
        let confirmed = OrderStatus::Confirmed;
        let cancelled = OrderStatus::Cancelled;

        assert!(draft.can_transition_to(&pending));
        assert!(draft.can_transition_to(&confirmed));
        assert!(draft.can_transition_to(&cancelled));
        assert!(!draft.can_transition_to(&OrderStatus::Shipped));

        assert!(pending.can_transition_to(&confirmed));
        assert!(pending.can_transition_to(&cancelled));
        assert!(!pending.can_transition_to(&OrderStatus::Shipped));
    }

    #[test]
    fn test_order_item_creation() {
        let order_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let product_request = CreateProductRequest {
            sku: "TEST-001".to_string(),
            name: "Test Product".to_string(),
            description: None,
            category: "Test".to_string(),
            price: rust_decimal::Decimal::new(1000, 2), // 10.00
            cost: rust_decimal::Decimal::new(500, 2),   // 5.00
            quantity: 100,
            min_stock_level: 10,
            max_stock_level: None,
            is_taxable: true,
            weight: None,
            dimensions: None,
            barcode: None,
            supplier_id: None,
        };

        let product = Product::new(product_request);

        let item_request = CreateOrderItemRequest {
            product_id,
            quantity: 2,
            unit_price: None,
            discount_percentage: Some(rust_decimal::Decimal::new(10, 0)), // 10%
        };

        let order_item = OrderItem::new(order_id, item_request, &product);

        assert_eq!(order_item.order_id, order_id);
        assert_eq!(order_item.product_id, product_id);
        assert_eq!(order_item.quantity, 2);
        assert_eq!(order_item.unit_price, product.price);
        assert_eq!(
            order_item.discount_percentage,
            rust_decimal::Decimal::new(10, 0)
        );

        // Test calculations
        // unit_price * quantity = 10.00 * 2 = 20.00
        // discount = 20.00 * 10% = 2.00
        // after_discount = 20.00 - 2.00 = 18.00
        // tax = 18.00 * 10% = 1.80
        // total = 18.00 + 1.80 = 19.80

        let expected_discount = rust_decimal::Decimal::new(200, 2); // 2.00
        let expected_tax = rust_decimal::Decimal::new(180, 2); // 1.80
        let expected_total = rust_decimal::Decimal::new(1980, 2); // 19.80

        assert_eq!(order_item.discount_amount, expected_discount);
        assert_eq!(order_item.tax_amount, expected_tax);
        assert_eq!(order_item.line_total, expected_total);
    }

    #[test]
    fn test_order_item_recalculation() {
        let order_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        let product_request = CreateProductRequest {
            sku: "TEST-001".to_string(),
            name: "Test Product".to_string(),
            description: None,
            category: "Test".to_string(),
            price: rust_decimal::Decimal::new(1000, 2), // 10.00
            cost: rust_decimal::Decimal::new(500, 2),   // 5.00
            quantity: 100,
            min_stock_level: 10,
            max_stock_level: None,
            is_taxable: false, // No tax for simpler calculation
            weight: None,
            dimensions: None,
            barcode: None,
            supplier_id: None,
        };

        let product = Product::new(product_request);

        let item_request = CreateOrderItemRequest {
            product_id,
            quantity: 1,
            unit_price: None,
            discount_percentage: None,
        };

        let mut order_item = OrderItem::new(order_id, item_request, &product);

        // Initial: 1 * 10.00 = 10.00 (no discount, no tax)
        assert_eq!(order_item.line_total, rust_decimal::Decimal::new(1000, 2));

        // Update quantity to 3
        order_item.update_quantity(3);
        assert_eq!(order_item.quantity, 3);
        assert_eq!(order_item.line_total, rust_decimal::Decimal::new(3000, 2)); // 30.00

        // Apply 20% discount
        order_item.apply_discount(rust_decimal::Decimal::new(20, 0));
        assert_eq!(
            order_item.discount_percentage,
            rust_decimal::Decimal::new(20, 0)
        );
        // 30.00 - 20% = 24.00
        assert_eq!(order_item.line_total, rust_decimal::Decimal::new(2400, 2));
    }

    #[test]
    fn test_order_operations() {
        let customer_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let request = CreateOrderRequest {
            customer_id,
            required_date: Some(Utc::now() + chrono::Duration::days(7)),
            shipping_address: "123 Main St".to_string(),
            billing_address: "123 Main St".to_string(),
            notes: None,
            items: vec![],
        };

        let mut order = Order::new(request, user_id);

        // Test confirmation
        order.confirm();
        assert_eq!(order.status, OrderStatus::Confirmed);

        // Test shipping
        let ship_date = Utc::now();
        order.ship(ship_date);
        assert_eq!(order.status, OrderStatus::Shipped);
        assert_eq!(order.shipping_status, ShippingStatus::Shipped);
        assert_eq!(order.shipped_date, Some(ship_date));

        // Test delivery
        order.deliver();
        assert_eq!(order.status, OrderStatus::Delivered);
        assert_eq!(order.shipping_status, ShippingStatus::Delivered);
    }

    #[test]
    fn test_order_cancellation() {
        let customer_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let request = CreateOrderRequest {
            customer_id,
            required_date: None,
            shipping_address: "123 Main St".to_string(),
            billing_address: "123 Main St".to_string(),
            notes: Some("Original note".to_string()),
            items: vec![],
        };

        let mut order = Order::new(request, user_id);

        // Test cancellation with reason
        order.cancel(Some("Customer requested".to_string()));
        assert_eq!(order.status, OrderStatus::Cancelled);
        assert!(order
            .notes
            .as_ref()
            .unwrap()
            .contains("Cancellation reason: Customer requested"));
        assert!(order.notes.as_ref().unwrap().contains("Original note"));
    }
}
