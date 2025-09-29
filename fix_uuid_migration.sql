-- Fix UUID Migration - Drop constraints, convert types, recreate constraints
-- This script will properly convert VARCHAR UUID columns to proper UUID type

BEGIN;

-- Step 1: Drop all foreign key constraints that reference UUID columns
ALTER TABLE stock_movements DROP CONSTRAINT IF EXISTS stock_movements_product_id_fkey;
ALTER TABLE sales_order_items DROP CONSTRAINT IF EXISTS sales_order_items_order_id_fkey;
ALTER TABLE sales_order_items DROP CONSTRAINT IF EXISTS sales_order_items_product_id_fkey;
ALTER TABLE sales_orders DROP CONSTRAINT IF EXISTS sales_orders_customer_id_fkey;
ALTER TABLE customer_addresses DROP CONSTRAINT IF EXISTS customer_addresses_customer_id_fkey;

-- Step 2: Convert primary key columns first
ALTER TABLE products ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE sales_orders ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE sales_order_items ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE stock_movements ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE customers ALTER COLUMN id TYPE UUID USING id::UUID;
ALTER TABLE customer_addresses ALTER COLUMN id TYPE UUID USING id::UUID;

-- Step 3: Convert foreign key columns
ALTER TABLE products ALTER COLUMN supplier_id TYPE UUID USING supplier_id::UUID;
ALTER TABLE sales_orders ALTER COLUMN customer_id TYPE UUID USING customer_id::UUID;
ALTER TABLE sales_order_items ALTER COLUMN order_id TYPE UUID USING order_id::UUID;
ALTER TABLE sales_order_items ALTER COLUMN product_id TYPE UUID USING product_id::UUID;
ALTER TABLE stock_movements ALTER COLUMN product_id TYPE UUID USING product_id::UUID;
ALTER TABLE stock_movements ALTER COLUMN reference_id TYPE UUID USING reference_id::UUID;
ALTER TABLE stock_movements ALTER COLUMN user_id TYPE UUID USING user_id::UUID;
ALTER TABLE customer_addresses ALTER COLUMN customer_id TYPE UUID USING customer_id::UUID;

-- Step 4: Recreate foreign key constraints
ALTER TABLE stock_movements ADD CONSTRAINT stock_movements_product_id_fkey FOREIGN KEY (product_id) REFERENCES products(id);
ALTER TABLE sales_order_items ADD CONSTRAINT sales_order_items_order_id_fkey FOREIGN KEY (order_id) REFERENCES sales_orders(id);
ALTER TABLE sales_order_items ADD CONSTRAINT sales_order_items_product_id_fkey FOREIGN KEY (product_id) REFERENCES products(id);
ALTER TABLE sales_orders ADD CONSTRAINT sales_orders_customer_id_fkey FOREIGN KEY (customer_id) REFERENCES customers(id);
ALTER TABLE customer_addresses ADD CONSTRAINT customer_addresses_customer_id_fkey FOREIGN KEY (customer_id) REFERENCES customers(id);

COMMIT;