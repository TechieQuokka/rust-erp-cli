use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::cli::parser::CustomerCommands;
use crate::cli::validator::CliValidator;
use crate::core::config::AppConfig;
use crate::modules::customers::{
    AddressType, BalanceOperation, CreateAddressRequest, CreateCustomerRequest, CustomerFilter,
    CustomerService, CustomerType, PostgresCustomerRepository, UpdateCustomerRequest,
};
use crate::utils::error::{ErpError, ErpResult};

pub struct CustomerHandler;

impl CustomerHandler {
    pub async fn handle(cmd: &CustomerCommands, _config: &AppConfig) -> ErpResult<()> {
        use crate::core::database::DatabaseManager;

        let connection = DatabaseManager::get_connection().await?;
        let pool = connection.pool().clone();

        let repository = Arc::new(PostgresCustomerRepository::new(Arc::new(pool)));
        let service = CustomerService::new(repository);

        match cmd {
            CustomerCommands::Add {
                name,
                first_name,
                last_name,
                email,
                phone,
                address,
                company,
                tax_id,
                notes,
            } => {
                Self::handle_add(
                    &service, name, first_name, last_name, email, phone, address, company, tax_id,
                    notes,
                )
                .await
            }

            CustomerCommands::List {
                search,
                customer_type,
                page,
                limit,
                format,
                sort_by,
                order,
            } => {
                Self::handle_list(
                    &service,
                    search,
                    customer_type,
                    *page,
                    *limit,
                    format,
                    sort_by,
                    order,
                )
                .await
            }

            CustomerCommands::Update {
                id,
                name,
                email,
                phone,
                address,
            } => Self::handle_update(&service, id, name, email, phone, address).await,

            CustomerCommands::Delete { id, force } => {
                Self::handle_delete(&service, id, *force).await
            }

            CustomerCommands::Search { query, field } => {
                Self::handle_search(&service, query, field).await
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_add(
        service: &CustomerService,
        name: &Option<String>,
        first_name: &Option<String>,
        last_name: &Option<String>,
        email: &str,
        phone: &Option<String>,
        address: &Option<String>,
        company: &Option<String>,
        tax_id: &Option<String>,
        _notes: &Option<String>,
    ) -> ErpResult<()> {
        // Determine name handling strategy
        let (final_first_name, final_last_name) = if company.is_some() {
            // Business customer: use name as company representative or use first/last name options
            match (name, first_name, last_name) {
                (Some(n), None, None) => {
                    // Single name provided for business
                    (n.clone(), "".to_string())
                }
                (None, Some(f), Some(l)) => {
                    // First and last name provided
                    (f.clone(), l.clone())
                }
                (None, Some(f), None) => {
                    // Only first name provided
                    (f.clone(), "".to_string())
                }
                _ => {
                    return Err(ErpError::validation_simple(
                        "For business customers, provide either name or first-name (and optionally last-name)",
                    ));
                }
            }
        } else {
            // Individual customer: require either name with space or first/last name options
            match (name, first_name, last_name) {
                (Some(n), None, None) => {
                    // Full name provided, split by space
                    let parts: Vec<&str> = n.split_whitespace().collect();
                    if parts.len() < 2 {
                        return Err(ErpError::validation_simple(
                            "For individual customers, provide full name with space (e.g., '김 철수') or use --first-name and --last-name options",
                        ));
                    }
                    (parts[0].to_string(), parts[1..].join(" "))
                }
                (None, Some(f), Some(l)) => {
                    // First and last name provided separately
                    (f.clone(), l.clone())
                }
                _ => {
                    return Err(ErpError::validation_simple(
                        "For individual customers, provide either full name or both --first-name and --last-name",
                    ));
                }
            }
        };

        // Validate required email
        let validated_email = CliValidator::validate_email_optional(&Some(email.to_string()))?
            .ok_or_else(|| ErpError::validation_simple("Valid email is required"))?;

        // Determine customer type based on company field
        let customer_type = if company.is_some() {
            CustomerType::Business
        } else {
            CustomerType::Individual
        };

        // Validate phone
        let validated_phone = CliValidator::validate_phone_optional(phone)?;

        // Create address if provided
        let mut addresses = Vec::new();
        if let Some(addr_str) = address {
            if !addr_str.trim().is_empty() {
                // Parse simple address format: "street, city, state, zip, country"
                let addr_parts: Vec<&str> = addr_str.split(',').map(|s| s.trim()).collect();
                if addr_parts.len() >= 3 {
                    addresses.push(CreateAddressRequest {
                        address_type: AddressType::Both,
                        street_address: addr_parts[0].to_string(),
                        city: addr_parts.get(1).unwrap_or(&"").to_string(),
                        state_province: addr_parts.get(2).unwrap_or(&"").to_string(),
                        postal_code: addr_parts.get(3).unwrap_or(&"00000").to_string(),
                        country: addr_parts.get(4).unwrap_or(&"USA").to_string(),
                        is_default: true,
                    });
                }
            }
        }

        // Create customer request
        let request = CreateCustomerRequest {
            first_name: final_first_name,
            last_name: final_last_name,
            company_name: if customer_type == CustomerType::Business {
                company.clone()
            } else {
                None
            },
            email: validated_email,
            phone: validated_phone,
            customer_type: customer_type.clone(),
            credit_limit: Some(customer_type.default_credit_limit()),
            tax_id: tax_id.clone(),
            notes: None,
            addresses,
        };

        // Create customer
        let customer = service.create_customer(request).await?;

        println!("✅ Customer created successfully!");
        println!("Customer Code: {}", customer.customer_code);
        println!("Name: {}", customer.display_name());
        println!("Email: {}", customer.email);
        if let Some(phone) = customer.phone {
            println!("Phone: {}", phone);
        }
        println!("Type: {}", customer.customer_type);
        println!("Status: {}", customer.status);
        println!("Credit Limit: ${}", customer.credit_limit);
        println!("Available Credit: ${}", customer.available_credit);

        if !customer.addresses.is_empty() {
            println!("\nAddresses:");
            for (i, addr) in customer.addresses.iter().enumerate() {
                println!(
                    "  {}. {} ({}{})",
                    i + 1,
                    addr.formatted_address().replace('\n', ", "),
                    addr.address_type,
                    if addr.is_default { ", Default" } else { "" }
                );
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_list(
        service: &CustomerService,
        search: &Option<String>,
        customer_type: &Option<String>,
        page: u32,
        limit: u32,
        format: &str,
        sort_by: &str,
        order: &str,
    ) -> ErpResult<()> {
        // Validate pagination
        let (validated_page, validated_limit) = CliValidator::validate_pagination(page, limit)?;

        // Parse customer type filter
        let type_filter = match customer_type.as_deref() {
            Some("individual") | Some("i") => Some(CustomerType::Individual),
            Some("business") | Some("b") => Some(CustomerType::Business),
            Some("wholesale") | Some("w") => Some(CustomerType::Wholesale),
            Some("retail") | Some("r") => Some(CustomerType::Retail),
            Some(_) => return Err(ErpError::validation_simple("Invalid customer type filter")),
            None => None,
        };

        // Create filter
        let filter = CustomerFilter {
            status: None,
            customer_type: type_filter,
            search: search
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            city: None,
            state_province: None,
            country: None,
            has_outstanding_balance: None,
            credit_limit_min: None,
            credit_limit_max: None,
        };

        // Get customers
        let result = service
            .list_customers(filter, validated_page, validated_limit, sort_by, order)
            .await?;

        if result.customers.is_empty() {
            println!("No customers found.");
            return Ok(());
        }

        match format {
            "json" => {
                // Output as JSON
                use serde_json::json;
                let output = json!({
                    "status": "success",
                    "data": result.customers,
                    "meta": {
                        "total": result.total,
                        "page": result.page,
                        "per_page": result.per_page,
                        "total_pages": (result.total + result.per_page as i64 - 1) / result.per_page as i64
                    }
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            "csv" => {
                // Output as CSV
                println!("Code,Name,Email,Type,Status,Credit Limit,Balance,Available");
                for customer in &result.customers {
                    println!(
                        "{},{},{},{},{},{},{},{}",
                        customer.customer_code,
                        customer.display_name(),
                        customer.email,
                        customer.customer_type,
                        customer.status,
                        customer.credit_limit,
                        customer.current_balance,
                        customer.available_credit
                    );
                }
            }
            _ => {
                // Default table format
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_header(vec![
                        "Code",
                        "Name",
                        "Email",
                        "Type",
                        "Status",
                        "Credit Limit",
                        "Balance",
                        "Available",
                    ]);

                for customer in &result.customers {
                    table.add_row(vec![
                        customer.customer_code.clone(),
                        customer.display_name(),
                        customer.email.clone(),
                        customer.customer_type.to_string(),
                        customer.status.to_string(),
                        format!("${}", customer.credit_limit),
                        format!("${}", customer.current_balance),
                        format!("${}", customer.available_credit),
                    ]);
                }

                println!("{}", table);
                println!(
                    "\nShowing {} of {} customers (Page {} of {})",
                    result.customers.len(),
                    result.total,
                    result.page,
                    (result.total + result.per_page as i64 - 1) / result.per_page as i64
                );
            }
        }

        Ok(())
    }

    async fn handle_update(
        service: &CustomerService,
        id: &str,
        name: &Option<String>,
        email: &Option<String>,
        phone: &Option<String>,
        address: &Option<String>,
    ) -> ErpResult<()> {
        // Parse customer ID or code
        let customer_id = if let Ok(uuid) = Uuid::parse_str(id) {
            uuid
        } else {
            // Try to find by customer code
            let customer = service.get_customer_by_code(id).await?;
            customer.id
        };

        // Parse name if provided
        let (first_name, last_name) = if let Some(name_str) = name {
            let parts: Vec<&str> = name_str.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(ErpError::validation_simple(
                    "Please provide both first and last name",
                ));
            }
            (Some(parts[0].to_string()), Some(parts[1..].join(" ")))
        } else {
            (None, None)
        };

        // Validate email if provided
        let validated_email = if email.is_some() {
            CliValidator::validate_email_optional(email)?
        } else {
            None
        };

        // Validate phone if provided
        let validated_phone = if phone.is_some() {
            CliValidator::validate_phone_optional(phone)?
        } else {
            None
        };

        // Create update request
        let update_request = UpdateCustomerRequest {
            first_name,
            last_name,
            company_name: None, // Not updating company name in this simple CLI
            email: validated_email,
            phone: validated_phone,
            customer_type: None,
            status: None,
            credit_limit: None,
            tax_id: None,
            notes: None,
        };

        // Update customer
        let updated_customer = service.update_customer(customer_id, update_request).await?;

        println!("✅ Customer updated successfully!");
        println!("Customer Code: {}", updated_customer.customer_code);
        println!("Name: {}", updated_customer.display_name());
        println!("Email: {}", updated_customer.email);
        if let Some(phone) = updated_customer.phone {
            println!("Phone: {}", phone);
        }
        println!("Status: {}", updated_customer.status);

        // Handle address update (simplified - just show current addresses)
        if address.is_some() {
            println!("\nNote: Address update requires separate address management commands");
        }

        if !updated_customer.addresses.is_empty() {
            println!("\nCurrent Addresses:");
            for (i, addr) in updated_customer.addresses.iter().enumerate() {
                println!(
                    "  {}. {} ({}{})",
                    i + 1,
                    addr.formatted_address().replace('\n', ", "),
                    addr.address_type,
                    if addr.is_default { ", Default" } else { "" }
                );
            }
        }

        Ok(())
    }

    async fn handle_delete(service: &CustomerService, id: &str, force: bool) -> ErpResult<()> {
        // Parse customer ID or code
        let customer_id = if let Ok(uuid) = Uuid::parse_str(id) {
            uuid
        } else {
            // Try to find by customer code
            let customer = service.get_customer_by_code(id).await?;
            customer.id
        };

        // Get customer details for confirmation
        let customer = service.get_customer_by_id(customer_id).await?;

        if !force {
            println!("Customer to delete:");
            println!("  Code: {}", customer.customer_code);
            println!("  Name: {}", customer.display_name());
            println!("  Email: {}", customer.email);
            println!("  Current Balance: ${}", customer.current_balance);

            if customer.has_outstanding_balance() {
                println!(
                    "\n⚠️  Warning: Customer has outstanding balance of ${}",
                    customer.current_balance
                );
                println!("Use --force to delete anyway, or collect payment first.");
                return Ok(());
            }

            // Interactive confirmation prompt
            print!("\nAre you sure you want to delete this customer? (y/N): ");
            std::io::Write::flush(&mut std::io::stdout())
                .map_err(|e| ErpError::internal(format!("Failed to flush stdout: {}", e)))?;

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| ErpError::internal(format!("Failed to read user input: {}", e)))?;

            let input = input.trim().to_lowercase();
            if input != "y" && input != "yes" {
                println!("❌ Deletion cancelled.");
                return Ok(());
            }
        }

        // Attempt to delete
        service.delete_customer(customer_id).await?;

        println!("✅ Customer deleted successfully!");
        println!(
            "Deleted: {} ({})",
            customer.display_name(),
            customer.customer_code
        );

        Ok(())
    }

    async fn handle_search(
        service: &CustomerService,
        query: &str,
        _field: &Option<String>,
    ) -> ErpResult<()> {
        if query.trim().is_empty() {
            return Err(ErpError::validation_simple("Search query cannot be empty"));
        }

        // Search customers
        let customers = service.search_customers(query.trim(), 20).await?;

        if customers.is_empty() {
            println!("No customers found for query: '{}'", query.trim());
            return Ok(());
        }

        // Create table
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Code", "Name", "Email", "Phone", "Type", "Status", "Balance",
            ]);

        for customer in &customers {
            table.add_row(vec![
                customer.customer_code.clone(),
                customer.display_name(),
                customer.email.clone(),
                customer.phone.clone().unwrap_or_default(),
                customer.customer_type.to_string(),
                customer.status.to_string(),
                format!("${}", customer.current_balance),
            ]);
        }

        println!("{}", table);
        println!(
            "\nFound {} customer(s) matching '{}'",
            customers.len(),
            query.trim()
        );

        Ok(())
    }

    // Additional helper methods for extended functionality
    pub async fn handle_balance(
        service: &CustomerService,
        customer_id: &str,
        operation: &str,
        amount: Decimal,
    ) -> ErpResult<()> {
        // Parse customer ID or code
        let id = if let Ok(uuid) = Uuid::parse_str(customer_id) {
            uuid
        } else {
            let customer = service.get_customer_by_code(customer_id).await?;
            customer.id
        };

        let balance_op = match operation.to_lowercase().as_str() {
            "add" | "+" => BalanceOperation::Add,
            "subtract" | "sub" | "-" => BalanceOperation::Subtract,
            "set" | "=" => BalanceOperation::Set,
            _ => {
                return Err(ErpError::validation_simple(
                    "Invalid operation. Use: add, subtract, or set",
                ))
            }
        };

        let updated_customer = service
            .update_customer_balance(id, amount, balance_op)
            .await?;

        println!("✅ Customer balance updated!");
        println!(
            "Customer: {} ({})",
            updated_customer.display_name(),
            updated_customer.customer_code
        );
        println!("New Balance: ${}", updated_customer.current_balance);
        println!("Available Credit: ${}", updated_customer.available_credit);

        Ok(())
    }

    pub async fn handle_credit_check(
        service: &CustomerService,
        customer_id: &str,
        amount: Decimal,
    ) -> ErpResult<()> {
        // Parse customer ID or code
        let id = if let Ok(uuid) = Uuid::parse_str(customer_id) {
            uuid
        } else {
            let customer = service.get_customer_by_code(customer_id).await?;
            customer.id
        };

        let result = service.check_credit_availability(id, amount).await?;

        println!("💳 Credit Check Results");
        println!("Amount Requested: ${}", amount);
        println!("Available Credit: ${}", result.available_credit);
        println!(
            "Status: {}",
            if result.approved {
                "✅ APPROVED"
            } else {
                "❌ DECLINED"
            }
        );
        println!("Message: {}", result.message);

        Ok(())
    }

    pub async fn handle_statistics(service: &CustomerService) -> ErpResult<()> {
        let stats = service.get_customer_statistics().await?;

        println!("📊 Customer Statistics");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total Customers: {}", stats.total_customers);
        println!("Active: {}", stats.active_customers);
        println!("Inactive: {}", stats.inactive_customers);
        println!("Suspended: {}", stats.suspended_customers);
        println!("Blacklisted: {}", stats.blacklisted_customers);
        println!();
        println!("Outstanding Balances:");
        println!(
            "  Customers with Balance: {}",
            stats.customers_with_outstanding_balance
        );
        println!("  Total Outstanding: ${}", stats.total_outstanding_balance);

        if stats.total_customers > 0 {
            let active_percentage =
                (stats.active_customers as f64 / stats.total_customers as f64) * 100.0;
            println!("  Active Rate: {:.1}%", active_percentage);
        }

        Ok(())
    }
}
