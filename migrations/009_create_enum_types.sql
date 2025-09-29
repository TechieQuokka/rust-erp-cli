-- Migration: create_enum_types
-- Version: 009
-- Description: Create enum types for products

-- Create enum types for products
CREATE TYPE product_status AS ENUM ('active', 'inactive', 'discontinued', 'out_of_stock');

-- Update products table to use the new enum type
ALTER TABLE products ALTER COLUMN status TYPE product_status USING status::product_status;

-- DOWN
-- Revert products table to use varchar
ALTER TABLE products ALTER COLUMN status TYPE VARCHAR(20) USING status::varchar;

-- Drop enum types
DROP TYPE IF EXISTS product_status;

