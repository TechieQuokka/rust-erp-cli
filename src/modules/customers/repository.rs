use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::database::models::customer::*;
use crate::utils::error::{ErpError, ErpResult};

// Helper function to build ORDER BY clause
fn build_sort_clause(sort_by: &str, sort_order: &str) -> (String, String) {
    let valid_sort_fields = ["name", "email", "created_at"];

    let sort_field = if valid_sort_fields.contains(&sort_by) {
        sort_by
    } else {
        "name"
    };

    let sort_order = if sort_order == "asc" || sort_order == "desc" {
        sort_order.to_uppercase()
    } else {
        "ASC".to_string()
    };

    (sort_field.to_string(), sort_order)
}

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create_customer(&self, customer: &Customer) -> ErpResult<()>;
    async fn create_customer_address(&self, address: &CustomerAddress) -> ErpResult<()>;
    async fn get_customer_by_id(&self, id: Uuid) -> ErpResult<Option<Customer>>;
    async fn get_customer_by_code(&self, customer_code: &str) -> ErpResult<Option<Customer>>;
    async fn get_customer_by_email(&self, email: &str) -> ErpResult<Option<Customer>>;
    async fn get_customer_addresses(&self, customer_id: Uuid) -> ErpResult<Vec<CustomerAddress>>;
    async fn list_customers(
        &self,
        filter: &CustomerFilter,
        page: u32,
        per_page: u32,
        sort_by: &str,
        sort_order: &str,
    ) -> ErpResult<CustomerListResponse>;
    async fn update_customer(&self, id: Uuid, customer: &Customer) -> ErpResult<()>;
    async fn update_customer_balance(
        &self,
        id: Uuid,
        new_balance: rust_decimal::Decimal,
    ) -> ErpResult<()>;
    async fn delete_customer(&self, id: Uuid) -> ErpResult<()>;
    async fn search_customers(&self, query: &str, limit: u32) -> ErpResult<Vec<Customer>>;
    async fn get_customers_with_outstanding_balance(&self) -> ErpResult<Vec<Customer>>;
    async fn get_customers_by_type(&self, customer_type: &CustomerType)
        -> ErpResult<Vec<Customer>>;
    async fn count_customers(&self) -> ErpResult<i64>;
    async fn count_customers_by_status(&self, status: &CustomerStatus) -> ErpResult<i64>;
}

pub struct PostgresCustomerRepository {
    pool: Arc<Pool<Postgres>>,
}

impl PostgresCustomerRepository {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerRepository for PostgresCustomerRepository {
    async fn create_customer(&self, customer: &Customer) -> ErpResult<()> {
        let full_name = format!("{} {}", customer.first_name, customer.last_name)
            .trim()
            .to_string();
        let customer_type_str = match customer.customer_type {
            CustomerType::Individual => "individual",
            CustomerType::Business => "business",
            CustomerType::Wholesale => "wholesale",
            CustomerType::Retail => "retail",
        };

        sqlx::query!(
            r#"
            INSERT INTO customers (id, name, email, phone, company, tax_id, customer_type, credit_limit, current_balance, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            customer.id,
            full_name,
            customer.email,
            customer.phone,
            customer.company_name,
            customer.tax_id,
            customer_type_str,
            customer.credit_limit,
            customer.current_balance,
            customer.notes
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to create customer: {}", e)))?;

        Ok(())
    }

    async fn create_customer_address(&self, address: &CustomerAddress) -> ErpResult<()> {
        let address_type_str = match address.address_type {
            AddressType::Billing => "billing",
            AddressType::Shipping => "shipping",
            AddressType::Both => "both",
        };

        sqlx::query!(
            r#"
            INSERT INTO customer_addresses (id, customer_id, address_type, address_line1, city, state_province, postal_code, country, is_default)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            address.id,
            address.customer_id,
            address_type_str,
            address.street_address,
            address.city,
            address.state_province,
            address.postal_code,
            address.country,
            address.is_default
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to create customer address: {}", e)))?;

        Ok(())
    }

    async fn get_customer_by_id(&self, id: Uuid) -> ErpResult<Option<Customer>> {
        let row = sqlx::query!(
            "SELECT id, name, email, phone, company, tax_id, customer_type, credit_limit, current_balance, notes, status, created_at, updated_at
             FROM customers WHERE id = $1",
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to get customer by id: {}", e)))?;

        if let Some(row) = row {
            let customer_type = match row.customer_type.as_str() {
                "business" => CustomerType::Business,
                "wholesale" => CustomerType::Wholesale,
                "retail" => CustomerType::Retail,
                _ => CustomerType::Individual,
            };

            let status = match row.status.as_deref() {
                Some("active") => CustomerStatus::Active,
                Some("inactive") => CustomerStatus::Inactive,
                Some("suspended") => CustomerStatus::Suspended,
                Some("blacklisted") => CustomerStatus::Blacklisted,
                _ => CustomerStatus::Active,
            };

            let name_parts: Vec<&str> = row.name.splitn(2, ' ').collect();
            let first_name = name_parts.first().unwrap_or(&"").to_string();
            let last_name = name_parts.get(1).unwrap_or(&"").to_string();

            let customer = Customer {
                id,
                customer_code: format!("CUST-{}", &row.id.to_string()[..8]),
                first_name,
                last_name,
                company_name: row.company,
                email: row.email.unwrap_or_default(),
                phone: row.phone,
                customer_type,
                status,
                credit_limit: row.credit_limit.unwrap_or_default(),
                current_balance: row.current_balance.unwrap_or_default(),
                tax_id: row.tax_id,
                notes: row.notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            Ok(Some(customer))
        } else {
            Ok(None)
        }
    }

    async fn get_customer_by_code(&self, customer_code: &str) -> ErpResult<Option<Customer>> {
        // Extract UUID from customer code format "CUST-{first 8 chars of UUID}"
        if !customer_code.starts_with("CUST-") || customer_code.len() < 13 {
            return Ok(None);
        }

        let partial_id = &customer_code[5..13]; // Extract the 8-char UUID prefix

        let row = sqlx::query!(
            "SELECT id, name, email, phone, company, tax_id, customer_type, credit_limit, current_balance, notes, status, created_at, updated_at
             FROM customers WHERE SUBSTRING(id::text, 1, 8) = $1",
            partial_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to get customer by code: {}", e)))?;

        if let Some(row) = row {
            let customer_type = match row.customer_type.as_str() {
                "business" => CustomerType::Business,
                "wholesale" => CustomerType::Wholesale,
                "retail" => CustomerType::Retail,
                _ => CustomerType::Individual,
            };

            let status = match row.status.as_deref() {
                Some("active") => CustomerStatus::Active,
                Some("inactive") => CustomerStatus::Inactive,
                Some("suspended") => CustomerStatus::Suspended,
                Some("blacklisted") => CustomerStatus::Blacklisted,
                _ => CustomerStatus::Active,
            };

            let name_parts: Vec<&str> = row.name.splitn(2, ' ').collect();
            let first_name = name_parts.first().unwrap_or(&"").to_string();
            let last_name = name_parts.get(1).unwrap_or(&"").to_string();

            let customer = Customer {
                id: row.id,
                customer_code: format!("CUST-{}", &row.id.to_string()[..8]),
                first_name,
                last_name,
                company_name: row.company,
                email: row.email.unwrap_or_default(),
                phone: row.phone,
                customer_type,
                status,
                credit_limit: row.credit_limit.unwrap_or_default(),
                current_balance: row.current_balance.unwrap_or_default(),
                tax_id: row.tax_id,
                notes: row.notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            Ok(Some(customer))
        } else {
            Ok(None)
        }
    }

    async fn get_customer_by_email(&self, email: &str) -> ErpResult<Option<Customer>> {
        let row = sqlx::query!(
            "SELECT id, name, email, phone, company, tax_id, customer_type, credit_limit, current_balance, notes, status, created_at, updated_at
             FROM customers WHERE email = $1",
            email
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to get customer by email: {}", e)))?;

        if let Some(row) = row {
            let customer_type = match row.customer_type.as_str() {
                "business" => CustomerType::Business,
                "wholesale" => CustomerType::Wholesale,
                "retail" => CustomerType::Retail,
                _ => CustomerType::Individual,
            };

            let status = match row.status.as_deref() {
                Some("active") => CustomerStatus::Active,
                Some("inactive") => CustomerStatus::Inactive,
                Some("suspended") => CustomerStatus::Suspended,
                Some("blacklisted") => CustomerStatus::Blacklisted,
                _ => CustomerStatus::Active,
            };

            let name_parts: Vec<&str> = row.name.splitn(2, ' ').collect();
            let first_name = name_parts.first().unwrap_or(&"").to_string();
            let last_name = name_parts.get(1).unwrap_or(&"").to_string();

            let customer = Customer {
                id: row.id,
                customer_code: format!("CUST-{}", &row.id.to_string()[..8]),
                first_name,
                last_name,
                company_name: row.company,
                email: row.email.unwrap_or_default(),
                phone: row.phone,
                customer_type,
                status,
                credit_limit: row.credit_limit.unwrap_or_default(),
                current_balance: row.current_balance.unwrap_or_default(),
                tax_id: row.tax_id,
                notes: row.notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            Ok(Some(customer))
        } else {
            Ok(None)
        }
    }

    async fn get_customer_addresses(&self, customer_id: Uuid) -> ErpResult<Vec<CustomerAddress>> {
        let rows = sqlx::query!(
            "SELECT id, customer_id, address_type, address_line1, address_line2, city, state_province, postal_code, country, is_default, created_at
             FROM customer_addresses WHERE customer_id = $1",
            customer_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to get customer addresses: {}", e)))?;

        let mut addresses = Vec::new();
        for row in rows {
            let address_type = match row.address_type.as_str() {
                "billing" => AddressType::Billing,
                "shipping" => AddressType::Shipping,
                "both" => AddressType::Both,
                _ => AddressType::Billing,
            };

            let address = CustomerAddress {
                id: row.id,
                customer_id,
                address_type,
                street_address: row.address_line1,
                city: row.city.unwrap_or_default(),
                state_province: row.state_province.unwrap_or_default(),
                postal_code: row.postal_code.unwrap_or_default(),
                country: row.country.unwrap_or_default(),
                is_default: row.is_default,
                created_at: row.created_at,
                updated_at: row.created_at, // Use created_at for updated_at since there's no updated_at in addresses table
            };

            addresses.push(address);
        }

        Ok(addresses)
    }

    async fn list_customers(
        &self,
        filter: &CustomerFilter,
        page: u32,
        per_page: u32,
        sort_by: &str,
        sort_order: &str,
    ) -> ErpResult<CustomerListResponse> {
        let offset = (page - 1) * per_page;
        let (sort_field, sort_order) = build_sort_clause(sort_by, sort_order);

        // Build dynamic WHERE clause based on filters
        let mut where_conditions = Vec::new();
        let mut bind_values = Vec::new();
        let mut param_count = 0;

        if let Some(search) = &filter.search {
            param_count += 1;
            let search_pattern = format!("%{}%", search);
            where_conditions.push(format!(
                "(name ILIKE ${} OR email ILIKE ${} OR company ILIKE ${})",
                param_count, param_count, param_count
            ));
            bind_values.push(search_pattern);
        }

        if let Some(customer_type) = &filter.customer_type {
            param_count += 1;
            let type_str = match customer_type {
                CustomerType::Individual => "individual",
                CustomerType::Business => "business",
                CustomerType::Wholesale => "wholesale",
                CustomerType::Retail => "retail",
            };
            where_conditions.push(format!("customer_type = ${}", param_count));
            bind_values.push(type_str.to_string());
        }

        if let Some(status) = &filter.status {
            param_count += 1;
            let status_str = match status {
                CustomerStatus::Active => "active",
                CustomerStatus::Inactive => "inactive",
                CustomerStatus::Suspended => "suspended",
                CustomerStatus::Blacklisted => "blacklisted",
            };
            where_conditions.push(format!("status = ${}", param_count));
            bind_values.push(status_str.to_string());
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Get total count with filters
        let count_query = format!("SELECT COUNT(*) FROM customers {}", where_clause);
        let mut count_query_builder = sqlx::query_scalar(&count_query);
        for value in &bind_values {
            count_query_builder = count_query_builder.bind(value);
        }
        let total: i64 = count_query_builder
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to count customers: {}", e)))?;

        // Get customers with pagination, filtering, and dynamic sorting
        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        let query = format!(
            "SELECT id, name, email, phone, company, tax_id, customer_type, credit_limit, current_balance, notes, created_at, updated_at
             FROM customers {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
            where_clause, sort_field, sort_order, limit_param, offset_param
        );

        let mut query_builder = sqlx::query_as::<
            _,
            (
                Uuid,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<rust_decimal::Decimal>,
                Option<rust_decimal::Decimal>,
                Option<String>,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(&query);

        for value in &bind_values {
            query_builder = query_builder.bind(value);
        }

        let rows = query_builder
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| ErpError::database(format!("Failed to fetch customers: {}", e)))?;

        let mut customers = Vec::new();
        for row in rows {
            let (
                id,
                name,
                email,
                phone,
                company,
                tax_id,
                customer_type_str,
                credit_limit,
                current_balance,
                notes,
                created_at,
                updated_at,
            ) = row;

            // Parse customer type from database
            let customer_type = match customer_type_str.as_deref() {
                Some("business") => CustomerType::Business,
                Some("wholesale") => CustomerType::Wholesale,
                Some("retail") => CustomerType::Retail,
                _ => CustomerType::Individual,
            };

            // Split name into first and last name (simplified)
            let name_parts: Vec<&str> = name.splitn(2, ' ').collect();
            let first_name = name_parts.first().unwrap_or(&"").to_string();
            let last_name = name_parts.get(1).unwrap_or(&"").to_string();

            let customer = Customer {
                id,
                customer_code: format!("CUST-{}", &id.to_string()[..8]), // Generate code from ID
                first_name,
                last_name,
                company_name: company,
                email: email.unwrap_or_default(),
                phone,
                customer_type,
                status: CustomerStatus::Active, // Default status
                credit_limit: credit_limit.unwrap_or_default(),
                current_balance: current_balance.unwrap_or_default(),
                tax_id,
                notes,
                created_at,
                updated_at,
            };

            // Load addresses for each customer
            let addresses = self.get_customer_addresses(customer.id).await?;
            customers.push(customer.to_response(addresses));
        }

        Ok(CustomerListResponse {
            customers,
            total,
            page,
            per_page,
        })
    }

    async fn update_customer(&self, id: Uuid, customer: &Customer) -> ErpResult<()> {
        let full_name = format!("{} {}", customer.first_name, customer.last_name)
            .trim()
            .to_string();
        let customer_type_str = match customer.customer_type {
            CustomerType::Individual => "individual",
            CustomerType::Business => "business",
            CustomerType::Wholesale => "wholesale",
            CustomerType::Retail => "retail",
        };
        let status_str = match customer.status {
            CustomerStatus::Active => "active",
            CustomerStatus::Inactive => "inactive",
            CustomerStatus::Suspended => "suspended",
            CustomerStatus::Blacklisted => "blacklisted",
        };

        sqlx::query!(
            r#"
            UPDATE customers
            SET name = $2, email = $3, phone = $4, company = $5,
                tax_id = $6, customer_type = $7, credit_limit = $8, current_balance = $9, notes = $10,
                status = $11, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
            id,
            full_name,
            customer.email,
            customer.phone,
            customer.company_name,
            customer.tax_id,
            customer_type_str,
            customer.credit_limit,
            customer.current_balance,
            customer.notes,
            status_str
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to update customer: {}", e)))?;

        Ok(())
    }

    async fn update_customer_balance(
        &self,
        id: Uuid,
        new_balance: rust_decimal::Decimal,
    ) -> ErpResult<()> {
        sqlx::query!(
            r#"
            UPDATE customers
            SET current_balance = $2, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
            id,
            new_balance
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to update customer balance: {}", e)))?;

        Ok(())
    }

    async fn delete_customer(&self, id: Uuid) -> ErpResult<()> {
        // Start a transaction to ensure atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ErpError::database(format!("Failed to start transaction: {}", e)))?;

        // First, delete all customer addresses
        sqlx::query!("DELETE FROM customer_addresses WHERE customer_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                ErpError::database(format!("Failed to delete customer addresses: {}", e))
            })?;

        // Then delete the customer
        let result = sqlx::query!("DELETE FROM customers WHERE id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ErpError::database(format!("Failed to delete customer: {}", e)))?;

        // Check if customer was actually deleted
        if result.rows_affected() == 0 {
            tx.rollback().await.map_err(|e| {
                ErpError::database(format!("Failed to rollback transaction: {}", e))
            })?;
            return Err(ErpError::not_found_simple("Customer not found"));
        }

        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| ErpError::database(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    async fn search_customers(&self, query: &str, limit: u32) -> ErpResult<Vec<Customer>> {
        let search_pattern = format!("%{}%", query);
        let rows = sqlx::query!(
            "SELECT id, name, email, phone, company, tax_id, customer_type, credit_limit, current_balance, notes, created_at, updated_at
             FROM customers
             WHERE name ILIKE $1 OR email ILIKE $1 OR company ILIKE $1
             ORDER BY created_at DESC
             LIMIT $2",
            search_pattern,
            limit as i64
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| ErpError::database(format!("Failed to search customers: {}", e)))?;

        let mut customers = Vec::new();
        for row in rows {
            let customer_type = match row.customer_type.as_str() {
                "business" => CustomerType::Business,
                "wholesale" => CustomerType::Wholesale,
                "retail" => CustomerType::Retail,
                _ => CustomerType::Individual,
            };

            let name_parts: Vec<&str> = row.name.splitn(2, ' ').collect();
            let first_name = name_parts.first().unwrap_or(&"").to_string();
            let last_name = name_parts.get(1).unwrap_or(&"").to_string();

            let customer = Customer {
                id: row.id,
                customer_code: format!("CUST-{}", &row.id.to_string()[..8]),
                first_name,
                last_name,
                company_name: row.company,
                email: row.email.unwrap_or_default(),
                phone: row.phone,
                customer_type,
                status: CustomerStatus::Active,
                credit_limit: row.credit_limit.unwrap_or_default(),
                current_balance: row.current_balance.unwrap_or_default(),
                tax_id: row.tax_id,
                notes: row.notes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };

            customers.push(customer);
        }

        Ok(customers)
    }

    async fn get_customers_with_outstanding_balance(&self) -> ErpResult<Vec<Customer>> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn get_customers_by_type(
        &self,
        _customer_type: &CustomerType,
    ) -> ErpResult<Vec<Customer>> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn count_customers(&self) -> ErpResult<i64> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn count_customers_by_status(&self, _status: &CustomerStatus) -> ErpResult<i64> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }
}

// Mock repository for testing
pub struct MockCustomerRepository {
    customers: std::sync::Mutex<std::collections::HashMap<Uuid, Customer>>,
    addresses: std::sync::Mutex<std::collections::HashMap<Uuid, Vec<CustomerAddress>>>,
}

impl MockCustomerRepository {
    pub fn new() -> Self {
        Self {
            customers: std::sync::Mutex::new(std::collections::HashMap::new()),
            addresses: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn with_customers(customers: Vec<Customer>) -> Self {
        let repo = Self::new();
        {
            let mut store = repo.customers.lock().unwrap();
            for customer in customers {
                store.insert(customer.id, customer);
            }
        }
        repo
    }
}

impl Default for MockCustomerRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CustomerRepository for MockCustomerRepository {
    async fn create_customer(&self, customer: &Customer) -> ErpResult<()> {
        let mut customers = self.customers.lock().unwrap();

        // Check for duplicate email
        for existing in customers.values() {
            if existing.email == customer.email {
                return Err(ErpError::validation_simple("Email already exists"));
            }
            if existing.customer_code == customer.customer_code {
                return Err(ErpError::validation_simple("Customer code already exists"));
            }
        }

        customers.insert(customer.id, customer.clone());
        Ok(())
    }

    async fn create_customer_address(&self, address: &CustomerAddress) -> ErpResult<()> {
        let mut addresses = self.addresses.lock().unwrap();
        let customer_addresses = addresses.entry(address.customer_id).or_default();

        // If this is a default address, unset other defaults
        if address.is_default {
            for addr in customer_addresses.iter_mut() {
                addr.is_default = false;
            }
        }

        customer_addresses.push(address.clone());
        Ok(())
    }

    async fn get_customer_by_id(&self, id: Uuid) -> ErpResult<Option<Customer>> {
        let customers = self.customers.lock().unwrap();
        Ok(customers.get(&id).cloned())
    }

    async fn get_customer_by_code(&self, customer_code: &str) -> ErpResult<Option<Customer>> {
        let customers = self.customers.lock().unwrap();
        Ok(customers
            .values()
            .find(|c| c.customer_code == customer_code)
            .cloned())
    }

    async fn get_customer_by_email(&self, email: &str) -> ErpResult<Option<Customer>> {
        let customers = self.customers.lock().unwrap();
        Ok(customers.values().find(|c| c.email == email).cloned())
    }

    async fn get_customer_addresses(&self, customer_id: Uuid) -> ErpResult<Vec<CustomerAddress>> {
        let addresses = self.addresses.lock().unwrap();
        Ok(addresses.get(&customer_id).cloned().unwrap_or_default())
    }

    async fn list_customers(
        &self,
        filter: &CustomerFilter,
        page: u32,
        per_page: u32,
        sort_by: &str,
        sort_order: &str,
    ) -> ErpResult<CustomerListResponse> {
        let customers = self.customers.lock().unwrap();
        let mut filtered_customers: Vec<_> = customers.values().collect();

        // Apply filters
        if let Some(status) = &filter.status {
            filtered_customers.retain(|c| &c.status == status);
        }
        if let Some(customer_type) = &filter.customer_type {
            filtered_customers.retain(|c| &c.customer_type == customer_type);
        }
        if let Some(search) = &filter.search {
            let search_lower = search.to_lowercase();
            filtered_customers.retain(|c| {
                c.first_name.to_lowercase().contains(&search_lower)
                    || c.last_name.to_lowercase().contains(&search_lower)
                    || c.email.to_lowercase().contains(&search_lower)
                    || c.customer_code.to_lowercase().contains(&search_lower)
                    || c.company_name
                        .as_ref()
                        .is_some_and(|name| name.to_lowercase().contains(&search_lower))
            });
        }
        if let Some(has_outstanding) = filter.has_outstanding_balance {
            if has_outstanding {
                filtered_customers.retain(|c| c.has_outstanding_balance());
            } else {
                filtered_customers.retain(|c| !c.has_outstanding_balance());
            }
        }

        // Apply sorting
        let (sort_field, sort_order) = build_sort_clause(sort_by, sort_order);
        match sort_field.as_str() {
            "name" => {
                if sort_order == "ASC" {
                    filtered_customers.sort_by(|a, b| {
                        let a_name = format!("{} {}", a.first_name, a.last_name);
                        let b_name = format!("{} {}", b.first_name, b.last_name);
                        a_name.cmp(&b_name)
                    });
                } else {
                    filtered_customers.sort_by(|a, b| {
                        let a_name = format!("{} {}", a.first_name, a.last_name);
                        let b_name = format!("{} {}", b.first_name, b.last_name);
                        b_name.cmp(&a_name)
                    });
                }
            }
            "email" => {
                if sort_order == "ASC" {
                    filtered_customers.sort_by(|a, b| a.email.cmp(&b.email));
                } else {
                    filtered_customers.sort_by(|a, b| b.email.cmp(&a.email));
                }
            }
            "created_at" => {
                if sort_order == "ASC" {
                    filtered_customers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                } else {
                    filtered_customers.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                }
            }
            _ => {
                // Default to name sorting
                filtered_customers.sort_by(|a, b| {
                    let a_name = format!("{} {}", a.first_name, a.last_name);
                    let b_name = format!("{} {}", b.first_name, b.last_name);
                    a_name.cmp(&b_name)
                });
            }
        }

        let total = filtered_customers.len() as i64;

        // Apply pagination
        let start = ((page.saturating_sub(1)) * per_page) as usize;
        let end = std::cmp::min(start + per_page as usize, filtered_customers.len());

        let paginated_customers = if start >= filtered_customers.len() {
            Vec::new()
        } else {
            filtered_customers[start..end].to_vec()
        };

        // Convert to response format
        let mut customer_responses = Vec::new();
        let addresses = self.addresses.lock().unwrap();
        for customer in paginated_customers {
            let customer_addresses = addresses.get(&customer.id).cloned().unwrap_or_default();
            customer_responses.push(customer.to_response(customer_addresses));
        }

        Ok(CustomerListResponse {
            customers: customer_responses,
            total,
            page,
            per_page,
        })
    }

    async fn update_customer(&self, id: Uuid, customer: &Customer) -> ErpResult<()> {
        let mut customers = self.customers.lock().unwrap();
        if let std::collections::hash_map::Entry::Occupied(mut e) = customers.entry(id) {
            e.insert(customer.clone());
            Ok(())
        } else {
            Err(ErpError::not_found_simple("Customer not found"))
        }
    }

    async fn update_customer_balance(
        &self,
        id: Uuid,
        new_balance: rust_decimal::Decimal,
    ) -> ErpResult<()> {
        let mut customers = self.customers.lock().unwrap();
        if let Some(customer) = customers.get_mut(&id) {
            customer.current_balance = new_balance;
            Ok(())
        } else {
            Err(ErpError::not_found_simple("Customer not found"))
        }
    }

    async fn delete_customer(&self, id: Uuid) -> ErpResult<()> {
        let mut customers = self.customers.lock().unwrap();
        let mut addresses = self.addresses.lock().unwrap();

        if customers.remove(&id).is_some() {
            addresses.remove(&id);
            Ok(())
        } else {
            Err(ErpError::not_found_simple("Customer not found"))
        }
    }

    async fn search_customers(&self, query: &str, limit: u32) -> ErpResult<Vec<Customer>> {
        let customers = self.customers.lock().unwrap();
        let query_lower = query.to_lowercase();

        let mut matching_customers: Vec<_> = customers
            .values()
            .filter(|c| {
                c.first_name.to_lowercase().contains(&query_lower)
                    || c.last_name.to_lowercase().contains(&query_lower)
                    || c.email.to_lowercase().contains(&query_lower)
                    || c.customer_code.to_lowercase().contains(&query_lower)
                    || c.company_name
                        .as_ref()
                        .is_some_and(|name| name.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        matching_customers.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        matching_customers.truncate(limit as usize);

        Ok(matching_customers)
    }

    async fn get_customers_with_outstanding_balance(&self) -> ErpResult<Vec<Customer>> {
        let customers = self.customers.lock().unwrap();
        let mut outstanding_customers: Vec<_> = customers
            .values()
            .filter(|c| c.has_outstanding_balance())
            .cloned()
            .collect();

        outstanding_customers.sort_by(|a, b| b.current_balance.cmp(&a.current_balance));
        Ok(outstanding_customers)
    }

    async fn get_customers_by_type(
        &self,
        customer_type: &CustomerType,
    ) -> ErpResult<Vec<Customer>> {
        let customers = self.customers.lock().unwrap();
        let mut type_customers: Vec<_> = customers
            .values()
            .filter(|c| &c.customer_type == customer_type)
            .cloned()
            .collect();

        type_customers.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(type_customers)
    }

    async fn count_customers(&self) -> ErpResult<i64> {
        let customers = self.customers.lock().unwrap();
        Ok(customers.len() as i64)
    }

    async fn count_customers_by_status(&self, status: &CustomerStatus) -> ErpResult<i64> {
        let customers = self.customers.lock().unwrap();
        Ok(customers.values().filter(|c| &c.status == status).count() as i64)
    }
}
