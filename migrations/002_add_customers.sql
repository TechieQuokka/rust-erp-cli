-- Customer management tables migration
-- Version: 002
-- Description: Create customers and customer addresses tables

-- Customers table for customer relationship management
CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(50),
    customer_type VARCHAR(50) DEFAULT 'regular',
    company VARCHAR(255),
    tax_id VARCHAR(100),
    credit_limit DECIMAL(10,2) DEFAULT 0.00,
    current_balance DECIMAL(10,2) DEFAULT 0.00,
    notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Customer addresses table for shipping and billing addresses
CREATE TABLE IF NOT EXISTS customer_addresses (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    address_type VARCHAR(20) NOT NULL DEFAULT 'billing', -- billing, shipping, other
    address_line1 VARCHAR(255) NOT NULL,
    address_line2 VARCHAR(255),
    city VARCHAR(100),
    state_province VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(100) DEFAULT 'US',
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

-- Create indexes for customers table
CREATE INDEX IF NOT EXISTS idx_customers_email ON customers(email);
CREATE INDEX IF NOT EXISTS idx_customers_name ON customers(name);
CREATE INDEX IF NOT EXISTS idx_customers_type ON customers(customer_type);
CREATE INDEX IF NOT EXISTS idx_customers_created_at ON customers(created_at);
CREATE INDEX IF NOT EXISTS idx_customers_company ON customers(company);

-- Create indexes for customer addresses table
CREATE INDEX IF NOT EXISTS idx_customer_addresses_customer_id ON customer_addresses(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_addresses_type ON customer_addresses(address_type);
CREATE INDEX IF NOT EXISTS idx_customer_addresses_default ON customer_addresses(is_default);
CREATE INDEX IF NOT EXISTS idx_customer_addresses_postal ON customer_addresses(postal_code);

-- DOWN
-- Rollback migration
DROP INDEX IF EXISTS idx_customer_addresses_postal;
DROP INDEX IF EXISTS idx_customer_addresses_default;
DROP INDEX IF EXISTS idx_customer_addresses_type;
DROP INDEX IF EXISTS idx_customer_addresses_customer_id;

DROP INDEX IF EXISTS idx_customers_company;
DROP INDEX IF EXISTS idx_customers_created_at;
DROP INDEX IF EXISTS idx_customers_type;
DROP INDEX IF EXISTS idx_customers_name;
DROP INDEX IF EXISTS idx_customers_email;

DROP TABLE IF EXISTS customer_addresses;
DROP TABLE IF EXISTS customers;