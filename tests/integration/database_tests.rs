use std::sync::Arc;
use uuid::Uuid;
use rust_decimal::Decimal;
use sqlx::{Pool, Sqlite};

use erp_cli::core::database::connection::DatabaseManager;
use erp_cli::core::database::models::{User, Product, Customer, Order, OrderItem};
use erp_cli::modules::inventory::{
    repository::{InventoryRepository, SqliteInventoryRepository},
    models::{CreateProductRequest, UpdateProductRequest},
};
use erp_cli::modules::customers::{
    repository::{CustomerRepository, SqliteCustomerRepository},
    models::{CreateCustomerRequest, UpdateCustomerRequest},
};
use erp_cli::modules::sales::{
    repository::{SalesRepository, SqliteSalesRepository},
    models::{CreateOrderRequest, OrderItemRequest},
};

use crate::common::TestContext;

#[tokio::test]
async fn test_database_migrations() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Check if tables exist by querying them
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&ctx.db_pool)
        .await?;

    let product_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products")
        .fetch_one(&ctx.db_pool)
        .await?;

    let customer_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM customers")
        .fetch_one(&ctx.db_pool)
        .await?;

    let order_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(&ctx.db_pool)
        .await?;

    assert_eq!(user_count, 0);
    assert_eq!(product_count, 0);
    assert_eq!(customer_count, 0);
    assert_eq!(order_count, 0);

    Ok(())
}

#[tokio::test]
async fn test_inventory_repository_crud() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;
    let repository = SqliteInventoryRepository::new(Arc::new(ctx.db_pool.clone()));

    // Create a product
    let create_request = CreateProductRequest {
        sku: "TEST-001".to_string(),
        name: "Test Product".to_string(),
        description: Some("Test Description".to_string()),
        category: "Electronics".to_string(),
        quantity: 100,
        unit_price: Decimal::new(2999, 2), // 29.99
    };

    let product = repository.create_product(create_request).await?;
    assert_eq!(product.sku, "TEST-001");
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.quantity, 100);

    // Read the product
    let retrieved_product = repository.get_product_by_id(&product.id).await?;
    assert!(retrieved_product.is_some());
    assert_eq!(retrieved_product.unwrap().sku, "TEST-001");

    // Update the product
    let update_request = UpdateProductRequest {
        name: Some("Updated Product".to_string()),
        description: Some("Updated Description".to_string()),
        category: Some("Updated Electronics".to_string()),
        quantity: Some(150),
        unit_price: Some(Decimal::new(3999, 2)), // 39.99
    };

    let updated_product = repository.update_product(&product.id, update_request).await?;
    assert!(updated_product.is_some());
    let updated = updated_product.unwrap();
    assert_eq!(updated.name, "Updated Product");
    assert_eq!(updated.quantity, 150);

    // List products
    let products = repository.list_products(None, None, 10, 0).await?;
    assert_eq!(products.len(), 1);

    // Delete the product
    let deleted = repository.delete_product(&product.id).await?;
    assert!(deleted);

    let should_be_none = repository.get_product_by_id(&product.id).await?;
    assert!(should_be_none.is_none());

    Ok(())
}

#[tokio::test]
async fn test_customer_repository_crud() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;
    let repository = SqliteCustomerRepository::new(Arc::new(ctx.db_pool.clone()));

    // Create a customer
    let create_request = CreateCustomerRequest {
        name: "Test Customer".to_string(),
        email: Some("test@customer.com".to_string()),
        phone: Some("010-1234-5678".to_string()),
        address: Some("Seoul, Korea".to_string()),
    };

    let customer = repository.create_customer(create_request).await?;
    assert_eq!(customer.name, "Test Customer");
    assert_eq!(customer.email, Some("test@customer.com".to_string()));

    // Read the customer
    let retrieved_customer = repository.get_customer_by_id(&customer.id).await?;
    assert!(retrieved_customer.is_some());
    assert_eq!(retrieved_customer.unwrap().name, "Test Customer");

    // Update the customer
    let update_request = UpdateCustomerRequest {
        name: Some("Updated Customer".to_string()),
        email: Some("updated@customer.com".to_string()),
        phone: Some("010-9876-5432".to_string()),
        address: Some("Busan, Korea".to_string()),
    };

    let updated_customer = repository.update_customer(&customer.id, update_request).await?;
    assert!(updated_customer.is_some());
    let updated = updated_customer.unwrap();
    assert_eq!(updated.name, "Updated Customer");
    assert_eq!(updated.email, Some("updated@customer.com".to_string()));

    // List customers
    let customers = repository.list_customers(10, 0).await?;
    assert_eq!(customers.len(), 1);

    // Search customers
    let search_results = repository.search_customers("Updated").await?;
    assert_eq!(search_results.len(), 1);

    // Delete the customer
    let deleted = repository.delete_customer(&customer.id).await?;
    assert!(deleted);

    let should_be_none = repository.get_customer_by_id(&customer.id).await?;
    assert!(should_be_none.is_none());

    Ok(())
}

#[tokio::test]
async fn test_sales_repository_crud() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;
    ctx.seed_test_data().await?;

    let repository = SqliteSalesRepository::new(Arc::new(ctx.db_pool.clone()));

    // Get test data IDs
    let customer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003")?;
    let product_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440005")?;

    // Create an order
    let order_items = vec![OrderItemRequest {
        product_id,
        quantity: 2,
        unit_price: Decimal::new(2999, 2), // 29.99
    }];

    let create_request = CreateOrderRequest {
        customer_id,
        items: order_items,
    };

    let order = repository.create_order(create_request).await?;
    assert_eq!(order.customer_id, customer_id);
    assert_eq!(order.status, "pending");

    // Read the order
    let retrieved_order = repository.get_order_by_id(&order.id).await?;
    assert!(retrieved_order.is_some());
    assert_eq!(retrieved_order.unwrap().customer_id, customer_id);

    // Update order status
    let updated_order = repository.update_order_status(&order.id, "processing").await?;
    assert!(updated_order.is_some());
    assert_eq!(updated_order.unwrap().status, "processing");

    // List orders
    let orders = repository.list_orders(10, 0).await?;
    assert!(!orders.is_empty());

    // Get order items
    let items = repository.get_order_items(&order.id).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].quantity, 2);

    Ok(())
}

#[tokio::test]
async fn test_database_constraints() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Test unique constraint on SKU
    let inventory_repo = SqliteInventoryRepository::new(Arc::new(ctx.db_pool.clone()));

    let create_request1 = CreateProductRequest {
        sku: "UNIQUE-001".to_string(),
        name: "Product 1".to_string(),
        description: None,
        category: "Test".to_string(),
        quantity: 10,
        unit_price: Decimal::new(1000, 2),
    };

    let create_request2 = CreateProductRequest {
        sku: "UNIQUE-001".to_string(), // Same SKU
        name: "Product 2".to_string(),
        description: None,
        category: "Test".to_string(),
        quantity: 20,
        unit_price: Decimal::new(2000, 2),
    };

    // First product should be created successfully
    let product1 = inventory_repo.create_product(create_request1).await?;
    assert_eq!(product1.sku, "UNIQUE-001");

    // Second product with same SKU should fail
    let result = inventory_repo.create_product(create_request2).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_transaction_rollback() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;
    ctx.seed_test_data().await?;

    let repository = SqliteSalesRepository::new(Arc::new(ctx.db_pool.clone()));

    // Get test data
    let customer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003")?;
    let invalid_product_id = Uuid::new_v4(); // Non-existent product

    let order_items = vec![OrderItemRequest {
        product_id: invalid_product_id,
        quantity: 1,
        unit_price: Decimal::new(1000, 2),
    }];

    let create_request = CreateOrderRequest {
        customer_id,
        items: order_items,
    };

    // This should fail due to foreign key constraint
    let result = repository.create_order(create_request).await;
    assert!(result.is_err());

    // Verify no partial order was created
    let orders = repository.list_orders(100, 0).await?;
    // Should only have the seeded test orders
    let new_orders: Vec<_> = orders.into_iter()
        .filter(|o| o.customer_id == customer_id)
        .collect();
    assert_eq!(new_orders.len(), 0); // No new orders should exist

    Ok(())
}