//! Utility to create a test order for invoice generation testing
use chrono::Utc;
use erp_cli::core::config::AppConfig;
use erp_cli::core::database::connection::DatabaseManager;
use rust_decimal::Decimal;
use sqlx::Row;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Load configuration
    let config = AppConfig::load().await?;

    // Initialize database connection
    DatabaseManager::initialize(config.database.clone()).await?;
    let connection = DatabaseManager::get_connection().await?;
    let pool = connection.pool();

    // Test data
    let order_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001")?;
    let item_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002")?;
    // Get an actual product ID from the database
    let product_row = sqlx::query("SELECT id FROM products LIMIT 1")
        .fetch_one(pool)
        .await?;

    let product_id = Uuid::parse_str(&product_row.get::<String, _>("id"))?;
    println!("✓ Found product: {}", product_id);

    // Find customer by customer code - we need to get the actual customer ID
    // Let's just get the first customer from the database
    let customer_row = sqlx::query("SELECT id FROM customers LIMIT 1")
        .fetch_one(pool)
        .await?;

    let customer_id = Uuid::parse_str(&customer_row.get::<String, _>("id"))?;
    println!("✓ Found customer: {}", customer_id);

    let now = Utc::now();

    // Insert sales order using raw query to avoid enum issues
    let result = sqlx::query(
        r#"
        INSERT INTO sales_orders (
            id, order_number, customer_id, order_date, status,
            total_amount, tax_amount, discount_amount,
            shipping_address, billing_address, payment_method, payment_status,
            notes, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#,
    )
    .bind(order_id.to_string())
    .bind("DEV-ORD-TEST-001")
    .bind(customer_id.to_string())
    .bind(now)
    .bind("confirmed") // Use confirmed status for invoice generation
    .bind(Decimal::new(13200, 2)) // 132.00
    .bind(Decimal::new(1200, 2)) // 12.00
    .bind(Decimal::new(0, 2)) // 0.00
    .bind("123 Test Street, Test City, Test State, 12345")
    .bind("123 Test Street, Test City, Test State, 12345")
    .bind("creditcard")
    .bind("pending")
    .bind("Test order created for invoice generation testing")
    .bind(now)
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(_) => println!("✓ Sales order created successfully"),
        Err(e) => {
            eprintln!("Failed to create sales order: {}", e);
            return Err(e.into());
        }
    }

    // Insert order item
    let result = sqlx::query(
        r#"
        INSERT INTO sales_order_items (
            id, order_id, product_id, quantity, unit_price, discount, line_total, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(item_id.to_string())
    .bind(order_id.to_string())
    .bind(product_id.to_string())
    .bind(2_i32)
    .bind(Decimal::new(6000, 2)) // 60.00
    .bind(Decimal::new(0, 2)) // 0.00
    .bind(Decimal::new(12000, 2)) // 120.00
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(_) => println!("✓ Order item created successfully"),
        Err(e) => {
            eprintln!("Failed to create order item: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🎉 Test order created successfully!");
    println!("Order ID: {}", order_id);
    println!("\nYou can now test invoice generation with:");
    println!("cargo run -- sales generate-invoice {}", order_id);

    Ok(())
}
