use anyhow::Result;

mod device_info;
mod scalar_collector;
mod table_collector;
mod types;

use device_info::DeviceInfo;
use scalar_collector::ScalarCollector;
use table_collector::TableCollector;
pub use types::{MonitoringResult, ScalarResult, TableResult};

use crate::config::AppConfig;
use crate::snmp::SnmpClient;

/// Основной коллектор для сбора данных
pub struct SnmpCollector;

impl SnmpCollector {
    /// Собирает все данные с устройства
    // TODO: Добавить pre-flight проверки (доступность устройства, поддерживаемые версии SNMP)
    // TODO: Добавить сбор системных метрик (время выполнения, количество запросов, etc...)
    pub async fn collect_all(
        mut client: SnmpClient,
        config: &AppConfig,
        client_type: &str,
    ) -> Result<MonitoringResult> {
        DeviceInfo::detect_and_set_device_type(&mut client).await;

        let scalars = ScalarCollector::collect_scalars(&mut client, config).await;

        // Условный сбор таблиц
        let tables = if config.settings.should_collect_tables() {
            Some(TableCollector::collect_tables(&mut client, config).await)
        } else {
            None
        };

        Ok(MonitoringResult {
            client_type: client_type.to_string(),
            scalars,
            tables,
        })
    }
}
