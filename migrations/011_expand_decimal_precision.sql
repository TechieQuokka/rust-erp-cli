-- Expand DECIMAL precision for price and cost columns
-- Version: 011
-- Description: Expand DECIMAL(10,2) to DECIMAL(14,2) for all monetary columns to maximize range

-- Expand products table monetary columns
ALTER TABLE products
    ALTER COLUMN price TYPE DECIMAL(14,2),
    ALTER COLUMN cost TYPE DECIMAL(14,2);

-- Expand weight column for consistency (optional but included)
ALTER TABLE products
    ALTER COLUMN weight TYPE DECIMAL(10,3);

-- Expand sales_orders table monetary columns
ALTER TABLE sales_orders
    ALTER COLUMN subtotal TYPE DECIMAL(14,2),
    ALTER COLUMN tax_amount TYPE DECIMAL(14,2),
    ALTER COLUMN discount_amount TYPE DECIMAL(14,2),
    ALTER COLUMN shipping_amount TYPE DECIMAL(14,2),
    ALTER COLUMN total_amount TYPE DECIMAL(14,2);

-- Expand sales_order_items table monetary columns
ALTER TABLE sales_order_items
    ALTER COLUMN unit_price TYPE DECIMAL(14,2),
    ALTER COLUMN discount_amount TYPE DECIMAL(14,2),
    ALTER COLUMN total_price TYPE DECIMAL(14,2);

-- DOWN
-- Rollback migration (WARNING: May cause data loss if values exceed DECIMAL(10,2) range)
ALTER TABLE sales_order_items
    ALTER COLUMN unit_price TYPE DECIMAL(10,2),
    ALTER COLUMN discount_amount TYPE DECIMAL(10,2),
    ALTER COLUMN total_price TYPE DECIMAL(10,2);

ALTER TABLE sales_orders
    ALTER COLUMN subtotal TYPE DECIMAL(10,2),
    ALTER COLUMN tax_amount TYPE DECIMAL(10,2),
    ALTER COLUMN discount_amount TYPE DECIMAL(10,2),
    ALTER COLUMN shipping_amount TYPE DECIMAL(10,2),
    ALTER COLUMN total_amount TYPE DECIMAL(10,2);

ALTER TABLE products
    ALTER COLUMN price TYPE DECIMAL(10,2),
    ALTER COLUMN cost TYPE DECIMAL(10,2),
    ALTER COLUMN weight TYPE DECIMAL(8,3);