use chrono::{DateTime, Utc};
use comfy_table::{Cell, Color, Table};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::cli::parser::SalesCommands;
use crate::core::config::AppConfig;
use crate::core::database::connection::DatabaseManager;
use crate::modules::customers::{CustomerService, PostgresCustomerRepository};
use crate::modules::inventory::{InventoryServiceImpl, PostgresInventoryRepository};
use crate::modules::sales::*;
use crate::utils::error::{ErpError, ErpResult};
use crate::utils::validation::ValidationService;

pub struct SalesHandler;

impl SalesHandler {
    pub async fn handle(cmd: &SalesCommands, config: &AppConfig) -> ErpResult<()> {
        DatabaseManager::initialize(config.database.clone()).await?;
        let connection = DatabaseManager::get_connection().await?;
        let pool = connection.pool().clone();

        let sales_repository = Box::new(PostgresSalesRepository::new(pool.clone()));
        let validation_service = ValidationService::new();

        let customer_repository = std::sync::Arc::new(PostgresCustomerRepository::new(
            std::sync::Arc::new(pool.clone()),
        ));
        let customer_service = CustomerService::new(customer_repository);

        let inventory_repository =
            std::sync::Arc::new(PostgresInventoryRepository::new(pool.clone()));
        let inventory_service = InventoryServiceImpl::new(inventory_repository);

        let sales_service = SalesService::new(sales_repository, validation_service)
            .with_customer_service(customer_service)
            .with_inventory_service(Box::new(inventory_service));
        match cmd {
            SalesCommands::CreateOrder {
                customer,
                items,
                discount,
                notes,
            } => Self::handle_create_order(&sales_service, customer, items, discount, notes).await,
            SalesCommands::ListOrders {
                status,
                customer,
                from_date,
                to_date,
                page,
                limit,
            } => {
                Self::handle_list_orders(
                    &sales_service,
                    status,
                    customer,
                    from_date,
                    to_date,
                    *page,
                    *limit,
                )
                .await
            }
            SalesCommands::UpdateOrder { id, status, notes } => {
                Self::handle_update_order(&sales_service, id, status, notes).await
            }
            SalesCommands::GenerateInvoice {
                order_id,
                output,
                format,
            } => Self::handle_generate_invoice(&sales_service, order_id, output, format).await,
        }
    }

    async fn handle_create_order(
        sales_service: &SalesService,
        customer: &str,
        items: &[String],
        discount: &Option<f64>,
        notes: &Option<String>,
    ) -> ErpResult<()> {
        let customer_id = Uuid::from_str(customer)
            .map_err(|_| ErpError::validation("customer_id", "Invalid customer ID format"))?;

        let mut order_items = Vec::new();
        for item_str in items {
            let parts: Vec<&str> = item_str.split(':').collect();
            if parts.len() != 2 {
                return Err(ErpError::validation(
                    "item",
                    "format should be 'product_id:quantity'",
                ));
            }

            let product_id = Uuid::from_str(parts[0])
                .map_err(|_| ErpError::validation("product_id", "invalid format"))?;
            let quantity = parts[1]
                .parse::<i32>()
                .map_err(|_| ErpError::validation("quantity", "invalid format"))?;

            order_items.push(OrderItemRequest {
                product_id,
                quantity,
                unit_price: None,
                discount: None,
            });
        }

        let discount_amount =
            discount.map(|d| Decimal::from_f64_retain(d).unwrap_or(Decimal::ZERO));

        let request = CreateOrderRequest {
            customer_id,
            items: order_items,
            shipping_address: None,
            billing_address: None,
            payment_method: None,
            notes: notes.clone(),
            discount_amount,
        };

        match sales_service.create_order(request).await {
            Ok(order_summary) => {
                println!("✅ Order created successfully!");
                Self::display_order_summary(&order_summary);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to create order: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_list_orders(
        sales_service: &SalesService,
        status: &Option<String>,
        customer: &Option<String>,
        from_date: &Option<String>,
        to_date: &Option<String>,
        page: u32,
        limit: u32,
    ) -> ErpResult<()> {
        let offset = (page.saturating_sub(1)) as i64 * limit as i64;

        let orders = if let Some(customer_str) = customer {
            let customer_id = Uuid::from_str(customer_str)
                .map_err(|_| ErpError::validation("customer_id", "invalid format"))?;
            sales_service.get_orders_by_customer(customer_id).await?
        } else if from_date.is_some() || to_date.is_some() {
            let start_date = if let Some(from) = from_date {
                DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", from))
                    .map_err(|_| {
                        ErpError::validation("from_date", "invalid format (use YYYY-MM-DD)")
                    })?
                    .with_timezone(&Utc)
            } else {
                DateTime::from_timestamp(0, 0).unwrap_or_default()
            };

            let end_date = if let Some(to) = to_date {
                DateTime::parse_from_rfc3339(&format!("{}T23:59:59Z", to))
                    .map_err(|_| {
                        ErpError::validation("to_date", "invalid format (use YYYY-MM-DD)")
                    })?
                    .with_timezone(&Utc)
            } else {
                Utc::now()
            };

            sales_service
                .get_orders_by_date_range(start_date, end_date)
                .await?
        } else {
            sales_service
                .list_orders(Some(limit as i64), Some(offset))
                .await?
        };

        let filtered_orders: Vec<_> = if let Some(status_str) = status {
            let status_filter = Self::parse_order_status(status_str)?;
            orders
                .into_iter()
                .filter(|o| o.status == status_filter)
                .collect()
        } else {
            orders
        };

        if filtered_orders.is_empty() {
            println!("No orders found.");
            return Ok(());
        }

        Self::display_orders_table(&filtered_orders);
        println!("\nTotal orders: {}", filtered_orders.len());

        Ok(())
    }

    async fn handle_update_order(
        sales_service: &SalesService,
        id: &str,
        status: &str,
        notes: &Option<String>,
    ) -> ErpResult<()> {
        let order_id =
            Uuid::from_str(id).map_err(|_| ErpError::validation("order_id", "invalid format"))?;

        let new_status = Self::parse_order_status(status)?;

        let update_request = UpdateOrderRequest {
            status: Some(new_status),
            payment_status: None,
            payment_method: None,
            shipping_address: None,
            billing_address: None,
            notes: notes.clone(),
        };

        match sales_service.update_order(order_id, update_request).await {
            Ok(updated_summary) => {
                println!("✅ Order updated successfully!");
                Self::display_order_summary(&updated_summary);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to update order: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_generate_invoice(
        sales_service: &SalesService,
        order_id: &str,
        output: &Option<String>,
        format: &str,
    ) -> ErpResult<()> {
        let order_uuid = Uuid::from_str(order_id)
            .map_err(|_| ErpError::validation("order_id", "invalid format"))?;

        match sales_service.generate_invoice(order_uuid).await {
            Ok(invoice) => {
                println!("✅ Invoice generated successfully!");
                Self::display_invoice(&invoice);

                if let Some(output_path) = output {
                    println!("\n📄 Invoice would be saved to: {}", output_path);
                    println!("📄 Format: {}", format);
                    println!("Note: File output implementation pending");
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to generate invoice: {}", e);
                Err(e)
            }
        }
    }

    fn parse_order_status(status_str: &str) -> ErpResult<OrderStatus> {
        match status_str.to_lowercase().as_str() {
            "draft" => Ok(OrderStatus::Draft),
            "pending" => Ok(OrderStatus::Pending),
            "confirmed" => Ok(OrderStatus::Confirmed),
            "processing" => Ok(OrderStatus::Processing),
            "shipped" => Ok(OrderStatus::Shipped),
            "delivered" => Ok(OrderStatus::Delivered),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "returned" => Ok(OrderStatus::Returned),
            _ => Err(ErpError::validation("status", format!(
                "Invalid status '{}'. Valid: draft, pending, confirmed, processing, shipped, delivered, cancelled, returned",
                status_str
            ))),
        }
    }

    fn display_order_summary(summary: &OrderSummary) {
        let mut table = Table::new();
        table.set_header(vec!["Field", "Value"]);

        table.add_row(vec!["Order Number", &summary.order.order_number]);
        table.add_row(vec!["Customer ID", &summary.order.customer_id.to_string()]);
        table.add_row(vec!["Status", &summary.order.status.to_string()]);
        table.add_row(vec![
            "Payment Status",
            &summary.order.payment_status.to_string(),
        ]);
        table.add_row(vec![
            "Order Date",
            &summary
                .order
                .order_date
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ]);
        table.add_row(vec!["Subtotal", &format!("${:.2}", summary.subtotal)]);
        table.add_row(vec![
            "Total Discount",
            &format!("${:.2}", summary.total_discount),
        ]);
        table.add_row(vec!["Tax Amount", &format!("${:.2}", summary.tax_amount)]);
        table.add_row(vec!["Grand Total", &format!("${:.2}", summary.grand_total)]);

        if let Some(notes) = &summary.order.notes {
            table.add_row(vec!["Notes", notes]);
        }

        println!("{}", table);

        if !summary.items.is_empty() {
            println!("\nOrder Items:");
            let mut items_table = Table::new();
            items_table.set_header(vec![
                "Product",
                "SKU",
                "Quantity",
                "Unit Price",
                "Discount",
                "Line Total",
            ]);

            for item_with_product in &summary.items {
                items_table.add_row(vec![
                    Cell::new(&item_with_product.product_name),
                    Cell::new(&item_with_product.product_sku),
                    Cell::new(&item_with_product.item.quantity.to_string()),
                    Cell::new(&format!("${:.2}", item_with_product.item.unit_price)),
                    Cell::new(&format!("${:.2}", item_with_product.item.discount)),
                    Cell::new(&format!("${:.2}", item_with_product.item.line_total)),
                ]);
            }

            println!("{}", items_table);
        }
    }

    fn display_orders_table(orders: &[SalesOrder]) {
        let mut table = Table::new();
        table.set_header(vec![
            "Order Number",
            "Customer ID",
            "Status",
            "Payment Status",
            "Total Amount",
            "Order Date",
        ]);

        for order in orders {
            let status_cell = match order.status {
                OrderStatus::Delivered => Cell::new(&order.status.to_string()).fg(Color::Green),
                OrderStatus::Cancelled | OrderStatus::Returned => {
                    Cell::new(&order.status.to_string()).fg(Color::Red)
                }
                OrderStatus::Processing | OrderStatus::Shipped => {
                    Cell::new(&order.status.to_string()).fg(Color::Yellow)
                }
                _ => Cell::new(&order.status.to_string()),
            };

            let payment_cell = match order.payment_status {
                PaymentStatus::Paid => {
                    Cell::new(&order.payment_status.to_string()).fg(Color::Green)
                }
                PaymentStatus::Failed | PaymentStatus::Overdue => {
                    Cell::new(&order.payment_status.to_string()).fg(Color::Red)
                }
                PaymentStatus::PartiallyPaid => {
                    Cell::new(&order.payment_status.to_string()).fg(Color::Yellow)
                }
                _ => Cell::new(&order.payment_status.to_string()),
            };

            table.add_row(vec![
                Cell::new(&order.order_number),
                Cell::new(&order.customer_id.to_string()[..8]),
                status_cell,
                payment_cell,
                Cell::new(&format!("${:.2}", order.total_amount)),
                Cell::new(&order.order_date.format("%Y-%m-%d").to_string()),
            ]);
        }

        println!("{}", table);
    }

    fn display_invoice(invoice: &Invoice) {
        println!("\n📄 INVOICE");
        println!("═══════════════════════════════════════");
        println!("Invoice Number: {}", invoice.invoice_number);
        println!("Issue Date: {}", invoice.issue_date.format("%Y-%m-%d"));
        println!("Due Date: {}", invoice.due_date.format("%Y-%m-%d"));
        println!();

        println!("Company Information:");
        println!("  Name: {}", invoice.company_info.name);
        println!("  Address: {}", invoice.company_info.address);
        println!("  Phone: {}", invoice.company_info.phone);
        println!("  Email: {}", invoice.company_info.email);
        if let Some(tax_id) = &invoice.company_info.tax_id {
            println!("  Tax ID: {}", tax_id);
        }
        println!();

        println!("Customer Information:");
        println!("  Name: {}", invoice.customer_info.name);
        if let Some(email) = &invoice.customer_info.email {
            println!("  Email: {}", email);
        }
        if let Some(phone) = &invoice.customer_info.phone {
            println!("  Phone: {}", phone);
        }
        if let Some(address) = &invoice.customer_info.billing_address {
            println!("  Billing Address: {}", address);
        }
        println!();

        Self::display_order_summary(&invoice.order_summary);
    }
}
