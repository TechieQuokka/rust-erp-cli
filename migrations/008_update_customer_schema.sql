-- Update customer table schema to match Rust model
-- Version: 008
-- Description: Update customers table to use first_name/last_name and add required fields

-- First, add new columns
ALTER TABLE customers ADD COLUMN IF NOT EXISTS first_name VARCHAR(255);
ALTER TABLE customers ADD COLUMN IF NOT EXISTS last_name VARCHAR(255);
ALTER TABLE customers ADD COLUMN IF NOT EXISTS customer_code VARCHAR(100);
ALTER TABLE customers ADD COLUMN IF NOT EXISTS customer_type VARCHAR(50) DEFAULT 'individual';
ALTER TABLE customers ADD COLUMN IF NOT EXISTS company_name VARCHAR(255);

-- Update existing data: split name into first_name and last_name
UPDATE customers
SET
    first_name = CASE
        WHEN name LIKE '% %' THEN SUBSTR(name, 1, INSTR(name, ' ') - 1)
        ELSE name
    END,
    last_name = CASE
        WHEN name LIKE '% %' THEN SUBSTR(name, INSTR(name, ' ') + 1)
        ELSE ''
    END,
    customer_code = 'CUST-' || SUBSTR(UPPER(name), 1, 2) || '-' || (ABS(RANDOM()) % 100000),
    customer_type = 'individual'
WHERE first_name IS NULL;

-- Make first_name and last_name NOT NULL after data migration
-- Note: In SQLite, we can't directly add NOT NULL constraint to existing columns
-- We need to recreate the table if we want strict NOT NULL constraints

-- Create indexes for new columns
CREATE INDEX IF NOT EXISTS idx_customers_first_name ON customers(first_name);
CREATE INDEX IF NOT EXISTS idx_customers_last_name ON customers(last_name);
CREATE INDEX IF NOT EXISTS idx_customers_customer_code ON customers(customer_code);
CREATE INDEX IF NOT EXISTS idx_customers_customer_type ON customers(customer_type);

-- We can drop the name column after confirming migration works
-- ALTER TABLE customers DROP COLUMN name;

-- DOWN
-- Rollback migration
DROP INDEX IF EXISTS idx_customers_customer_type;
DROP INDEX IF EXISTS idx_customers_customer_code;
DROP INDEX IF EXISTS idx_customers_last_name;
DROP INDEX IF EXISTS idx_customers_first_name;

ALTER TABLE customers DROP COLUMN IF EXISTS company_name;
ALTER TABLE customers DROP COLUMN IF EXISTS customer_type;
ALTER TABLE customers DROP COLUMN IF EXISTS customer_code;
ALTER TABLE customers DROP COLUMN IF EXISTS last_name;
ALTER TABLE customers DROP COLUMN IF EXISTS first_name;