-- Add customer_type column to customers table (PostgreSQL compatible)
-- Version: 012
-- Description: Add customer_type column and set values based on company field

-- Add customer_type column with default value
ALTER TABLE customers
ADD COLUMN IF NOT EXISTS customer_type VARCHAR(20) NOT NULL DEFAULT 'individual';

-- Update customer_type to 'business' where company is not null
UPDATE customers
SET customer_type = 'business'
WHERE company IS NOT NULL AND company != '';

-- Create index on customer_type for better query performance
CREATE INDEX IF NOT EXISTS idx_customers_customer_type ON customers(customer_type);

-- Add comment for documentation
COMMENT ON COLUMN customers.customer_type IS 'Customer type: individual, business, wholesale, or retail';
