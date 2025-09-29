use crate::cli::parser::InventoryCommands;
use crate::cli::validator::CliValidator;
use crate::core::config::AppConfig;
use crate::core::database::connection::DatabaseManager;
use crate::core::database::models::product::StockStatus;
use crate::modules::inventory::{
    CreateInventoryItemRequest, InventoryFilter, InventoryModule, UpdateInventoryItemRequest,
};
use crate::utils::error::ErpResult;
use crate::utils::inventory_formatter::InventoryFormatter;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};
use tracing::{error, info};
use uuid::Uuid;

pub struct InventoryHandler;

impl InventoryHandler {
    pub async fn handle(cmd: &InventoryCommands, config: &AppConfig) -> ErpResult<()> {
        match cmd {
            InventoryCommands::Add {
                name,
                quantity,
                price,
                cost,
                category,
                sku,
                min_stock,
                description,
            } => {
                Self::handle_add(
                    name,
                    *quantity,
                    *price,
                    cost,
                    category,
                    sku,
                    min_stock,
                    description,
                    config,
                )
                .await
            }
            InventoryCommands::List {
                low_stock,
                category,
                search,
                page,
                limit,
                format,
                sort_by,
                order,
            } => {
                Self::handle_list(
                    *low_stock, category, search, *page, *limit, format, sort_by, order, config,
                )
                .await
            }
            InventoryCommands::Update {
                id,
                name,
                quantity,
                price,
                cost,
                category,
                description,
            } => Self::handle_update(id, name, quantity, price, cost, category, description).await,
            InventoryCommands::Remove { id, force } => Self::handle_remove(id, *force).await,
            InventoryCommands::LowStock { threshold, format } => {
                Self::handle_low_stock(threshold, format).await
            }
        }
    }

    async fn handle_add(
        name: &str,
        quantity: i32,
        price: f64,
        cost: &Option<f64>,
        category: &Option<String>,
        sku: &Option<String>,
        min_stock: &Option<i32>,
        description: &Option<String>,
        _config: &AppConfig,
    ) -> ErpResult<()> {
        info!("Adding new product: {}", name);

        // 입력 검증
        let validated_name = CliValidator::validate_product_name(name)?;
        let validated_quantity = CliValidator::validate_quantity(quantity)?;
        let validated_price = CliValidator::validate_price(price)?;

        let validated_cost = match cost {
            Some(c) => Some(CliValidator::validate_price(*c)?),
            None => None,
        };

        let validated_category = match category {
            Some(cat) => CliValidator::validate_category(cat)?,
            None => "general".to_string(), // 기본 카테고리
        };

        let validated_sku = match sku {
            Some(s) => Some(CliValidator::validate_sku(s)?),
            None => None,
        };

        let validated_min_stock = match min_stock {
            Some(stock) => Some(CliValidator::validate_quantity(*stock)?),
            None => Some(0), // Default minimum stock
        };

        // 요청 객체 생성
        let request = CreateInventoryItemRequest {
            name: validated_name.clone(),
            description: description.clone(),
            category: validated_category.clone(),
            price: validated_price,
            cost: validated_cost,
            quantity: validated_quantity,
            min_stock: validated_min_stock.unwrap(),
            max_stock: None,
            sku: validated_sku,
            is_taxable: Some(true),
            weight: None,
            dimensions: None,
            barcode: None,
            supplier_id: None,
            location: None,
        };

        // TODO: Get actual user_id from authentication context
        let _user_id = Uuid::new_v4();

        // 실제 데이터베이스 연결 사용
        let db_connection = DatabaseManager::get_connection().await?;
        let inventory_module = InventoryModule::new_with_postgres(db_connection.pool().clone());
        let user_id = Uuid::new_v4(); // TODO: Get from auth context
        let response = inventory_module
            .service()
            .create_product(request, user_id)
            .await;
        match response {
            Ok(product) => {
                println!("✅ 제품이 성공적으로 추가되었습니다!");
                println!();

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS);
                table.set_header(vec!["속성", "값"]);

                table.add_row(vec!["ID", &product.id.to_string()]);
                table.add_row(vec!["SKU", &product.sku]);
                table.add_row(vec!["제품명", &product.name]);
                table.add_row(vec!["카테고리", &product.category]);
                table.add_row(vec!["가격", &format!("₩{:.2}", product.price)]);
                table.add_row(vec!["원가", &format!("₩{:.2}", product.cost)]);
                table.add_row(vec!["수량", &product.quantity.to_string()]);
                table.add_row(vec!["최소 재고", &product.min_stock_level.to_string()]);
                table.add_row(vec!["재고 상태", &format!("{}", product.stock_status)]);
                table.add_row(vec![
                    "마진",
                    &format!("₩{:.2} ({:.1}%)", product.margin, product.margin_percentage),
                ]);

                if let Some(desc) = &product.description {
                    table.add_row(vec!["설명", desc]);
                }

                println!("{}", table);
                Ok(())
            }
            Err(e) => {
                error!("Failed to create product: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_list(
        low_stock: bool,
        category: &Option<String>,
        search: &Option<String>,
        page: u32,
        limit: u32,
        format: &str,
        sort_by: &str,
        order: &str,
        _config: &AppConfig,
    ) -> ErpResult<()> {
        info!(
            "Listing products - low_stock: {}, category: {:?}, search: {:?}, format: {}, sort_by: {}, order: {}",
            low_stock, category, search, format, sort_by, order
        );

        // 입력 검증
        let (validated_page, validated_limit) = CliValidator::validate_pagination(page, limit)?;

        let validated_category = match category {
            Some(cat) => Some(CliValidator::validate_category(cat)?),
            None => None,
        };

        // 필터 생성
        let _filter = InventoryFilter {
            category: validated_category.clone(),
            search_query: search.clone(),
            low_stock_only: if low_stock { Some(true) } else { None },
            page: Some(validated_page),
            limit: Some(validated_limit),
            sort_by: Some(sort_by.to_string()),
            sort_order: Some(order.to_string()),
            ..Default::default()
        };

        // 실제 데이터베이스 연결 사용
        let db_connection = DatabaseManager::get_connection().await?;
        let inventory_module = InventoryModule::new_with_postgres(db_connection.pool().clone());
        let response = inventory_module.service().list_products(_filter).await;
        match response {
            Ok(response) => {
                if response.items.is_empty() {
                    match format {
                        "json" => println!("{{\"items\": [], \"total\": 0}}"),
                        "csv" => println!("SKU,제품명,카테고리,가격,원가,총수량,사용가능수량,예약수량,최소재고,상태,재고상태,위치,마진율"),
                        "yaml" => println!("items: []\ntotal: 0"),
                        _ => println!("📋 조건에 맞는 제품이 없습니다."),
                    }
                    return Ok(());
                }

                match format {
                    "json" => {
                        let json_output = InventoryFormatter::to_json(&response)?;
                        println!("{}", json_output);
                    }
                    "csv" => {
                        let csv_output = InventoryFormatter::to_csv(&response)?;
                        println!("{}", csv_output);
                    }
                    "yaml" => {
                        let yaml_output = InventoryFormatter::to_yaml(&response)?;
                        println!("{}", yaml_output);
                    }
                    _ => {
                        // Default table format
                        println!(
                            "📋 제품 목록 ({} / {} 개)",
                            response.items.len(),
                            response.total
                        );
                        println!(
                            "   🔴 재고부족: {} | ❌ 품절: {} | 📄 페이지: {} ({}/페이지)",
                            response.low_stock_count,
                            response.out_of_stock_count,
                            validated_page,
                            validated_limit
                        );
                        println!();

                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS);
                        table.set_header(vec![
                            "SKU",
                            "제품명",
                            "카테고리",
                            "가격",
                            "원가",
                            "재고",
                            "상태",
                            "마진",
                        ]);

                        for item in &response.items {
                            let status_icon = match item.stock_status {
                                StockStatus::OutOfStock => "❌",
                                StockStatus::LowStock => "🔴",
                                StockStatus::InStock => "✅",
                                StockStatus::Overstocked => "📦",
                            };

                            let stock_info = if item.reserved_quantity > 0 {
                                format!("{} ({}/{})", item.quantity, item.available_quantity, item.reserved_quantity)
                            } else {
                                format!("{}", item.quantity)
                            };

                            table.add_row(vec![
                                &item.sku,
                                &item.name,
                                &item.category,
                                &format!("₩{:.2}", item.price),
                                &format!("₩{:.2}", item.cost),
                                &format!("{} {}", stock_info, status_icon),
                                &format!("{}", item.stock_status),
                                &format!("{:.1}%", item.margin_percentage),
                            ]);
                        }

                        println!("{}", table);

                        // 페이지네이션 정보
                        let total_pages =
                            (response.total + validated_limit as i64 - 1) / validated_limit as i64;
                        if total_pages > 1 {
                            println!();
                            println!(
                                "📖 페이지 {} / {} (전체 {} 개)",
                                validated_page, total_pages, response.total
                            );
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to list products: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_update(
        id: &str,
        name: &Option<String>,
        quantity: &Option<i32>,
        price: &Option<f64>,
        cost: &Option<f64>,
        category: &Option<String>,
        description: &Option<String>,
    ) -> ErpResult<()> {
        info!("Updating product: {}", id);

        // 입력 검증
        let _validated_id = CliValidator::validate_id_or_sku(id)?;

        let validated_name = match name {
            Some(n) => Some(CliValidator::validate_product_name(n)?),
            None => None,
        };

        let validated_quantity = match quantity {
            Some(q) => Some(CliValidator::validate_quantity(*q)?),
            None => None,
        };

        let validated_price = match price {
            Some(p) => Some(CliValidator::validate_price(*p)?),
            None => None,
        };

        let validated_cost = match cost {
            Some(c) => Some(CliValidator::validate_price(*c)?),
            None => None,
        };

        let validated_category = match category {
            Some(c) => Some(CliValidator::validate_category(c)?),
            None => None,
        };

        // 업데이트 요청 생성
        let request = UpdateInventoryItemRequest {
            name: validated_name.clone(),
            description: description.clone(),
            category: validated_category.clone(),
            price: validated_price,
            cost: validated_cost,
            quantity: validated_quantity,
            min_stock: None,
            max_stock: None,
            is_taxable: None,
            weight: None,
            dimensions: None,
            barcode: None,
            supplier_id: None,
            location: None,
        };

        // 업데이트할 내용이 있는지 확인
        if request.name.is_none()
            && request.category.is_none()
            && request.price.is_none()
            && request.cost.is_none()
            && request.quantity.is_none()
            && request.description.is_none()
        {
            println!("📝 업데이트할 내용이 없습니다.");
            return Ok(());
        }

        // TODO: Get actual user_id from authentication context
        let _user_id = Uuid::new_v4();

        // 실제 데이터베이스 연결 사용
        let db_connection = DatabaseManager::get_connection().await?;
        let inventory_module = InventoryModule::new_with_postgres(db_connection.pool().clone());
        let user_id = Uuid::new_v4(); // TODO: Get from auth context
        let response = inventory_module
            .service()
            .update_product(id, request, user_id)
            .await;
        match response {
            Ok(product) => {
                println!("✅ 제품이 성공적으로 수정되었습니다!");
                println!();

                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS);
                table.set_header(vec!["속성", "이전 → 새 값"]);

                if let Some(new_name) = validated_name {
                    table.add_row(vec!["제품명", &format!("→ {}", new_name)]);
                }
                if let Some(new_category) = validated_category {
                    table.add_row(vec!["카테고리", &format!("→ {}", new_category)]);
                }
                if validated_price.is_some() {
                    table.add_row(vec!["가격", &format!("→ ₩{:.2}", product.price)]);
                }
                if validated_cost.is_some() {
                    table.add_row(vec!["원가", &format!("→ ₩{:.2}", product.cost)]);
                }
                if validated_quantity.is_some() {
                    table.add_row(vec!["수량", &format!("→ {}", product.quantity)]);
                }
                if let Some(new_description) = description {
                    table.add_row(vec!["설명", &format!("→ {}", new_description)]);
                }

                table.add_row(vec!["SKU", &product.sku]);
                table.add_row(vec!["현재 수량", &product.quantity.to_string()]);
                table.add_row(vec!["재고 상태", &format!("{}", product.stock_status)]);

                println!("{}", table);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update product: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_remove(id: &str, force: bool) -> ErpResult<()> {
        info!("Removing product: {} (force: {})", id, force);

        // 입력 검증
        let validated_id = CliValidator::validate_id_or_sku(id)?;

        // 실제 데이터베이스 연결 사용
        let db_connection = DatabaseManager::get_connection().await?;
        let inventory_module = InventoryModule::new_with_postgres(db_connection.pool().clone());

        // 제품 정보 조회
        let product = match inventory_module.service().get_product(&validated_id).await {
            Ok(product) => product,
            Err(e) => {
                error!("Product not found: {}", validated_id);
                return Err(e);
            }
        };

        println!("🗑️  제품 삭제");
        println!("   SKU: {}", product.sku);
        println!("   제품명: {}", product.name);
        println!("   현재 수량: {}", product.quantity);
        println!();

        if !force {
            println!("⚠️  이 작업은 제품을 비활성화합니다. (실제 데이터는 유지됨)");
            println!("   완전 삭제를 원하면 --force 플래그를 사용하세요.");
            println!();

            // 실제 운영환경에서는 여기서 사용자 확인을 받아야 함
            // 현재는 자동으로 진행
        }

        // TODO: Get actual user_id from authentication context
        let _user_id = Uuid::new_v4();

        // 실제 삭제 수행
        let user_id = Uuid::new_v4(); // TODO: Get from auth context
        match inventory_module
            .service()
            .delete_product(&validated_id, force, user_id)
            .await
        {
            Ok(()) => {
                if force {
                    println!("✅ 제품이 완전히 삭제되었습니다.");
                } else {
                    println!("✅ 제품이 삭제되었습니다.");
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete product: {}", e);
                Err(e)
            }
        }
    }

    async fn handle_low_stock(threshold: &Option<i32>, format: &str) -> ErpResult<()> {
        info!("Getting low stock alerts with threshold: {:?}", threshold);

        let validated_threshold = match threshold {
            Some(t) => Some(CliValidator::validate_quantity(*t)?),
            None => None,
        };

        // Use actual inventory service instead of stubbed implementation
        let db_connection = DatabaseManager::get_connection().await?;
        let inventory_module = InventoryModule::new_with_postgres(db_connection.pool().clone());
        let alerts_result = inventory_module.service().get_low_stock_alerts(validated_threshold).await;

        match alerts_result {
            Ok(alerts) => {
                if alerts.is_empty() {
                    match format {
                        "json" => println!("[]"),
                        "csv" => println!("SKU,제품명,카테고리,현재수량,최소수량,부족수량"),
                        "yaml" => println!("[]"),
                        _ => println!("✅ 저재고 알림이 없습니다!"),
                    }
                    return Ok(());
                }

                match format {
                    "json" => {
                        let json_output = InventoryFormatter::low_stock_to_json(&alerts)?;
                        println!("{}", json_output);
                    }
                    "csv" => {
                        let csv_output = InventoryFormatter::low_stock_to_csv(&alerts)?;
                        println!("{}", csv_output);
                    }
                    "yaml" => match serde_yaml::to_string(&alerts) {
                        Ok(yaml) => println!("{}", yaml),
                        Err(e) => {
                            return Err(crate::utils::error::ErpError::internal(format!(
                                "YAML 변환 오류: {}",
                                e
                            )))
                        }
                    },
                    _ => {
                        let threshold_text = match validated_threshold {
                            Some(t) => format!("임계값 {} 이하", t),
                            None => "최소 재고 수준 이하".to_string(),
                        };

                        println!(
                            "🔴 저재고 알림 ({}) - {} 개 제품",
                            threshold_text,
                            alerts.len()
                        );
                        println!();

                        let mut table = Table::new();
                        table
                            .load_preset(UTF8_FULL)
                            .apply_modifier(UTF8_ROUND_CORNERS);
                        table.set_header(vec![
                            "SKU",
                            "제품명",
                            "카테고리",
                            "현재수량",
                            "최소수량",
                            "부족수량",
                        ]);

                        for alert in &alerts {
                            table.add_row(vec![
                                &alert.sku,
                                &alert.name,
                                &alert.category,
                                &alert.current_quantity.to_string(),
                                &alert.min_stock_level.to_string(),
                                &alert.shortfall.to_string(),
                            ]);
                        }

                        println!("{}", table);
                        println!();
                        println!("💡 재주문 권장: 부족 수량만큼 주문하시기 바랍니다.");
                    }
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to get low stock alerts: {}", e);
                Err(e)
            }
        }
    }
}
