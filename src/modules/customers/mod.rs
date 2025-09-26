//! Customer management module
//!
//! This module provides comprehensive customer relationship management functionality
//! including customer creation, updating, search, credit management, and address handling.

pub mod models;
pub mod repository;
pub mod service;

pub use models::*;
pub use repository::{CustomerRepository, MockCustomerRepository, PostgresCustomerRepository};
pub use service::{BalanceOperation, CreditCheckResult, CustomerService, CustomerStatistics};
