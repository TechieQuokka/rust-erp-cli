use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::models::*;
use super::repository::SalesRepository;
use crate::modules::customers::service::CustomerService;
use crate::modules::inventory::service::InventoryService;
use crate::utils::error::{ErpError, ErpResult};
use crate::utils::validation::ValidationService;

pub struct SalesService {
    repository: Box<dyn SalesRepository>,
    _validation_service: ValidationService,
    customer_service: Option<CustomerService>,
    inventory_service: Option<Box<dyn InventoryService>>,
}

impl SalesService {
    pub fn new(
        repository: Box<dyn SalesRepository>,
        validation_service: ValidationService,
    ) -> Self {
        Self {
            repository,
            _validation_service: validation_service,
            customer_service: None,
            inventory_service: None,
        }
    }

    pub fn with_customer_service(mut self, customer_service: CustomerService) -> Self {
        self.customer_service = Some(customer_service);
        self
    }

    pub fn with_inventory_service(mut self, inventory_service: Box<dyn InventoryService>) -> Self {
        self.inventory_service = Some(inventory_service);
        self
    }

    pub async fn create_order(&self, request: CreateOrderRequest) -> ErpResult<OrderSummary> {
        self.validate_create_order_request(&request)?;

        if let Some(customer_service) = &self.customer_service {
            let _customer = customer_service
                .get_customer_by_id(request.customer_id)
                .await?;
        }

        let order_id = Uuid::new_v4();
        let order_number = self.repository.get_next_order_number().await?;
        let now = Utc::now();

        let mut order_items = Vec::new();
        let mut subtotal = Decimal::ZERO;
        let mut total_discount = Decimal::ZERO;

        for item_request in &request.items {
            let unit_price = if let Some(price) = item_request.unit_price {
                price
            } else if let Some(inventory_service) = &self.inventory_service {
                let product = inventory_service
                    .get_product(&item_request.product_id.to_string())
                    .await?;
                product.price
            } else {
                return Err(ErpError::validation("unit_price", "must be provided"));
            };

            if let Some(inventory_service) = &self.inventory_service {
                let product = inventory_service
                    .get_product(&item_request.product_id.to_string())
                    .await?;
                if product.quantity < item_request.quantity {
                    return Err(ErpError::validation(
                        "quantity",
                        format!(
                            "Insufficient inventory for product {}. Available: {}, Requested: {}",
                            item_request.product_id, product.quantity, item_request.quantity
                        ),
                    ));
                }
            }

            let item_discount = item_request.discount.unwrap_or(Decimal::ZERO);
            let line_total = (unit_price * Decimal::from(item_request.quantity)) - item_discount;

            let order_item = SalesOrderItem {
                id: Uuid::new_v4(),
                order_id,
                product_id: item_request.product_id,
                quantity: item_request.quantity,
                unit_price,
                discount: item_discount,
                line_total,
                created_at: now,
            };

            subtotal += line_total;
            total_discount += item_discount;
            order_items.push(order_item);
        }

        let order_discount = request.discount_amount.unwrap_or(Decimal::ZERO);
        let subtotal_after_discount = subtotal - order_discount;
        let tax_rate = Decimal::new(10, 2);
        let tax_amount = subtotal_after_discount * tax_rate / Decimal::from(100);
        let grand_total = subtotal_after_discount + tax_amount;

        let order = SalesOrder {
            id: order_id,
            order_number: order_number.clone(),
            customer_id: request.customer_id,
            order_date: now,
            status: OrderStatus::Draft,
            total_amount: grand_total,
            tax_amount,
            discount_amount: order_discount,
            shipping_address: request.shipping_address,
            billing_address: request.billing_address,
            payment_method: request.payment_method,
            payment_status: PaymentStatus::Pending,
            notes: request.notes,
            created_at: now,
            updated_at: now,
        };

        self.repository.create_order(&order).await?;
        self.repository.create_order_items(&order_items).await?;

        let items_with_products = self
            .repository
            .get_order_items_with_products(order_id)
            .await?;

        Ok(OrderSummary {
            order,
            items: items_with_products,
            subtotal,
            total_discount: total_discount + order_discount,
            tax_amount,
            grand_total,
        })
    }

    pub async fn get_order_by_id(&self, id: Uuid) -> ErpResult<Option<OrderSummary>> {
        let order = match self.repository.get_order_by_id(id).await? {
            Some(order) => order,
            None => return Ok(None),
        };

        let items = self.repository.get_order_items_with_products(id).await?;
        let (subtotal, total_discount, tax_amount) =
            self.repository.calculate_order_totals(id).await?;

        Ok(Some(OrderSummary {
            order,
            items,
            subtotal,
            total_discount,
            tax_amount,
            grand_total: subtotal - total_discount + tax_amount,
        }))
    }

    pub async fn get_order_by_number(&self, order_number: &str) -> ErpResult<Option<OrderSummary>> {
        let order = match self.repository.get_order_by_number(order_number).await? {
            Some(order) => order,
            None => return Ok(None),
        };

        let items = self
            .repository
            .get_order_items_with_products(order.id)
            .await?;
        let (subtotal, total_discount, tax_amount) =
            self.repository.calculate_order_totals(order.id).await?;

        Ok(Some(OrderSummary {
            order,
            items,
            subtotal,
            total_discount,
            tax_amount,
            grand_total: subtotal - total_discount + tax_amount,
        }))
    }

    pub async fn list_orders(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> ErpResult<Vec<SalesOrder>> {
        self.repository.list_orders(limit, offset).await
    }

    pub async fn search_orders(
        &self,
        query: &str,
        status: Option<OrderStatus>,
    ) -> ErpResult<Vec<SalesOrder>> {
        if query.trim().is_empty() {
            return Err(ErpError::validation("query", "cannot be empty"));
        }
        self.repository.search_orders(query, status).await
    }

    pub async fn update_order(
        &self,
        id: Uuid,
        updates: UpdateOrderRequest,
    ) -> ErpResult<OrderSummary> {
        let existing_order = self.repository.get_order_by_id(id).await?;
        if existing_order.is_none() {
            return Err(ErpError::not_found("Order", id.to_string()));
        }

        self.validate_update_order_request(&updates)?;
        self.repository.update_order(id, &updates).await?;

        let updated_order_summary = self.get_order_by_id(id).await?;
        match updated_order_summary {
            Some(summary) => Ok(summary),
            None => Err(ErpError::not_found(
                "Order",
                format!("with ID {} not found after update", id),
            )),
        }
    }

    pub async fn update_order_status(&self, id: Uuid, status: OrderStatus) -> ErpResult<()> {
        let existing_order = self.repository.get_order_by_id(id).await?;
        if existing_order.is_none() {
            return Err(ErpError::not_found("Order", id.to_string()));
        }

        if let Some(inventory_service) = &self.inventory_service {
            if status == OrderStatus::Confirmed {
                let items = self.repository.get_order_items(id).await?;
                for item in &items {
                    inventory_service
                        .adjust_stock(
                            &item.product_id.to_string(),
                            -item.quantity,
                            format!("Order {} confirmed", id),
                            Uuid::new_v4(),
                        )
                        .await?;
                }
            }
        }

        self.repository.update_order_status(id, status).await
    }

    pub async fn update_payment_status(
        &self,
        id: Uuid,
        payment_status: PaymentStatus,
    ) -> ErpResult<()> {
        let existing_order = self.repository.get_order_by_id(id).await?;
        if existing_order.is_none() {
            return Err(ErpError::not_found("Order", id.to_string()));
        }

        self.repository
            .update_payment_status(id, payment_status)
            .await
    }

    pub async fn cancel_order(&self, id: Uuid) -> ErpResult<()> {
        let existing_order = self.repository.get_order_by_id(id).await?;
        let order = match existing_order {
            Some(order) => order,
            None => return Err(ErpError::not_found("Order", id.to_string())),
        };

        if matches!(
            order.status,
            OrderStatus::Delivered | OrderStatus::Cancelled | OrderStatus::Returned
        ) {
            return Err(ErpError::validation(
                "status",
                format!("Cannot cancel order in status: {}", order.status),
            ));
        }

        if let Some(inventory_service) = &self.inventory_service {
            if order.status == OrderStatus::Confirmed || order.status == OrderStatus::Processing {
                let items = self.repository.get_order_items(id).await?;
                for item in &items {
                    inventory_service
                        .adjust_stock(
                            &item.product_id.to_string(),
                            item.quantity,
                            format!("Order {} cancelled", id),
                            Uuid::new_v4(),
                        )
                        .await?;
                }
            }
        }

        self.repository
            .update_order_status(id, OrderStatus::Cancelled)
            .await
    }

    pub async fn delete_order(&self, id: Uuid) -> ErpResult<()> {
        let existing_order = self.repository.get_order_by_id(id).await?;
        if existing_order.is_none() {
            return Err(ErpError::not_found("Order", id.to_string()));
        }

        self.repository.delete_order(id).await
    }

    pub async fn get_orders_by_customer(&self, customer_id: Uuid) -> ErpResult<Vec<SalesOrder>> {
        if let Some(customer_service) = &self.customer_service {
            let _customer = customer_service.get_customer_by_id(customer_id).await?;
        }

        self.repository.get_orders_by_customer(customer_id).await
    }

    pub async fn get_orders_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> ErpResult<Vec<SalesOrder>> {
        if start_date > end_date {
            return Err(ErpError::validation(
                "date_range",
                "Start date must be before end date",
            ));
        }

        self.repository
            .get_orders_by_date_range(start_date, end_date)
            .await
    }

    pub async fn generate_invoice(&self, order_id: Uuid) -> ErpResult<Invoice> {
        let order_summary = self.get_order_by_id(order_id).await?;
        let order_summary = match order_summary {
            Some(summary) => summary,
            None => return Err(ErpError::not_found("Order", order_id.to_string())),
        };

        if order_summary.order.status == OrderStatus::Draft {
            return Err(ErpError::validation(
                "order_status",
                "Cannot generate invoice for draft orders",
            ));
        }

        let customer_info = if let Some(customer_service) = &self.customer_service {
            let customer = customer_service
                .get_customer_by_id(order_summary.order.customer_id)
                .await?;
            CustomerInfo {
                name: format!("{} {}", customer.first_name, customer.last_name),
                email: Some(customer.email),
                phone: customer.phone,
                billing_address: order_summary.order.billing_address.clone(),
            }
        } else {
            CustomerInfo {
                name: "Unknown Customer".to_string(),
                email: None,
                phone: None,
                billing_address: order_summary.order.billing_address.clone(),
            }
        };

        let company_info = CompanyInfo {
            name: "Your Company Name".to_string(),
            address: "123 Business St, City, Country".to_string(),
            phone: "+1-234-567-8900".to_string(),
            email: "billing@yourcompany.com".to_string(),
            tax_id: Some("TAX123456789".to_string()),
        };

        let invoice_number = format!("INV-{}", order_summary.order.order_number);
        let issue_date = Utc::now();
        let due_date = issue_date + chrono::Duration::days(30);

        Ok(Invoice {
            order_id,
            invoice_number,
            issue_date,
            due_date,
            order_summary,
            customer_info,
            company_info,
        })
    }

    pub async fn get_sales_statistics(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> ErpResult<SalesStatistics> {
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if start > end {
                return Err(ErpError::validation(
                    "date_range",
                    "Start date must be before end date",
                ));
            }
        }

        self.repository
            .get_sales_statistics(start_date, end_date)
            .await
    }

    fn validate_create_order_request(&self, request: &CreateOrderRequest) -> ErpResult<()> {
        if request.items.is_empty() {
            return Err(ErpError::validation(
                "items",
                "Order must contain at least one item",
            ));
        }

        for (index, item) in request.items.iter().enumerate() {
            if item.quantity <= 0 {
                return Err(ErpError::validation(
                    "quantity",
                    format!("Item {} quantity must be positive", index + 1),
                ));
            }

            if let Some(price) = item.unit_price {
                if price < Decimal::ZERO {
                    return Err(ErpError::validation(
                        "price",
                        format!("Item {} price cannot be negative", index + 1),
                    ));
                }
            }

            if let Some(discount) = item.discount {
                if discount < Decimal::ZERO {
                    return Err(ErpError::validation(
                        "discount",
                        format!("Item {} discount cannot be negative", index + 1),
                    ));
                }
            }
        }

        if let Some(discount) = request.discount_amount {
            if discount < Decimal::ZERO {
                return Err(ErpError::validation(
                    "discount_amount",
                    "cannot be negative",
                ));
            }
        }

        Ok(())
    }

    fn validate_update_order_request(&self, request: &UpdateOrderRequest) -> ErpResult<()> {
        if let Some(shipping_address) = &request.shipping_address {
            if shipping_address.trim().is_empty() {
                return Err(ErpError::validation("shipping_address", "cannot be empty"));
            }
        }

        if let Some(billing_address) = &request.billing_address {
            if billing_address.trim().is_empty() {
                return Err(ErpError::validation("billing_address", "cannot be empty"));
            }
        }

        Ok(())
    }

    pub fn get_order_status_options() -> Vec<OrderStatus> {
        vec![
            OrderStatus::Draft,
            OrderStatus::Pending,
            OrderStatus::Confirmed,
            OrderStatus::Processing,
            OrderStatus::Shipped,
            OrderStatus::Delivered,
            OrderStatus::Cancelled,
            OrderStatus::Returned,
        ]
    }

    pub fn get_payment_status_options() -> Vec<PaymentStatus> {
        vec![
            PaymentStatus::Pending,
            PaymentStatus::Paid,
            PaymentStatus::PartiallyPaid,
            PaymentStatus::Overdue,
            PaymentStatus::Failed,
            PaymentStatus::Refunded,
        ]
    }

    pub fn get_payment_method_options() -> Vec<PaymentMethod> {
        vec![
            PaymentMethod::Cash,
            PaymentMethod::CreditCard,
            PaymentMethod::DebitCard,
            PaymentMethod::BankTransfer,
            PaymentMethod::Check,
            PaymentMethod::PayPal,
            PaymentMethod::Crypto,
        ]
    }
}
