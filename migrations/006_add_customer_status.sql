-- Add status column to customers table
-- Version: 006
-- Description: Add customer status tracking

-- Add status column to customers table
ALTER TABLE customers ADD COLUMN IF NOT EXISTS status VARCHAR(20) DEFAULT 'active';

-- Create index for status column
CREATE INDEX IF NOT EXISTS idx_customers_status ON customers(status);

-- Update existing customers to have active status
UPDATE customers SET status = 'active' WHERE status IS NULL;

-- DOWN
-- Rollback migration
DROP INDEX IF EXISTS idx_customers_status;
ALTER TABLE customers DROP COLUMN IF EXISTS status;