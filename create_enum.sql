-- Create enum types for PostgreSQL compatibility
-- This fixes the product_status enum type mismatch

BEGIN;

-- Drop existing types if they exist (to handle recreating)
DROP TYPE IF EXISTS product_status CASCADE;
DROP TYPE IF EXISTS stock_movement_type CASCADE;
DROP TYPE IF EXISTS customer_type CASCADE;

-- Create enum types
CREATE TYPE product_status AS ENUM ('active', 'inactive', 'discontinued', 'out_of_stock');
CREATE TYPE stock_movement_type AS ENUM ('in', 'out', 'adjustment');
CREATE TYPE customer_type AS ENUM ('individual', 'business');

-- Update products table to use enum type for status (remove default, convert type, restore default)
ALTER TABLE products ALTER COLUMN status DROP DEFAULT;
ALTER TABLE products ALTER COLUMN status TYPE product_status USING status::product_status;
ALTER TABLE products ALTER COLUMN status SET DEFAULT 'active'::product_status;

-- Update stock_movements table to use enum type for movement_type
ALTER TABLE stock_movements
ALTER COLUMN movement_type TYPE stock_movement_type USING movement_type::stock_movement_type;

-- Update customers table to use enum type for customer_type (if column exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'customers' AND column_name = 'customer_type') THEN
        ALTER TABLE customers
        ALTER COLUMN customer_type TYPE customer_type USING customer_type::customer_type;
    END IF;
END $$;

COMMIT;
