use tokio::time::{Duration, timeout};

use super::types::ScalarResult;
use crate::config::AppConfig;
use crate::snmp::{SnmpClient, parse_oid};

/// Модуль для сбора скалярных SNMP значений
pub struct ScalarCollector;

impl ScalarCollector {
    /// Собирает все скалярные значения из конфигурации
    pub async fn collect_scalars(client: &mut SnmpClient, config: &AppConfig) -> Vec<ScalarResult> {
        let mut results = Vec::new();

        for (name, oid_str) in &config.profile.scalars {
            let result = Self::collect_single_scalar(client, name, oid_str).await;
            results.push(result);
        }

        results
    }

    /// Собирает одно скалярное значение
    async fn collect_single_scalar(
        client: &mut SnmpClient,
        name: &str,
        oid_str: &str,
    ) -> ScalarResult {
        match parse_oid(oid_str) {
            Ok(oid) => {
                let timeout_duration = Duration::from_secs(3);
                match timeout(timeout_duration, client.get(&oid)).await {
                    Ok(Ok(value)) => ScalarResult {
                        name: name.to_string(),
                        oid: oid_str.to_string(),
                        value: Some(format!("{:?}", value)),
                        error: None,
                    },
                    Ok(Err(e)) => ScalarResult {
                        name: name.to_string(),
                        oid: oid_str.to_string(),
                        value: None,
                        error: Some(format!("SNMP ERROR: {}", e)),
                    },
                    Err(_) => ScalarResult {
                        name: name.to_string(),
                        oid: oid_str.to_string(),
                        value: None,
                        error: Some("TIMEOUT".to_string()),
                    },
                }
            }
            Err(e) => ScalarResult {
                name: name.to_string(),
                oid: oid_str.to_string(),
                value: None,
                error: Some(format!("OID PARSE ERROR: {}", e)),
            },
        }
    }
}
