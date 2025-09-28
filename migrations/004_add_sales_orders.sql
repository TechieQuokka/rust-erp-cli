-- Migration: add_sales_orders
-- Version: 004
-- Description: Create sales orders and order items tables

-- Note: SQLite doesn't support ENUM types, so we use VARCHAR with check constraints

-- Sales orders table
CREATE TABLE IF NOT EXISTS sales_orders (
    id VARCHAR(36) PRIMARY KEY,
    order_number VARCHAR(255) NOT NULL UNIQUE,
    customer_id VARCHAR(36) NOT NULL,
    order_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    total_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    tax_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    discount_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    shipping_address TEXT,
    billing_address TEXT,
    payment_method VARCHAR(50),
    payment_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    -- Note: Foreign key constraint temporarily disabled for SQLite compatibility
    -- FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT
);

-- Sales order items table
CREATE TABLE IF NOT EXISTS sales_order_items (
    id VARCHAR(36) PRIMARY KEY,
    order_id VARCHAR(36) NOT NULL,
    product_id VARCHAR(36) NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(15,2) NOT NULL CHECK (unit_price >= 0),
    discount DECIMAL(15,2) NOT NULL DEFAULT 0.00 CHECK (discount >= 0),
    line_total DECIMAL(15,2) NOT NULL CHECK (line_total >= 0),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    -- Note: Foreign key constraints temporarily disabled for SQLite compatibility
    -- FOREIGN KEY (order_id) REFERENCES sales_orders(id) ON DELETE CASCADE,
    -- FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT
);

-- Create indexes for sales_orders table
CREATE INDEX IF NOT EXISTS idx_sales_orders_customer_id ON sales_orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_sales_orders_order_date ON sales_orders(order_date);
CREATE INDEX IF NOT EXISTS idx_sales_orders_status ON sales_orders(status);
CREATE INDEX IF NOT EXISTS idx_sales_orders_payment_status ON sales_orders(payment_status);
CREATE INDEX IF NOT EXISTS idx_sales_orders_created_at ON sales_orders(created_at);
CREATE INDEX IF NOT EXISTS idx_sales_orders_order_number ON sales_orders(order_number);

-- Create indexes for sales_order_items table
CREATE INDEX IF NOT EXISTS idx_sales_order_items_order_id ON sales_order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_sales_order_items_product_id ON sales_order_items(product_id);

-- DOWN
-- Rollback migration
DROP INDEX IF EXISTS idx_sales_order_items_product_id;
DROP INDEX IF EXISTS idx_sales_order_items_order_id;
DROP INDEX IF EXISTS idx_sales_orders_order_number;
DROP INDEX IF EXISTS idx_sales_orders_created_at;
DROP INDEX IF EXISTS idx_sales_orders_payment_status;
DROP INDEX IF EXISTS idx_sales_orders_status;
DROP INDEX IF EXISTS idx_sales_orders_order_date;
DROP INDEX IF EXISTS idx_sales_orders_customer_id;

DROP TABLE IF EXISTS sales_order_items;
DROP TABLE IF EXISTS sales_orders;

-- No enum types to drop in SQLite

