use rust_decimal::Decimal;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::modules::customers::models::*;
use crate::modules::customers::repository::CustomerRepository;
use crate::utils::error::{ErpError, ErpResult};
use crate::utils::validation::{validate_email, validate_phone};

pub struct CustomerService {
    repository: Arc<dyn CustomerRepository>,
}

impl CustomerService {
    pub fn new(repository: Arc<dyn CustomerRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_customer(
        &self,
        request: CreateCustomerRequest,
    ) -> ErpResult<CustomerResponse> {
        info!("Creating new customer: {}", request.email);

        // Validate request
        self.validate_create_request(&request).await?;

        // Check if email already exists
        if (self
            .repository
            .get_customer_by_email(&request.email)
            .await?)
            .is_some()
        {
            return Err(ErpError::validation_simple("Email already exists"));
        }

        // Create customer
        let customer = Customer::new(request.clone());

        // Store customer
        self.repository.create_customer(&customer).await?;

        // Create addresses if provided
        let mut customer_addresses = Vec::new();
        for address_request in request.addresses {
            let address = CustomerAddress::new(customer.id, address_request);
            self.repository.create_customer_address(&address).await?;
            customer_addresses.push(address);
        }

        info!(
            "Customer created successfully: {} ({})",
            customer.customer_code, customer.id
        );

        Ok(customer.to_response(customer_addresses))
    }

    pub async fn get_customer_by_id(&self, id: Uuid) -> ErpResult<CustomerResponse> {
        let customer = self
            .repository
            .get_customer_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        let addresses = self.repository.get_customer_addresses(customer.id).await?;

        Ok(customer.to_response(addresses))
    }

    pub async fn get_customer_by_code(&self, customer_code: &str) -> ErpResult<CustomerResponse> {
        let customer = self
            .repository
            .get_customer_by_code(customer_code)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        let addresses = self.repository.get_customer_addresses(customer.id).await?;

        Ok(customer.to_response(addresses))
    }

    pub async fn get_customer_by_email(&self, email: &str) -> ErpResult<CustomerResponse> {
        let customer = self
            .repository
            .get_customer_by_email(email)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        let addresses = self.repository.get_customer_addresses(customer.id).await?;

        Ok(customer.to_response(addresses))
    }

    pub async fn list_customers(
        &self,
        filter: CustomerFilter,
        page: u32,
        per_page: u32,
        sort_by: &str,
        sort_order: &str,
    ) -> ErpResult<CustomerListResponse> {
        // Validate pagination parameters
        if page == 0 {
            return Err(ErpError::validation_simple("Page must be greater than 0"));
        }
        if per_page == 0 || per_page > 100 {
            return Err(ErpError::validation_simple(
                "Per page must be between 1 and 100",
            ));
        }

        self.repository
            .list_customers(&filter, page, per_page, sort_by, sort_order)
            .await
    }

    pub async fn update_customer(
        &self,
        id: Uuid,
        request: UpdateCustomerRequest,
    ) -> ErpResult<CustomerResponse> {
        info!("Updating customer: {}", id);

        // Get existing customer
        let mut customer = self
            .repository
            .get_customer_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        // Validate update request
        self.validate_update_request(&request, &customer).await?;

        // Check for email conflicts if email is being updated
        if let Some(new_email) = &request.email {
            if new_email != &customer.email
                && (self.repository.get_customer_by_email(new_email).await?).is_some()
            {
                return Err(ErpError::validation_simple("Email already exists"));
            }
        }

        // Apply updates
        customer.update(request);

        // Save updated customer
        self.repository.update_customer(id, &customer).await?;

        let addresses = self.repository.get_customer_addresses(customer.id).await?;

        info!(
            "Customer updated successfully: {} ({})",
            customer.customer_code, customer.id
        );

        Ok(customer.to_response(addresses))
    }

    pub async fn delete_customer(&self, id: Uuid) -> ErpResult<()> {
        info!("Deleting customer: {}", id);

        // Check if customer exists
        let customer = self
            .repository
            .get_customer_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        // Check if customer has outstanding balance
        if customer.has_outstanding_balance() {
            warn!(
                "Attempted to delete customer {} with outstanding balance: {}",
                customer.customer_code, customer.current_balance
            );
            return Err(ErpError::business_rule(
                "Cannot delete customer with outstanding balance",
            ));
        }

        // Check if customer can be safely deleted (no recent orders, etc.)
        // This would typically check for foreign key constraints
        // For now, we'll allow deletion

        self.repository.delete_customer(id).await?;

        info!(
            "Customer deleted successfully: {} ({})",
            customer.customer_code, id
        );
        Ok(())
    }

    pub async fn search_customers(
        &self,
        query: &str,
        limit: u32,
    ) -> ErpResult<Vec<CustomerResponse>> {
        if query.is_empty() {
            return Err(ErpError::validation_simple("Search query cannot be empty"));
        }

        let limit = std::cmp::min(limit, 50); // Cap at 50 results
        let customers = self.repository.search_customers(query, limit).await?;

        let mut results = Vec::new();
        for customer in customers {
            let addresses = self.repository.get_customer_addresses(customer.id).await?;
            results.push(customer.to_response(addresses));
        }

        Ok(results)
    }

    pub async fn add_customer_address(
        &self,
        customer_id: Uuid,
        request: CreateAddressRequest,
    ) -> ErpResult<CustomerAddress> {
        // Validate customer exists
        let _customer = self
            .repository
            .get_customer_by_id(customer_id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        // Validate address request
        self.validate_address_request(&request)?;

        let address = CustomerAddress::new(customer_id, request);
        self.repository.create_customer_address(&address).await?;

        info!(
            "Address added for customer: {} ({})",
            customer_id, address.id
        );

        Ok(address)
    }

    pub async fn update_customer_balance(
        &self,
        id: Uuid,
        amount: Decimal,
        operation: BalanceOperation,
    ) -> ErpResult<CustomerResponse> {
        let mut customer = self
            .repository
            .get_customer_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        let new_balance = match operation {
            BalanceOperation::Add => customer.current_balance + amount,
            BalanceOperation::Subtract => customer.current_balance - amount,
            BalanceOperation::Set => amount,
        };

        if new_balance < Decimal::ZERO {
            return Err(ErpError::business_rule(
                "Customer balance cannot be negative",
            ));
        }

        customer.current_balance = new_balance;
        self.repository
            .update_customer_balance(id, new_balance)
            .await?;

        let addresses = self.repository.get_customer_addresses(customer.id).await?;

        info!(
            "Customer balance updated: {} -> {} for customer {}",
            customer.current_balance, new_balance, customer.customer_code
        );

        Ok(customer.to_response(addresses))
    }

    pub async fn check_credit_availability(
        &self,
        customer_id: Uuid,
        amount: Decimal,
    ) -> ErpResult<CreditCheckResult> {
        let customer = self
            .repository
            .get_customer_by_id(customer_id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        if !customer.is_active() {
            return Ok(CreditCheckResult {
                approved: false,
                available_credit: Decimal::ZERO,
                message: "Customer account is not active".to_string(),
            });
        }

        let available_credit = customer.available_credit();
        let approved = available_credit >= amount;

        Ok(CreditCheckResult {
            approved,
            available_credit,
            message: if approved {
                "Credit approved".to_string()
            } else {
                format!(
                    "Insufficient credit. Available: {}, Requested: {}",
                    available_credit, amount
                )
            },
        })
    }

    pub async fn get_customers_with_outstanding_balance(&self) -> ErpResult<Vec<CustomerResponse>> {
        let customers = self
            .repository
            .get_customers_with_outstanding_balance()
            .await?;

        let mut results = Vec::new();
        for customer in customers {
            let addresses = self.repository.get_customer_addresses(customer.id).await?;
            results.push(customer.to_response(addresses));
        }

        Ok(results)
    }

    pub async fn get_customers_by_type(
        &self,
        customer_type: CustomerType,
    ) -> ErpResult<Vec<CustomerResponse>> {
        let customers = self
            .repository
            .get_customers_by_type(&customer_type)
            .await?;

        let mut results = Vec::new();
        for customer in customers {
            let addresses = self.repository.get_customer_addresses(customer.id).await?;
            results.push(customer.to_response(addresses));
        }

        Ok(results)
    }

    pub async fn get_customer_statistics(&self) -> ErpResult<CustomerStatistics> {
        let total_customers = self.repository.count_customers().await?;
        let active_customers = self
            .repository
            .count_customers_by_status(&CustomerStatus::Active)
            .await?;
        let inactive_customers = self
            .repository
            .count_customers_by_status(&CustomerStatus::Inactive)
            .await?;
        let suspended_customers = self
            .repository
            .count_customers_by_status(&CustomerStatus::Suspended)
            .await?;
        let blacklisted_customers = self
            .repository
            .count_customers_by_status(&CustomerStatus::Blacklisted)
            .await?;

        let outstanding_balance_customers = self
            .repository
            .get_customers_with_outstanding_balance()
            .await?;
        let total_outstanding_balance = outstanding_balance_customers
            .iter()
            .map(|c| c.current_balance)
            .sum::<Decimal>();

        Ok(CustomerStatistics {
            total_customers,
            active_customers,
            inactive_customers,
            suspended_customers,
            blacklisted_customers,
            customers_with_outstanding_balance: outstanding_balance_customers.len() as i64,
            total_outstanding_balance,
        })
    }

    pub async fn activate_customer(&self, id: Uuid) -> ErpResult<CustomerResponse> {
        let mut customer = self
            .repository
            .get_customer_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        if customer.status == CustomerStatus::Active {
            return Err(ErpError::business_rule("Customer is already active"));
        }

        customer.status = CustomerStatus::Active;
        self.repository.update_customer(id, &customer).await?;

        let addresses = self.repository.get_customer_addresses(customer.id).await?;

        info!(
            "Customer activated: {} ({})",
            customer.customer_code, customer.id
        );

        Ok(customer.to_response(addresses))
    }

    pub async fn suspend_customer(
        &self,
        id: Uuid,
        reason: Option<String>,
    ) -> ErpResult<CustomerResponse> {
        let mut customer = self
            .repository
            .get_customer_by_id(id)
            .await?
            .ok_or_else(|| ErpError::not_found_simple("Customer not found"))?;

        if customer.status == CustomerStatus::Suspended {
            return Err(ErpError::business_rule("Customer is already suspended"));
        }

        customer.status = CustomerStatus::Suspended;
        if let Some(reason) = reason {
            customer.notes = Some(format!("Suspended: {}", reason));
        }

        self.repository.update_customer(id, &customer).await?;

        let addresses = self.repository.get_customer_addresses(customer.id).await?;

        warn!(
            "Customer suspended: {} ({})",
            customer.customer_code, customer.id
        );

        Ok(customer.to_response(addresses))
    }

    // Private validation methods

    async fn validate_create_request(&self, request: &CreateCustomerRequest) -> ErpResult<()> {
        if request.first_name.trim().is_empty() {
            return Err(ErpError::validation_simple("First name is required"));
        }

        // Allow empty last name for Asian naming conventions (Korean, Japanese, Chinese, etc.)
        // The CLI layer handles the logic to put full Asian names in first_name and empty last_name
        if request.last_name.trim().is_empty() {
            // Check if first_name contains non-ASCII characters (likely Asian name)
            let has_non_ascii = request.first_name.chars().any(|c| {
                ('\u{AC00}'..='\u{D7AF}').contains(&c) || // Korean Hangul
                ('\u{3040}'..='\u{309F}').contains(&c) || // Japanese Hiragana
                ('\u{30A0}'..='\u{30FF}').contains(&c) || // Japanese Katakana
                ('\u{4E00}'..='\u{9FAF}').contains(&c) || // CJK Unified Ideographs
                (!c.is_ascii() && c.is_alphabetic()) // Other non-ASCII alphabetic characters
            });

            if !has_non_ascii {
                return Err(ErpError::validation_simple(
                    "Last name is required for Latin names",
                ));
            }
        }

        if validate_email(&request.email).is_err() {
            return Err(ErpError::validation_simple("Invalid email format"));
        }

        if let Some(phone) = &request.phone {
            if !phone.trim().is_empty() && validate_phone(phone).is_err() {
                return Err(ErpError::validation_simple("Invalid phone format"));
            }
        }

        // Validate business-specific fields
        if request.customer_type.requires_tax_id() && request.tax_id.is_none() {
            return Err(ErpError::validation(
                "tax_id",
                format!("{} customers require a tax ID", request.customer_type),
            ));
        }

        if request.customer_type == CustomerType::Business && request.company_name.is_none() {
            return Err(ErpError::validation_simple(
                "Business customers require a company name",
            ));
        }

        // Validate credit limit
        if let Some(credit_limit) = request.credit_limit {
            if credit_limit < Decimal::ZERO {
                return Err(ErpError::validation_simple(
                    "Credit limit cannot be negative",
                ));
            }
            if credit_limit > Decimal::from(1_000_000) {
                return Err(ErpError::validation_simple(
                    "Credit limit cannot exceed 1,000,000",
                ));
            }
        }

        // Validate addresses
        for address in &request.addresses {
            self.validate_address_request(address)?;
        }

        Ok(())
    }

    async fn validate_update_request(
        &self,
        request: &UpdateCustomerRequest,
        _current: &Customer,
    ) -> ErpResult<()> {
        if let Some(first_name) = &request.first_name {
            if first_name.trim().is_empty() {
                return Err(ErpError::validation_simple("First name cannot be empty"));
            }
        }

        if let Some(last_name) = &request.last_name {
            if last_name.trim().is_empty() {
                // Check if first_name contains non-ASCII characters (likely Asian name)
                if let Some(first_name) = &request.first_name {
                    let has_non_ascii = first_name.chars().any(|c| {
                        ('\u{AC00}'..='\u{D7AF}').contains(&c) || // Korean Hangul
                        ('\u{3040}'..='\u{309F}').contains(&c) || // Japanese Hiragana
                        ('\u{30A0}'..='\u{30FF}').contains(&c) || // Japanese Katakana
                        ('\u{4E00}'..='\u{9FAF}').contains(&c) || // CJK Unified Ideographs
                        (!c.is_ascii() && c.is_alphabetic()) // Other non-ASCII alphabetic characters
                    });
                    if !has_non_ascii {
                        return Err(ErpError::validation_simple(
                            "Last name cannot be empty for non-Asian names",
                        ));
                    }
                } else {
                    return Err(ErpError::validation_simple("Last name cannot be empty"));
                }
            }
        }

        if let Some(email) = &request.email {
            if validate_email(email).is_err() {
                return Err(ErpError::validation_simple("Invalid email format"));
            }
        }

        if let Some(phone) = &request.phone {
            if !phone.trim().is_empty() && validate_phone(phone).is_err() {
                return Err(ErpError::validation_simple("Invalid phone format"));
            }
        }

        if let Some(credit_limit) = request.credit_limit {
            if credit_limit < Decimal::ZERO {
                return Err(ErpError::validation_simple(
                    "Credit limit cannot be negative",
                ));
            }
            if credit_limit > Decimal::from(1_000_000) {
                return Err(ErpError::validation_simple(
                    "Credit limit cannot exceed 1,000,000",
                ));
            }
        }

        Ok(())
    }

    fn validate_address_request(&self, request: &CreateAddressRequest) -> ErpResult<()> {
        if request.street_address.trim().is_empty() {
            return Err(ErpError::validation_simple("Street address is required"));
        }

        if request.city.trim().is_empty() {
            return Err(ErpError::validation_simple("City is required"));
        }

        if request.state_province.trim().is_empty() {
            return Err(ErpError::validation_simple("State/Province is required"));
        }

        if request.postal_code.trim().is_empty() {
            return Err(ErpError::validation_simple("Postal code is required"));
        }

        if request.country.trim().is_empty() {
            return Err(ErpError::validation_simple("Country is required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum BalanceOperation {
    Add,
    Subtract,
    Set,
}

#[derive(Debug, Clone)]
pub struct CreditCheckResult {
    pub approved: bool,
    pub available_credit: Decimal,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CustomerStatistics {
    pub total_customers: i64,
    pub active_customers: i64,
    pub inactive_customers: i64,
    pub suspended_customers: i64,
    pub blacklisted_customers: i64,
    pub customers_with_outstanding_balance: i64,
    pub total_outstanding_balance: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::customers::repository::MockCustomerRepository;
    use tokio;

    fn create_test_service() -> CustomerService {
        let repository = Arc::new(MockCustomerRepository::new());
        CustomerService::new(repository)
    }

    fn create_test_customer_request() -> CreateCustomerRequest {
        CreateCustomerRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            company_name: None,
            email: "john.doe@example.com".to_string(),
            phone: Some("+1234567890".to_string()),
            customer_type: CustomerType::Individual,
            credit_limit: Some(Decimal::from(5000)),
            tax_id: None,
            notes: Some("Test customer".to_string()),
            addresses: vec![CreateAddressRequest {
                address_type: AddressType::Both,
                street_address: "123 Main St".to_string(),
                city: "Springfield".to_string(),
                state_province: "IL".to_string(),
                postal_code: "62701".to_string(),
                country: "USA".to_string(),
                is_default: true,
            }],
        }
    }

    #[tokio::test]
    async fn test_create_customer_success() {
        let service = create_test_service();
        let request = create_test_customer_request();

        let result = service.create_customer(request.clone()).await;

        assert!(result.is_ok());
        let customer_response = result.unwrap();
        assert_eq!(customer_response.first_name, request.first_name);
        assert_eq!(customer_response.last_name, request.last_name);
        assert_eq!(customer_response.email, request.email);
        assert_eq!(customer_response.addresses.len(), 1);
        assert!(customer_response.customer_code.starts_with("CUST-JD-"));
    }

    #[tokio::test]
    async fn test_create_customer_duplicate_email() {
        let service = create_test_service();
        let request = create_test_customer_request();

        // Create first customer
        let _ = service.create_customer(request.clone()).await.unwrap();

        // Try to create second customer with same email
        let result = service.create_customer(request).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ErpError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_create_customer_validation_errors() {
        let service = create_test_service();

        // Test empty first name
        let mut request = create_test_customer_request();
        request.first_name = "".to_string();
        let result = service.create_customer(request).await;
        assert!(result.is_err());

        // Test invalid email
        let mut request = create_test_customer_request();
        request.email = "invalid-email".to_string();
        let result = service.create_customer(request).await;
        assert!(result.is_err());

        // Test business customer without company name
        let mut request = create_test_customer_request();
        request.customer_type = CustomerType::Business;
        request.company_name = None;
        let result = service.create_customer(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_customer_by_id() {
        let service = create_test_service();
        let request = create_test_customer_request();

        // Create customer
        let created = service.create_customer(request).await.unwrap();

        // Get customer by ID
        let result = service.get_customer_by_id(created.id).await;

        assert!(result.is_ok());
        let customer = result.unwrap();
        assert_eq!(customer.id, created.id);
        assert_eq!(customer.email, created.email);
    }

    #[tokio::test]
    async fn test_get_customer_by_code() {
        let service = create_test_service();
        let request = create_test_customer_request();

        // Create customer
        let created = service.create_customer(request).await.unwrap();

        // Get customer by code
        let result = service.get_customer_by_code(&created.customer_code).await;

        assert!(result.is_ok());
        let customer = result.unwrap();
        assert_eq!(customer.customer_code, created.customer_code);
        assert_eq!(customer.email, created.email);
    }

    #[tokio::test]
    async fn test_update_customer() {
        let service = create_test_service();
        let request = create_test_customer_request();

        // Create customer
        let created = service.create_customer(request).await.unwrap();

        // Update customer
        let update_request = UpdateCustomerRequest {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            company_name: Some("Acme Corp".to_string()),
            email: Some("jane.smith@acme.com".to_string()),
            phone: None,
            customer_type: Some(CustomerType::Business),
            status: Some(CustomerStatus::Active),
            credit_limit: Some(Decimal::from(10000)),
            tax_id: Some("123456789".to_string()),
            notes: Some("Updated customer".to_string()),
        };

        let result = service.update_customer(created.id, update_request).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.first_name, "Jane");
        assert_eq!(updated.last_name, "Smith");
        assert_eq!(updated.email, "jane.smith@acme.com");
        assert_eq!(updated.customer_type, CustomerType::Business);
        assert_eq!(updated.credit_limit, Decimal::from(10000));
    }

    #[tokio::test]
    async fn test_delete_customer() {
        let service = create_test_service();
        let request = create_test_customer_request();

        // Create customer
        let created = service.create_customer(request).await.unwrap();

        // Delete customer
        let result = service.delete_customer(created.id).await;

        assert!(result.is_ok());

        // Verify customer is deleted
        let get_result = service.get_customer_by_id(created.id).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_search_customers() {
        let service = create_test_service();

        // Create test customers
        let mut request1 = create_test_customer_request();
        request1.email = "john.doe@example.com".to_string();
        request1.first_name = "John".to_string();

        let mut request2 = create_test_customer_request();
        request2.email = "jane.smith@example.com".to_string();
        request2.first_name = "Jane".to_string();
        request2.last_name = "Smith".to_string();

        service.create_customer(request1).await.unwrap();
        service.create_customer(request2).await.unwrap();

        // Search by first name
        let result = service.search_customers("John", 10).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].first_name, "John");

        // Search by email domain
        let result = service.search_customers("example.com", 10).await.unwrap();
        assert_eq!(result.len(), 2);

        // Search with no results
        let result = service.search_customers("nonexistent", 10).await.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_credit_check() {
        let service = create_test_service();
        let request = create_test_customer_request();

        // Create customer with 5000 credit limit
        let created = service.create_customer(request).await.unwrap();

        // Check credit for amount within limit
        let result = service
            .check_credit_availability(created.id, Decimal::from(3000))
            .await
            .unwrap();
        assert!(result.approved);
        assert_eq!(result.available_credit, Decimal::from(5000));

        // Check credit for amount exceeding limit
        let result = service
            .check_credit_availability(created.id, Decimal::from(6000))
            .await
            .unwrap();
        assert!(!result.approved);
        assert_eq!(result.available_credit, Decimal::from(5000));
    }

    #[tokio::test]
    async fn test_customer_balance_operations() {
        let service = create_test_service();
        let request = create_test_customer_request();

        // Create customer
        let created = service.create_customer(request).await.unwrap();
        assert_eq!(created.current_balance, Decimal::ZERO);

        // Add to balance
        let result = service
            .update_customer_balance(created.id, Decimal::from(1000), BalanceOperation::Add)
            .await
            .unwrap();
        assert_eq!(result.current_balance, Decimal::from(1000));

        // Subtract from balance
        let result = service
            .update_customer_balance(created.id, Decimal::from(300), BalanceOperation::Subtract)
            .await
            .unwrap();
        assert_eq!(result.current_balance, Decimal::from(700));

        // Set balance
        let result = service
            .update_customer_balance(created.id, Decimal::from(2000), BalanceOperation::Set)
            .await
            .unwrap();
        assert_eq!(result.current_balance, Decimal::from(2000));
    }
}
