use crate::modules::reports::{InventoryStatusReport, SalesSummaryReport};
use crate::utils::error::{ErpError, ErpResult};
use serde::Serialize;
use std::fs;

/// 보고서 출력 형식 처리 유틸리티
pub struct OutputFormatter;

impl OutputFormatter {
    /// 보고서를 XML 형식으로 변환
    pub fn to_xml<T: Serialize>(data: &T) -> ErpResult<String> {
        // XML 래퍼 구조체를 만들어 루트 요소를 정의
        #[derive(Serialize)]
        struct XmlWrapper<T> {
            report: T,
        }

        let wrapper = XmlWrapper { report: data };

        match quick_xml::se::to_string(&wrapper) {
            Ok(xml) => Ok(format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}",
                xml
            )),
            Err(e) => Err(ErpError::internal(format!("XML 변환 오류: {}", e))),
        }
    }

    /// 보고서를 YAML 형식으로 변환
    pub fn to_yaml<T: Serialize>(data: &T) -> ErpResult<String> {
        match serde_yaml::to_string(data) {
            Ok(yaml) => Ok(yaml),
            Err(e) => Err(ErpError::internal(format!("YAML 변환 오류: {}", e))),
        }
    }

    /// 보고서를 CSV 형식으로 변환 (매출 요약용)
    pub fn sales_summary_to_csv(report: &SalesSummaryReport) -> ErpResult<String> {
        let mut csv = String::new();

        // 헤더 정보
        csv.push_str("Report Type,Sales Summary\n");
        csv.push_str(&format!(
            "Generated At,{}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        csv.push_str(&format!("Period,{:?}\n", report.period));
        csv.push('\n');

        // 요약 정보
        csv.push_str("Summary Information\n");
        csv.push_str("Metric,Value\n");
        csv.push_str(&format!("Total Orders,{}\n", report.total_orders));
        csv.push_str(&format!("Total Revenue,{}\n", report.total_revenue));
        csv.push_str(&format!("Total Items Sold,{}\n", report.total_items_sold));
        csv.push_str(&format!(
            "Average Order Value,{}\n",
            report.average_order_value
        ));
        csv.push('\n');

        // 상위 판매 제품
        if !report.top_selling_products.is_empty() {
            csv.push_str("Top Selling Products\n");
            csv.push_str("Product Name,SKU,Quantity Sold,Total Revenue\n");
            for product in &report.top_selling_products {
                csv.push_str(&format!(
                    "\"{}\",{},{},{}\n",
                    product.name.replace('"', "\"\""),
                    product.sku,
                    product.quantity_sold,
                    product.total_revenue
                ));
            }
            csv.push('\n');
        }

        // 상태별 매출
        if !report.sales_by_status.is_empty() {
            csv.push_str("Sales By Status\n");
            csv.push_str("Status,Order Count,Total Amount\n");
            for status in &report.sales_by_status {
                csv.push_str(&format!(
                    "\"{}\",{},{}\n",
                    status.status.replace('"', "\"\""),
                    status.order_count,
                    status.total_amount
                ));
            }
        }

        Ok(csv)
    }

    /// 보고서를 CSV 형식으로 변환 (재고 상태용)
    pub fn inventory_status_to_csv(report: &InventoryStatusReport) -> ErpResult<String> {
        let mut csv = String::new();

        // 헤더 정보
        csv.push_str("Report Type,Inventory Status\n");
        csv.push_str(&format!(
            "Generated At,{}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        csv.push('\n');

        // 요약 정보
        csv.push_str("Summary Information\n");
        csv.push_str("Metric,Value\n");
        csv.push_str(&format!("Total Products,{}\n", report.total_products));
        csv.push_str(&format!("Total Stock Value,{}\n", report.total_stock_value));
        csv.push_str(&format!(
            "Low Stock Items,{}\n",
            report.low_stock_items.len()
        ));
        csv.push_str(&format!(
            "Out of Stock Items,{}\n",
            report.out_of_stock_items.len()
        ));
        csv.push('\n');

        // 저재고 아이템
        if !report.low_stock_items.is_empty() {
            csv.push_str("Low Stock Items\n");
            csv.push_str("Product Name,SKU,Current Stock,Reorder Level,Suggested Reorder Quantity,Stock Value\n");
            for item in &report.low_stock_items {
                csv.push_str(&format!(
                    "\"{}\",{},{},{},{},{}\n",
                    item.name.replace('"', "\"\""),
                    item.sku,
                    item.current_stock,
                    item.reorder_level,
                    item.suggested_reorder_quantity,
                    item.stock_value
                ));
            }
            csv.push('\n');
        }

        // 품절 아이템
        if !report.out_of_stock_items.is_empty() {
            csv.push_str("Out of Stock Items\n");
            csv.push_str("Product Name,SKU,Last Stock Date,Pending Orders\n");
            for item in &report.out_of_stock_items {
                csv.push_str(&format!(
                    "\"{}\",{},{},{}\n",
                    item.name.replace('"', "\"\""),
                    item.sku,
                    item.last_stock_date
                        .map_or("N/A".to_string(), |d| d.to_string()),
                    item.pending_orders
                ));
            }
            csv.push('\n');
        }

        // 카테고리별 재고
        if !report.inventory_by_category.is_empty() {
            csv.push_str("Inventory By Category\n");
            csv.push_str(
                "Category,Product Count,Total Stock,Total Value,Average Stock Per Product\n",
            );
            for category in &report.inventory_by_category {
                csv.push_str(&format!(
                    "\"{}\",{},{},{},{}\n",
                    category.category.replace('"', "\"\""),
                    category.product_count,
                    category.total_stock,
                    category.total_value,
                    category.average_stock_per_product
                ));
            }
        }

        Ok(csv)
    }

    /// 보고서를 HTML 형식으로 변환 (매출 요약용)
    pub fn sales_summary_to_html(report: &SalesSummaryReport) -> ErpResult<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"ko\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        html.push_str("    <title>매출 요약 보고서</title>\n");
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str(
            "        table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n",
        );
        html.push_str(
            "        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n",
        );
        html.push_str("        th { background-color: #f2f2f2; }\n");
        html.push_str(
            "        .header { background-color: #4CAF50; color: white; padding: 10px; }\n",
        );
        html.push_str(
            "        .summary { background-color: #f9f9f9; padding: 10px; margin: 10px 0; }\n",
        );
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // 헤더
        html.push_str("    <div class=\"header\">\n");
        html.push_str("        <h1>매출 요약 보고서</h1>\n");
        html.push_str(&format!(
            "        <p>생성 시간: {}</p>\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        html.push_str(&format!("        <p>기간: {:?}</p>\n", report.period));
        html.push_str("    </div>\n");

        // 요약 정보
        html.push_str("    <div class=\"summary\">\n");
        html.push_str("        <h2>요약 정보</h2>\n");
        html.push_str("        <table>\n");
        html.push_str("            <tr><th>항목</th><th>값</th></tr>\n");
        html.push_str(&format!(
            "            <tr><td>총 주문 수</td><td>{}</td></tr>\n",
            report.total_orders
        ));
        html.push_str(&format!(
            "            <tr><td>총 매출</td><td>₩{}</td></tr>\n",
            report.total_revenue
        ));
        html.push_str(&format!(
            "            <tr><td>총 판매 수량</td><td>{}</td></tr>\n",
            report.total_items_sold
        ));
        html.push_str(&format!(
            "            <tr><td>평균 주문 금액</td><td>₩{}</td></tr>\n",
            report.average_order_value
        ));
        html.push_str("        </table>\n");
        html.push_str("    </div>\n");

        // 상위 판매 제품
        if !report.top_selling_products.is_empty() {
            html.push_str("    <h2>상위 판매 제품</h2>\n");
            html.push_str("    <table>\n");
            html.push_str(
                "        <tr><th>제품명</th><th>SKU</th><th>판매량</th><th>매출</th></tr>\n",
            );
            for product in &report.top_selling_products {
                html.push_str(&format!(
                    "        <tr><td>{}</td><td>{}</td><td>{}</td><td>₩{}</td></tr>\n",
                    product.name, product.sku, product.quantity_sold, product.total_revenue
                ));
            }
            html.push_str("    </table>\n");
        }

        // 주문 상태별 매출
        if !report.sales_by_status.is_empty() {
            html.push_str("    <h2>주문 상태별 매출</h2>\n");
            html.push_str("    <table>\n");
            html.push_str("        <tr><th>상태</th><th>주문 수</th><th>총 금액</th></tr>\n");
            for status in &report.sales_by_status {
                html.push_str(&format!(
                    "        <tr><td>{}</td><td>{}</td><td>₩{}</td></tr>\n",
                    status.status, status.order_count, status.total_amount
                ));
            }
            html.push_str("    </table>\n");
        }

        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    /// 파일에 보고서 저장
    pub fn save_to_file(content: &str, output_path: &str) -> ErpResult<()> {
        match fs::write(output_path, content) {
            Ok(_) => Ok(()),
            Err(e) => Err(ErpError::internal(format!("파일 저장 오류: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::reports::{ReportPeriod, SalesSummaryReport};
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[test]
    fn test_to_yaml() {
        let report = SalesSummaryReport {
            period: ReportPeriod::Monthly,
            generated_at: Utc::now(),
            total_orders: 100,
            total_revenue: Decimal::new(50000, 2),
            total_items_sold: 300,
            average_order_value: Decimal::new(50000, 4),
            top_selling_products: vec![],
            sales_by_status: vec![],
            daily_sales: vec![],
        };

        let yaml = OutputFormatter::to_yaml(&report);
        assert!(yaml.is_ok());
        assert!(yaml.unwrap().contains("total_orders: 100"));
    }

    #[test]
    fn test_to_xml() {
        let report = SalesSummaryReport {
            period: ReportPeriod::Monthly,
            generated_at: Utc::now(),
            total_orders: 100,
            total_revenue: Decimal::new(50000, 2),
            total_items_sold: 300,
            average_order_value: Decimal::new(50000, 4),
            top_selling_products: vec![],
            sales_by_status: vec![],
            daily_sales: vec![],
        };

        let xml = OutputFormatter::to_xml(&report);
        assert!(xml.is_ok());
        let xml_content = xml.unwrap();
        assert!(xml_content.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml_content.contains("<total_orders>100</total_orders>"));
    }

    #[test]
    fn test_sales_summary_to_csv() {
        let report = SalesSummaryReport {
            period: ReportPeriod::Monthly,
            generated_at: Utc::now(),
            total_orders: 100,
            total_revenue: Decimal::new(50000, 2),
            total_items_sold: 300,
            average_order_value: Decimal::new(50000, 4),
            top_selling_products: vec![],
            sales_by_status: vec![],
            daily_sales: vec![],
        };

        let csv = OutputFormatter::sales_summary_to_csv(&report);
        assert!(csv.is_ok());
        let csv_content = csv.unwrap();
        assert!(csv_content.contains("Report Type,Sales Summary"));
        assert!(csv_content.contains("Total Orders,100"));
    }

    #[test]
    fn test_sales_summary_to_html() {
        let report = SalesSummaryReport {
            period: ReportPeriod::Monthly,
            generated_at: Utc::now(),
            total_orders: 100,
            total_revenue: Decimal::new(50000, 2),
            total_items_sold: 300,
            average_order_value: Decimal::new(50000, 4),
            top_selling_products: vec![],
            sales_by_status: vec![],
            daily_sales: vec![],
        };

        let html = OutputFormatter::sales_summary_to_html(&report);
        assert!(html.is_ok());
        let html_content = html.unwrap();
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("매출 요약 보고서"));
        assert!(html_content.contains("총 주문 수"));
    }
}
