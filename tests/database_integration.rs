use sqlx::Row;

mod common;
use common::TestContext;

#[tokio::test]
async fn test_database_connection() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Test that we can connect to the database
    let result = sqlx::query("SELECT 1 as test_value")
        .fetch_one(&ctx.db_pool)
        .await?;

    let test_value: i32 = result.get("test_value");
    assert_eq!(test_value, 1);

    Ok(())
}

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

    // Verify tables are empty initially
    assert_eq!(user_count, 0);
    assert_eq!(product_count, 0);
    assert_eq!(customer_count, 0);
    assert_eq!(order_count, 0);

    Ok(())
}

#[tokio::test]
async fn test_database_schema() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Test that all required tables exist by describing them
    let tables = vec!["users", "products", "customers", "orders", "order_items"];

    for table in tables {
        let table_info = sqlx::query(&format!("PRAGMA table_info({})", table))
            .fetch_all(&ctx.db_pool)
            .await?;

        assert!(
            !table_info.is_empty(),
            "Table {} should exist and have columns",
            table
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_basic_crud_operations() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Test basic INSERT operation
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&user_id)
    .bind("testuser")
    .bind("test@example.com")
    .bind("$2b$12$test_hash")
    .bind("user")
    .execute(&ctx.db_pool)
    .await?;

    // Test SELECT operation
    let user = sqlx::query("SELECT id, email, role FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&ctx.db_pool)
        .await?;

    assert_eq!(user.get::<String, _>("id"), user_id);
    assert_eq!(user.get::<String, _>("email"), "test@example.com");
    assert_eq!(user.get::<String, _>("role"), "user");

    // Test UPDATE operation
    sqlx::query("UPDATE users SET email = ? WHERE id = ?")
        .bind("updated@example.com")
        .bind(&user_id)
        .execute(&ctx.db_pool)
        .await?;

    // Verify update
    let updated_user = sqlx::query("SELECT email FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&ctx.db_pool)
        .await?;

    assert_eq!(
        updated_user.get::<String, _>("email"),
        "updated@example.com"
    );

    // Test DELETE operation
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&user_id)
        .execute(&ctx.db_pool)
        .await?;

    // Verify deletion
    let deleted_user = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&ctx.db_pool)
        .await?;

    assert!(deleted_user.is_none());

    Ok(())
}

#[tokio::test]
async fn test_database_constraints() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Test that we can't insert invalid data
    let product_id = uuid::Uuid::new_v4().to_string();

    // Insert a product with a specific SKU
    let result = sqlx::query(
        "INSERT INTO products (id, sku, name, description, category, quantity, price, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&product_id)
    .bind("TEST-001")
    .bind("Test Product")
    .bind("Test description")
    .bind("Electronics")
    .bind(10)
    .bind(29.99)
    .execute(&ctx.db_pool)
    .await;

    assert!(result.is_ok(), "First product insertion should succeed");

    // Try to insert another product with the same SKU - should fail
    let duplicate_product_id = uuid::Uuid::new_v4().to_string();
    let duplicate_result = sqlx::query(
        "INSERT INTO products (id, sku, name, description, category, quantity, price, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&duplicate_product_id)
    .bind("TEST-001")  // Same SKU
    .bind("Duplicate Product")
    .bind("Duplicate description")
    .bind("Electronics")
    .bind(20)
    .bind(39.99)
    .execute(&ctx.db_pool)
    .await;

    assert!(
        duplicate_result.is_err(),
        "Duplicate SKU insertion should fail"
    );

    Ok(())
}

#[tokio::test]
async fn test_transaction_rollback() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Begin a transaction
    let mut tx = ctx.db_pool.begin().await?;

    let user_id = uuid::Uuid::new_v4().to_string();

    // Insert a user within the transaction
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&user_id)
    .bind("transactionuser")
    .bind("transaction_test@example.com")
    .bind("$2b$12$test_hash")
    .bind("user")
    .execute(&mut *tx)
    .await?;

    // Rollback the transaction
    tx.rollback().await?;

    // Verify that the user was not actually inserted
    let user = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&ctx.db_pool)
        .await?;

    assert!(user.is_none(), "User should not exist after rollback");

    Ok(())
}
