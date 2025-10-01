use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrder {
    pub id: Uuid,
    pub order_number: String,
    pub customer_id: Uuid,
    pub order_date: DateTime<Utc>,
    pub status: OrderStatus,
    pub total_amount: Decimal,
    pub tax_amount: Decimal,
    pub discount_amount: Decimal,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub payment_method: Option<PaymentMethod>,
    pub payment_status: PaymentStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesOrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub discount: Decimal,
    pub line_total: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
#[derive(Default)]
pub enum OrderStatus {
    #[default]
    Draft,
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Returned,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_method", rename_all = "lowercase")]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    DebitCard,
    BankTransfer,
    Check,
    PayPal,
    Crypto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
#[derive(Default)]
pub enum PaymentStatus {
    #[default]
    Pending,
    Paid,
    PartiallyPaid,
    Overdue,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: Uuid,
    pub items: Vec<OrderItemRequest>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub payment_method: Option<PaymentMethod>,
    pub notes: Option<String>,
    pub discount_amount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemRequest {
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Option<Decimal>,
    pub discount: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderRequest {
    pub status: Option<OrderStatus>,
    pub payment_status: Option<PaymentStatus>,
    pub payment_method: Option<PaymentMethod>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderSummary {
    pub order: SalesOrder,
    pub items: Vec<OrderItemWithProduct>,
    pub subtotal: Decimal,
    pub total_discount: Decimal,
    pub tax_amount: Decimal,
    pub grand_total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemWithProduct {
    pub item: SalesOrderItem,
    pub product_name: String,
    pub product_sku: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub order_id: Uuid,
    pub invoice_number: String,
    pub issue_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub order_summary: OrderSummary,
    pub customer_info: CustomerInfo,
    pub company_info: CompanyInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInfo {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub billing_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub tax_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesStatistics {
    pub total_orders: i64,
    pub total_revenue: Decimal,
    pub average_order_value: Decimal,
    pub orders_by_status: Vec<(OrderStatus, i64)>,
    pub top_customers: Vec<TopCustomer>,
    pub top_products: Vec<TopProduct>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopCustomer {
    pub customer_id: Uuid,
    pub customer_name: String,
    pub total_orders: i64,
    pub total_spent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopProduct {
    pub product_id: Uuid,
    pub product_name: String,
    pub product_sku: String,
    pub quantity_sold: i64,
    pub revenue: Decimal,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Draft => write!(f, "Draft"),
            OrderStatus::Pending => write!(f, "Pending"),
            OrderStatus::Confirmed => write!(f, "Confirmed"),
            OrderStatus::Processing => write!(f, "Processing"),
            OrderStatus::Shipped => write!(f, "Shipped"),
            OrderStatus::Delivered => write!(f, "Delivered"),
            OrderStatus::Cancelled => write!(f, "Cancelled"),
            OrderStatus::Returned => write!(f, "Returned"),
        }
    }
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "Cash"),
            PaymentMethod::CreditCard => write!(f, "Credit Card"),
            PaymentMethod::DebitCard => write!(f, "Debit Card"),
            PaymentMethod::BankTransfer => write!(f, "Bank Transfer"),
            PaymentMethod::Check => write!(f, "Check"),
            PaymentMethod::PayPal => write!(f, "PayPal"),
            PaymentMethod::Crypto => write!(f, "Cryptocurrency"),
        }
    }
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "Pending"),
            PaymentStatus::Paid => write!(f, "Paid"),
            PaymentStatus::PartiallyPaid => write!(f, "Partially Paid"),
            PaymentStatus::Overdue => write!(f, "Overdue"),
            PaymentStatus::Failed => write!(f, "Failed"),
            PaymentStatus::Refunded => write!(f, "Refunded"),
        }
    }
}
