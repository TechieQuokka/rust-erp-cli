use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::database::models::customer::*;
use crate::utils::error::{ErpError, ErpResult};

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
    async fn create_customer(&self, _customer: &Customer) -> ErpResult<()> {
        // TODO: Implement with proper SQLx queries when database is set up
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn create_customer_address(&self, _address: &CustomerAddress) -> ErpResult<()> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn get_customer_by_id(&self, _id: Uuid) -> ErpResult<Option<Customer>> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn get_customer_by_code(&self, _customer_code: &str) -> ErpResult<Option<Customer>> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn get_customer_by_email(&self, _email: &str) -> ErpResult<Option<Customer>> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn get_customer_addresses(&self, _customer_id: Uuid) -> ErpResult<Vec<CustomerAddress>> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn list_customers(
        &self,
        _filter: &CustomerFilter,
        _page: u32,
        _per_page: u32,
    ) -> ErpResult<CustomerListResponse> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn update_customer(&self, _id: Uuid, _customer: &Customer) -> ErpResult<()> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn update_customer_balance(
        &self,
        _id: Uuid,
        _new_balance: rust_decimal::Decimal,
    ) -> ErpResult<()> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn delete_customer(&self, _id: Uuid) -> ErpResult<()> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
    }

    async fn search_customers(&self, _query: &str, _limit: u32) -> ErpResult<Vec<Customer>> {
        Err(ErpError::internal(
            "Database operations not yet implemented - use MockCustomerRepository for testing",
        ))
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
        let customer_addresses = addresses
            .entry(address.customer_id)
            .or_insert_with(Vec::new);

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
                        .map_or(false, |name| name.to_lowercase().contains(&search_lower))
            });
        }
        if let Some(has_outstanding) = filter.has_outstanding_balance {
            if has_outstanding {
                filtered_customers.retain(|c| c.has_outstanding_balance());
            } else {
                filtered_customers.retain(|c| !c.has_outstanding_balance());
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
        if customers.contains_key(&id) {
            customers.insert(id, customer.clone());
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
                        .map_or(false, |name| name.to_lowercase().contains(&query_lower))
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
