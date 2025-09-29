use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

pub struct TestContext {
    pub _temp_dir: TempDir,
    pub db_pool: Pool<Sqlite>,
    pub _config_path: String,
}

impl TestContext {
    pub async fn new() -> anyhow::Result<Self> {
        INIT.call_once(|| {
            tracing_subscriber::fmt::init();
        });

        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite://{}", db_path.display());

        // Create test database
        let db_pool = SqlitePool::connect(&db_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&db_pool).await?;

        let config_path = temp_dir
            .path()
            .join("config.toml")
            .to_string_lossy()
            .to_string();

        Ok(TestContext {
            _temp_dir: temp_dir,
            db_pool,
            _config_path: config_path,
        })
    }

    pub async fn _seed_test_data(&self) -> anyhow::Result<()> {
        // Add test users
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, role, created_at, updated_at)
            VALUES
                ('550e8400-e29b-41d4-a716-446655440001', 'admin', 'admin@test.com', '$2b$12$example_hash', 'admin', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('550e8400-e29b-41d4-a716-446655440002', 'testuser', 'user@test.com', '$2b$12$example_hash', 'user', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#
        )
        .execute(&self.db_pool)
        .await?;

        // Add test customers
        sqlx::query(
            r#"
            INSERT INTO customers (id, name, email, phone, created_at, updated_at)
            VALUES
                ('550e8400-e29b-41d4-a716-446655440003', 'Test Customer 1', 'customer1@test.com', '010-1234-5678', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('550e8400-e29b-41d4-a716-446655440004', 'Test Customer 2', 'customer2@test.com', '010-2345-6789', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#
        )
        .execute(&self.db_pool)
        .await?;

        // Add test products
        sqlx::query(
            r#"
            INSERT INTO products (id, sku, name, description, category, quantity, price, created_at, updated_at)
            VALUES
                ('550e8400-e29b-41d4-a716-446655440005', 'TEST-001', 'Test Product 1', 'Test product description 1', 'Electronics', 100, 29.99, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('550e8400-e29b-41d4-a716-446655440006', 'TEST-002', 'Test Product 2', 'Test product description 2', 'Clothing', 50, 19.99, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

// Test data constants
pub mod fixtures {
    use rust_decimal::Decimal;
    use uuid::Uuid;

    pub const _TEST_USER_ID: &str = "550e8400-e29b-41d4-a716-446655440001";
    pub const _TEST_ADMIN_EMAIL: &str = "admin@test.com";
    pub const _TEST_USER_EMAIL: &str = "user@test.com";

    pub const _TEST_CUSTOMER_ID: &str = "550e8400-e29b-41d4-a716-446655440003";
    pub const _TEST_CUSTOMER_NAME: &str = "Test Customer 1";
    pub const _TEST_CUSTOMER_EMAIL: &str = "customer1@test.com";

    pub const _TEST_PRODUCT_ID: &str = "550e8400-e29b-41d4-a716-446655440005";
    pub const _TEST_PRODUCT_SKU: &str = "TEST-001";
    pub const _TEST_PRODUCT_NAME: &str = "Test Product 1";

    pub fn _test_decimal(value: &str) -> Decimal {
        value.parse().expect("Invalid decimal")
    }

    pub fn _test_uuid(id: &str) -> Uuid {
        id.parse().expect("Invalid UUID")
    }
}

// Custom assertion macros for better error messages
#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        assert!(
            $haystack.contains($needle),
            "Expected '{}' to contain '{}' but it didn't",
            $haystack,
            $needle
        );
    };
}

#[macro_export]
macro_rules! assert_decimal_eq {
    ($left:expr, $right:expr) => {
        assert_eq!(
            $left.normalize(),
            $right.normalize(),
            "Decimal values don't match: {} != {}",
            $left,
            $right
        );
    };
}
