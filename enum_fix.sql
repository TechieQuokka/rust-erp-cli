-- Fix for product_status enum type
CREATE TYPE product_status AS ENUM ('active', 'inactive', 'discontinued', 'out_of_stock');