-- Test data for ERP system

-- Users
INSERT OR IGNORE INTO users (id, email, password_hash, role, created_at, updated_at) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'admin@test.com', '$2b$12$example_hash_admin', 'admin', datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440002', 'user@test.com', '$2b$12$example_hash_user', 'user', datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440003', 'manager@test.com', '$2b$12$example_hash_manager', 'manager', datetime('now'), datetime('now'));

-- Customers
INSERT OR IGNORE INTO customers (id, name, email, phone, address, created_at, updated_at) VALUES
('550e8400-e29b-41d4-a716-446655440010', 'Samsung Electronics', 'samsung@test.com', '02-1234-5678', 'Seoul, Gangnam-gu', datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440011', 'LG Corporation', 'lg@test.com', '02-2345-6789', 'Seoul, Yeongdeungpo-gu', datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440012', 'Hyundai Motor', 'hyundai@test.com', '02-3456-7890', 'Seoul, Gangbuk-gu', datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440013', 'POSCO Holdings', 'posco@test.com', '052-4567-8901', 'Pohang, Gyeongbuk', datetime('now'), datetime('now'));

-- Products
INSERT OR IGNORE INTO products (id, sku, name, description, category, quantity, unit_price, created_at, updated_at) VALUES
('550e8400-e29b-41d4-a716-446655440020', 'ELEC-001', 'Galaxy Smartphone', 'Premium smartphone with advanced features', 'Electronics', 150, 899.99, datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440021', 'ELEC-002', 'OLED TV 55inch', '4K OLED Smart TV', 'Electronics', 75, 1299.99, datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440022', 'AUTO-001', 'Engine Parts Set', 'Complete engine maintenance kit', 'Automotive', 200, 450.00, datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440023', 'STEEL-001', 'Steel Coil Grade A', 'High quality steel coil for construction', 'Materials', 50, 2500.00, datetime('now'), datetime('now')),
('550e8400-e29b-41d4-a716-446655440024', 'ELEC-003', 'Wireless Headphones', 'Noise-cancelling wireless headphones', 'Electronics', 300, 199.99, datetime('now'), datetime('now'));

-- Orders
INSERT OR IGNORE INTO orders (id, customer_id, status, total_amount, created_at, updated_at) VALUES
('550e8400-e29b-41d4-a716-446655440030', '550e8400-e29b-41d4-a716-446655440010', 'completed', 1799.98, datetime('now', '-7 days'), datetime('now', '-5 days')),
('550e8400-e29b-41d4-a716-446655440031', '550e8400-e29b-41d4-a716-446655440011', 'pending', 1299.99, datetime('now', '-2 days'), datetime('now', '-2 days')),
('550e8400-e29b-41d4-a716-446655440032', '550e8400-e29b-41d4-a716-446655440012', 'processing', 900.00, datetime('now', '-1 days'), datetime('now', '-1 days'));

-- Order Items
INSERT OR IGNORE INTO order_items (id, order_id, product_id, quantity, unit_price, total_price) VALUES
('550e8400-e29b-41d4-a716-446655440040', '550e8400-e29b-41d4-a716-446655440030', '550e8400-e29b-41d4-a716-446655440020', 2, 899.99, 1799.98),
('550e8400-e29b-41d4-a716-446655440041', '550e8400-e29b-41d4-a716-446655440031', '550e8400-e29b-41d4-a716-446655440021', 1, 1299.99, 1299.99),
('550e8400-e29b-41d4-a716-446655440042', '550e8400-e29b-41d4-a716-446655440032', '550e8400-e29b-41d4-a716-446655440022', 2, 450.00, 900.00);

-- Configuration settings
INSERT OR IGNORE INTO app_config (key, value, description, created_at, updated_at) VALUES
('company_name', 'Test ERP Corp', 'Company name for testing', datetime('now'), datetime('now')),
('currency', 'USD', 'Default currency', datetime('now'), datetime('now')),
('tax_rate', '0.1', 'Default tax rate (10%)', datetime('now'), datetime('now')),
('low_stock_threshold', '20', 'Alert when stock below this level', datetime('now'), datetime('now')),
('invoice_prefix', 'INV', 'Invoice number prefix', datetime('now'), datetime('now'));