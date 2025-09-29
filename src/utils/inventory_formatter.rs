use crate::modules::inventory::InventoryListResponse;
use crate::utils::error::{ErpError, ErpResult};
use serde_json;

/// 재고 데이터를 다양한 형식으로 출력하는 포맷터
pub struct InventoryFormatter;

impl InventoryFormatter {
    /// 재고 목록을 JSON 형식으로 변환
    pub fn to_json(response: &InventoryListResponse) -> ErpResult<String> {
        match serde_json::to_string_pretty(response) {
            Ok(json) => Ok(json),
            Err(e) => Err(ErpError::internal(format!("JSON 변환 오류: {}", e))),
        }
    }

    /// 재고 목록을 CSV 형식으로 변환
    pub fn to_csv(response: &InventoryListResponse) -> ErpResult<String> {
        let mut csv = String::new();

        // 헤더
        csv.push_str("SKU,제품명,카테고리,가격,원가,총수량,사용가능수량,예약수량,최소재고,상태,재고상태,위치,마진율\n");

        // 데이터 행들
        for item in &response.items {
            let margin_percentage = if item.price > item.cost {
                ((item.price - item.cost) / item.price * rust_decimal::Decimal::from(100))
                    .round_dp(1)
            } else {
                rust_decimal::Decimal::ZERO
            };

            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",{},{},{},{},{},{},\"{}\",\"{}\",\"{}\",{}%\n",
                item.sku.replace('"', "\"\""),
                item.name.replace('"', "\"\""),
                item.category.replace('"', "\"\""),
                item.price,
                item.cost,
                item.quantity,
                item.available_quantity,
                item.reserved_quantity,
                item.min_stock_level,
                item.status,
                item.stock_status,
                item.location.as_deref().unwrap_or(""),
                margin_percentage
            ));
        }

        Ok(csv)
    }

    /// 재고 목록을 YAML 형식으로 변환
    pub fn to_yaml(response: &InventoryListResponse) -> ErpResult<String> {
        match serde_yaml::to_string(response) {
            Ok(yaml) => Ok(yaml),
            Err(e) => Err(ErpError::internal(format!("YAML 변환 오류: {}", e))),
        }
    }

    /// 저재고 알림을 JSON 형식으로 변환
    pub fn low_stock_to_json(
        alerts: &[crate::modules::inventory::LowStockAlert],
    ) -> ErpResult<String> {
        match serde_json::to_string_pretty(alerts) {
            Ok(json) => Ok(json),
            Err(e) => Err(ErpError::internal(format!("JSON 변환 오류: {}", e))),
        }
    }

    /// 저재고 알림을 CSV 형식으로 변환
    pub fn low_stock_to_csv(
        alerts: &[crate::modules::inventory::LowStockAlert],
    ) -> ErpResult<String> {
        let mut csv = String::new();

        // 헤더
        csv.push_str("SKU,제품명,카테고리,현재수량,최소수량,부족수량\n");

        // 데이터 행들
        for alert in alerts {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",{},{},{}\n",
                alert.sku.replace('"', "\"\""),
                alert.name.replace('"', "\"\""),
                alert.category.replace('"', "\"\""),
                alert.current_quantity,
                alert.min_stock_level,
                alert.shortfall
            ));
        }

        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::models::product::{ProductStatus, StockStatus};
    use crate::modules::inventory::InventoryItemResponse;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn create_test_response() -> InventoryListResponse {
        InventoryListResponse {
            items: vec![InventoryItemResponse {
                id: Uuid::new_v4(),
                sku: "TEST001".to_string(),
                name: "Test Product".to_string(),
                description: Some("Test description".to_string()),
                category: "Electronics".to_string(),
                price: Decimal::new(1999, 2), // 19.99
                cost: Decimal::new(1299, 2),  // 12.99
                quantity: 100,
                available_quantity: 95,
                reserved_quantity: 5,
                min_stock_level: 10,
                max_stock_level: Some(500),
                status: ProductStatus::Active,
                stock_status: StockStatus::InStock,
                location: Some("A1".to_string()),
                last_movement_date: Some(Utc::now()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                margin: Decimal::new(700, 2),             // 7.00
                margin_percentage: Decimal::new(3503, 2), // 35.03%
                reorder_needed: false,
                days_of_stock: Some(95),
            }],
            total: 1,
            page: 1,
            per_page: 20,
            low_stock_count: 0,
            out_of_stock_count: 0,
        }
    }

    #[test]
    fn test_to_json() {
        let response = create_test_response();
        let json = InventoryFormatter::to_json(&response);
        assert!(json.is_ok());
        let json_content = json.unwrap();
        assert!(json_content.contains("TEST001"));
        assert!(json_content.contains("Test Product"));
    }

    #[test]
    fn test_to_csv() {
        let response = create_test_response();
        let csv = InventoryFormatter::to_csv(&response);
        assert!(csv.is_ok());
        let csv_content = csv.unwrap();
        assert!(csv_content.contains("SKU,제품명,카테고리"));
        assert!(csv_content.contains("TEST001"));
    }

    #[test]
    fn test_to_yaml() {
        let response = create_test_response();
        let yaml = InventoryFormatter::to_yaml(&response);
        assert!(yaml.is_ok());
        let yaml_content = yaml.unwrap();
        assert!(yaml_content.contains("sku: TEST001"));
    }
}
