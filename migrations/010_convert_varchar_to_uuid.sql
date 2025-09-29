-- Migration: convert_varchar_to_uuid
-- Version: 010
-- Description: Convert VARCHAR(36) ID columns to proper UUID type

-- First, let's backup the existing data and convert the primary tables

-- 1. Convert products table
ALTER TABLE products ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE products ALTER COLUMN supplier_id TYPE UUID USING supplier_id::UUID;

-- 2. Convert sales_orders table
ALTER TABLE sales_orders ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE sales_orders ALTER COLUMN customer_id TYPE UUID USING customer_id::UUID;
ALTER TABLE sales_orders ALTER COLUMN shipping_address_id TYPE UUID USING shipping_address_id::UUID;
ALTER TABLE sales_orders ALTER COLUMN billing_address_id TYPE UUID USING billing_address_id::UUID;

-- 3. Convert sales_order_items table
ALTER TABLE sales_order_items ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE sales_order_items ALTER COLUMN order_id TYPE UUID USING order_id::UUID;
ALTER TABLE sales_order_items ALTER COLUMN product_id TYPE UUID USING product_id::UUID;

-- 4. Convert stock_movements table
ALTER TABLE stock_movements ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE stock_movements ALTER COLUMN product_id TYPE UUID USING product_id::UUID;
ALTER TABLE stock_movements ALTER COLUMN reference_id TYPE UUID USING reference_id::UUID;
ALTER TABLE stock_movements ALTER COLUMN user_id TYPE UUID USING user_id::UUID;

-- 5. Convert customers table if it exists
ALTER TABLE customers ALTER COLUMN id TYPE UUID USING id::UUID;

-- 6. Convert customer_addresses table if it exists
ALTER TABLE customer_addresses ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE customer_addresses ALTER COLUMN customer_id TYPE UUID USING customer_id::UUID;

-- DOWN
-- Rollback by converting back to VARCHAR(36)
ALTER TABLE products ALTER COLUMN id TYPE VARCHAR(36) USING id::TEXT;
ALTER TABLE products ALTER COLUMN supplier_id TYPE VARCHAR(36) USING supplier_id::TEXT;

ALTER TABLE sales_orders ALTER COLUMN id TYPE VARCHAR(36) USING id::TEXT;
ALTER TABLE sales_orders ALTER COLUMN customer_id TYPE VARCHAR(36) USING customer_id::TEXT;
ALTER TABLE sales_orders ALTER COLUMN shipping_address_id TYPE VARCHAR(36) USING shipping_address_id::TEXT;
ALTER TABLE sales_orders ALTER COLUMN billing_address_id TYPE VARCHAR(36) USING billing_address_id::TEXT;

ALTER TABLE sales_order_items ALTER COLUMN id TYPE VARCHAR(36) USING id::TEXT;
ALTER TABLE sales_order_items ALTER COLUMN order_id TYPE VARCHAR(36) USING order_id::TEXT;
ALTER TABLE sales_order_items ALTER COLUMN product_id TYPE VARCHAR(36) USING product_id::TEXT;

ALTER TABLE stock_movements ALTER COLUMN id TYPE VARCHAR(36) USING id::TEXT;
ALTER TABLE stock_movements ALTER COLUMN product_id TYPE VARCHAR(36) USING product_id::TEXT;
ALTER TABLE stock_movements ALTER COLUMN reference_id TYPE VARCHAR(36) USING reference_id::TEXT;
ALTER TABLE stock_movements ALTER COLUMN user_id TYPE VARCHAR(36) USING user_id::TEXT;

ALTER TABLE customers ALTER COLUMN id TYPE VARCHAR(36) USING id::TEXT;

ALTER TABLE customer_addresses ALTER COLUMN id TYPE VARCHAR(36) USING id::TEXT;
ALTER TABLE customer_addresses ALTER COLUMN customer_id TYPE VARCHAR(36) USING customer_id::TEXT;