use tokio::time::{Duration, timeout};

use super::types::TableResult;
use crate::config::AppConfig;
use crate::snmp::{SnmpClient, parse_oid};

/// Модуль для сбора табличных SNMP данных
pub struct TableCollector;

impl TableCollector {
    /// Собирает все таблицы из конфигурации
    pub async fn collect_tables(client: &mut SnmpClient, config: &AppConfig) -> Vec<TableResult> {
        let mut results = Vec::new();

        for (table_name, table_oid) in &config.profile.tables {
            // Для ifTable берем больше записей
            let max_items = if table_name == "ifTable" { 50 } else { 20 };

            let result =
                Self::collect_single_table(client, table_name, table_oid, config, Some(max_items))
                    .await;

            results.push(result);
        }

        results
    }

    /// Собирает данные из одной таблицы
    pub async fn collect_single_table(
        client: &mut SnmpClient,
        table_name: &str,
        table_oid: &str,
        config: &AppConfig,
        max_items: Option<usize>,
    ) -> TableResult {
        match parse_oid(table_oid) {
            Ok(root_oid) => {
                let timeout_duration = Duration::from_secs(config.get_timeout());
                let limit = max_items.unwrap_or(50);

                match timeout(timeout_duration, client.walk_limited(&root_oid, limit)).await {
                    Ok(Ok(rows)) => {
                        let formatted_rows: Vec<(String, String)> = rows
                            .into_iter()
                            .map(|(oid, value)| (oid.to_string(), value))
                            .collect();

                        TableResult {
                            name: table_name.to_string(),
                            oid: table_oid.to_string(),
                            rows: formatted_rows,
                            error: None,
                            limited_to: Some(limit),
                        }
                    }
                    Ok(Err(e)) => TableResult {
                        name: table_name.to_string(),
                        oid: table_oid.to_string(),
                        rows: Vec::new(),
                        error: Some(format!("SNMP ERROR: {}", e)),
                        limited_to: Some(limit),
                    },
                    Err(_) => TableResult {
                        name: table_name.to_string(),
                        oid: table_oid.to_string(),
                        rows: Vec::new(),
                        error: Some("TIMEOUT".to_string()),
                        limited_to: Some(limit),
                    },
                }
            }
            Err(e) => TableResult {
                name: table_name.to_string(),
                oid: table_oid.to_string(),
                rows: Vec::new(),
                error: Some(format!("OID PARSE ERROR: {}", e)),
                limited_to: max_items,
            },
        }
    }
}
