use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub customer_code: String,
    pub first_name: String,
    pub last_name: String,
    pub company_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub customer_type: CustomerType,
    pub status: CustomerStatus,
    pub credit_limit: Decimal,
    pub current_balance: Decimal,
    pub tax_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CustomerAddress {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub address_type: AddressType,
    pub street_address: String,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "customer_type", rename_all = "lowercase")]
pub enum CustomerType {
    Individual,
    Business,
    Wholesale,
    Retail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "customer_status", rename_all = "lowercase")]
pub enum CustomerStatus {
    Active,
    Inactive,
    Suspended,
    Blacklisted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "address_type", rename_all = "lowercase")]
pub enum AddressType {
    Billing,
    Shipping,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    pub first_name: String,
    pub last_name: String,
    pub company_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub customer_type: CustomerType,
    pub credit_limit: Option<Decimal>,
    pub tax_id: Option<String>,
    pub notes: Option<String>,
    pub addresses: Vec<CreateAddressRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAddressRequest {
    pub address_type: AddressType,
    pub street_address: String,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCustomerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub customer_type: Option<CustomerType>,
    pub status: Option<CustomerStatus>,
    pub credit_limit: Option<Decimal>,
    pub tax_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub id: Uuid,
    pub customer_code: String,
    pub first_name: String,
    pub last_name: String,
    pub company_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub customer_type: CustomerType,
    pub status: CustomerStatus,
    pub credit_limit: Decimal,
    pub current_balance: Decimal,
    pub available_credit: Decimal,
    pub tax_id: Option<String>,
    pub notes: Option<String>,
    pub addresses: Vec<CustomerAddress>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CustomerResponse {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn display_name(&self) -> String {
        if let Some(company) = &self.company_name {
            format!("{} ({})", company, self.full_name())
        } else {
            self.full_name()
        }
    }

    pub fn has_outstanding_balance(&self) -> bool {
        self.current_balance > Decimal::ZERO
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerListResponse {
    pub customers: Vec<CustomerResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerFilter {
    pub status: Option<CustomerStatus>,
    pub customer_type: Option<CustomerType>,
    pub search: Option<String>, // Search in name, email, company
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub country: Option<String>,
    pub has_outstanding_balance: Option<bool>,
    pub credit_limit_min: Option<Decimal>,
    pub credit_limit_max: Option<Decimal>,
}

impl Customer {
    pub fn new(request: CreateCustomerRequest) -> Self {
        let now = Utc::now();
        let customer_code = Self::generate_customer_code(&request.first_name, &request.last_name);

        Self {
            id: Uuid::new_v4(),
            customer_code,
            first_name: request.first_name,
            last_name: request.last_name,
            company_name: request.company_name,
            email: request.email,
            phone: request.phone,
            customer_type: request.customer_type,
            status: CustomerStatus::Active,
            credit_limit: request.credit_limit.unwrap_or(Decimal::ZERO),
            current_balance: Decimal::ZERO,
            tax_id: request.tax_id,
            notes: request.notes,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, request: UpdateCustomerRequest) {
        if let Some(first_name) = request.first_name {
            self.first_name = first_name;
        }
        if let Some(last_name) = request.last_name {
            self.last_name = last_name;
        }
        if let Some(company_name) = request.company_name {
            self.company_name = Some(company_name);
        }
        if let Some(email) = request.email {
            self.email = email;
        }
        if let Some(phone) = request.phone {
            self.phone = Some(phone);
        }
        if let Some(customer_type) = request.customer_type {
            self.customer_type = customer_type;
        }
        if let Some(status) = request.status {
            self.status = status;
        }
        if let Some(credit_limit) = request.credit_limit {
            self.credit_limit = credit_limit;
        }
        if let Some(tax_id) = request.tax_id {
            self.tax_id = Some(tax_id);
        }
        if let Some(notes) = request.notes {
            self.notes = Some(notes);
        }
        self.updated_at = Utc::now();
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn display_name(&self) -> String {
        if let Some(company) = &self.company_name {
            format!("{} ({})", company, self.full_name())
        } else {
            self.full_name()
        }
    }

    pub fn available_credit(&self) -> Decimal {
        self.credit_limit - self.current_balance
    }

    pub fn has_available_credit(&self, amount: Decimal) -> bool {
        self.available_credit() >= amount
    }

    pub fn is_active(&self) -> bool {
        self.status == CustomerStatus::Active
    }

    pub fn can_place_order(&self) -> bool {
        matches!(self.status, CustomerStatus::Active)
    }

    pub fn update_balance(&mut self, amount: Decimal) {
        self.current_balance += amount;
        self.updated_at = Utc::now();
    }

    pub fn has_outstanding_balance(&self) -> bool {
        self.current_balance > Decimal::ZERO
    }

    fn generate_customer_code(first_name: &str, last_name: &str) -> String {
        let prefix = format!(
            "{}{}",
            first_name.chars().next().unwrap_or('X').to_uppercase(),
            last_name.chars().next().unwrap_or('X').to_uppercase()
        );
        let timestamp = Utc::now().timestamp();
        format!("CUST-{}-{}", prefix, timestamp % 100000)
    }

    pub fn to_response(&self, addresses: Vec<CustomerAddress>) -> CustomerResponse {
        CustomerResponse {
            id: self.id,
            customer_code: self.customer_code.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            company_name: self.company_name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            customer_type: self.customer_type.clone(),
            status: self.status.clone(),
            credit_limit: self.credit_limit,
            current_balance: self.current_balance,
            available_credit: self.available_credit(),
            tax_id: self.tax_id.clone(),
            notes: self.notes.clone(),
            addresses,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl CustomerAddress {
    pub fn new(customer_id: Uuid, request: CreateAddressRequest) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            customer_id,
            address_type: request.address_type,
            street_address: request.street_address,
            city: request.city,
            state_province: request.state_province,
            postal_code: request.postal_code,
            country: request.country,
            is_default: request.is_default,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn formatted_address(&self) -> String {
        format!(
            "{}\n{}, {} {}\n{}",
            self.street_address, self.city, self.state_province, self.postal_code, self.country
        )
    }

    pub fn is_billing_address(&self) -> bool {
        matches!(self.address_type, AddressType::Billing | AddressType::Both)
    }

    pub fn is_shipping_address(&self) -> bool {
        matches!(self.address_type, AddressType::Shipping | AddressType::Both)
    }
}

impl CustomerType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Individual,
            Self::Business,
            Self::Wholesale,
            Self::Retail,
        ]
    }

    pub fn default_credit_limit(&self) -> Decimal {
        match self {
            Self::Individual => Decimal::from(1000),
            Self::Business => Decimal::from(10000),
            Self::Wholesale => Decimal::from(50000),
            Self::Retail => Decimal::from(5000),
        }
    }

    pub fn requires_tax_id(&self) -> bool {
        matches!(self, Self::Business | Self::Wholesale)
    }
}

impl CustomerStatus {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Active,
            Self::Inactive,
            Self::Suspended,
            Self::Blacklisted,
        ]
    }

    pub fn can_place_orders(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn can_receive_credit(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl AddressType {
    pub fn all() -> Vec<Self> {
        vec![Self::Billing, Self::Shipping, Self::Both]
    }
}

impl std::fmt::Display for CustomerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Individual => write!(f, "individual"),
            Self::Business => write!(f, "business"),
            Self::Wholesale => write!(f, "wholesale"),
            Self::Retail => write!(f, "retail"),
        }
    }
}

impl std::fmt::Display for CustomerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Suspended => write!(f, "suspended"),
            Self::Blacklisted => write!(f, "blacklisted"),
        }
    }
}

impl std::fmt::Display for AddressType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Billing => write!(f, "billing"),
            Self::Shipping => write!(f, "shipping"),
            Self::Both => write!(f, "both"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_creation() {
        let request = CreateCustomerRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            company_name: Some("Acme Corp".to_string()),
            email: "john@acme.com".to_string(),
            phone: Some("+1234567890".to_string()),
            customer_type: CustomerType::Business,
            credit_limit: Some(Decimal::from(10000)),
            tax_id: Some("12345678901".to_string()),
            notes: Some("VIP customer".to_string()),
            addresses: vec![],
        };

        let customer = Customer::new(request);

        assert_eq!(customer.first_name, "John");
        assert_eq!(customer.last_name, "Doe");
        assert_eq!(customer.company_name, Some("Acme Corp".to_string()));
        assert_eq!(customer.customer_type, CustomerType::Business);
        assert_eq!(customer.status, CustomerStatus::Active);
        assert_eq!(customer.credit_limit, Decimal::from(10000));
        assert_eq!(customer.current_balance, Decimal::ZERO);
        assert!(customer.customer_code.starts_with("CUST-JD-"));
    }

    #[test]
    fn test_customer_display_name() {
        let mut customer = create_test_customer();

        customer.company_name = Some("Test Company".to_string());
        assert_eq!(customer.display_name(), "Test Company (John Doe)");

        customer.company_name = None;
        assert_eq!(customer.display_name(), "John Doe");
    }

    #[test]
    fn test_customer_credit_management() {
        let mut customer = create_test_customer();
        customer.credit_limit = Decimal::from(1000);
        customer.current_balance = Decimal::from(200);

        assert_eq!(customer.available_credit(), Decimal::from(800));
        assert!(customer.has_available_credit(Decimal::from(500)));
        assert!(!customer.has_available_credit(Decimal::from(1000)));
        assert!(customer.has_outstanding_balance());

        customer.update_balance(Decimal::from(300));
        assert_eq!(customer.current_balance, Decimal::from(500));
        assert_eq!(customer.available_credit(), Decimal::from(500));
    }

    #[test]
    fn test_customer_status_checks() {
        let mut customer = create_test_customer();

        assert!(customer.is_active());
        assert!(customer.can_place_order());

        customer.status = CustomerStatus::Suspended;
        assert!(!customer.is_active());
        assert!(!customer.can_place_order());
    }

    #[test]
    fn test_customer_address_creation() {
        let customer_id = Uuid::new_v4();
        let request = CreateAddressRequest {
            address_type: AddressType::Both,
            street_address: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            state_province: "IL".to_string(),
            postal_code: "62701".to_string(),
            country: "USA".to_string(),
            is_default: true,
        };

        let address = CustomerAddress::new(customer_id, request);

        assert_eq!(address.customer_id, customer_id);
        assert_eq!(address.address_type, AddressType::Both);
        assert!(address.is_billing_address());
        assert!(address.is_shipping_address());
        assert!(address.is_default);

        let formatted = address.formatted_address();
        assert!(formatted.contains("123 Main St"));
        assert!(formatted.contains("Springfield, IL 62701"));
        assert!(formatted.contains("USA"));
    }

    #[test]
    fn test_customer_type_defaults() {
        assert_eq!(
            CustomerType::Business.default_credit_limit(),
            Decimal::from(10000)
        );
        assert!(CustomerType::Business.requires_tax_id());
        assert!(!CustomerType::Individual.requires_tax_id());
    }

    fn create_test_customer() -> Customer {
        let request = CreateCustomerRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            company_name: None,
            email: "john@example.com".to_string(),
            phone: None,
            customer_type: CustomerType::Individual,
            credit_limit: None,
            tax_id: None,
            notes: None,
            addresses: vec![],
        };

        Customer::new(request)
    }
}
