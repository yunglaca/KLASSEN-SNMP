use anyhow::Result;

use tokio::time::{Duration, timeout};

use crate::config::AppConfig;
use crate::snmp::{DeviceDetector, SnmpClient, parse_oid, set_global_device_type};

/// Результат сбора скалярных значений
#[derive(Debug, Clone)]
pub struct ScalarResult {
    pub name: String,
    pub oid: String,
    pub value: Option<String>,
    pub error: Option<String>,
}

/// Результат сбора таблицы
#[derive(Debug, Clone)]
pub struct TableResult {
    pub name: String,
    pub oid: String,
    pub rows: Vec<(String, String)>, // (OID, value)
    pub error: Option<String>,
    pub limited_to: Option<usize>,
}

/// Полный результат мониторинга устройства
#[derive(Debug, Clone)]
pub struct MonitoringResult {
    pub client_type: String,
    pub scalars: Vec<ScalarResult>,
    pub tables: Vec<TableResult>,
}

/// Коллектор для сбора SNMP данных
pub struct SnmpCollector;

impl SnmpCollector {
    /// Собирает все скалярные значения из конфигурации
    pub async fn collect_scalars(client: &mut SnmpClient, config: &AppConfig) -> Vec<ScalarResult> {
        let mut results = Vec::new();

        for (name, oid_str) in &config.profile.scalars {
            let result = match parse_oid(oid_str) {
                Ok(oid) => {
                    let timeout_duration = Duration::from_secs(3);
                    match timeout(timeout_duration, client.get(&oid)).await {
                        Ok(Ok(value)) => ScalarResult {
                            name: name.clone(),
                            oid: oid_str.clone(),
                            value: Some(format!("{:?}", value)),
                            error: None,
                        },
                        Ok(Err(e)) => ScalarResult {
                            name: name.clone(),
                            oid: oid_str.clone(),
                            value: None,
                            error: Some(format!("SNMP ERROR: {}", e)),
                        },
                        Err(_) => ScalarResult {
                            name: name.clone(),
                            oid: oid_str.clone(),
                            value: None,
                            error: Some("TIMEOUT".to_string()),
                        },
                    }
                }
                Err(e) => ScalarResult {
                    name: name.clone(),
                    oid: oid_str.clone(),
                    value: None,
                    error: Some(format!("OID PARSE ERROR: {}", e)),
                },
            };

            results.push(result);
        }

        results
    }

    /// Собирает данные из одной таблицы
    pub async fn collect_table(
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

    /// Собирает все таблицы из конфигурации
    pub async fn collect_tables(client: &mut SnmpClient, config: &AppConfig) -> Vec<TableResult> {
        let mut results = Vec::new();

        for (table_name, table_oid) in &config.profile.tables {
            // Для ifTable берем больше записей
            let max_items = if table_name == "ifTable" { 50 } else { 20 };

            let result =
                Self::collect_table(client, table_name, table_oid, config, Some(max_items)).await;

            results.push(result);
        }

        results
    }

    /// Собирает все данные с устройства
    // TODO: Добавить pre-flight проверки (доступность устройства, поддерживаемые версии SNMP)
    // TODO: Добавить сбор системных метрик (время выполнения, количество запросов, etc...)
    pub async fn collect_all(
        mut client: SnmpClient,
        config: &AppConfig,
        client_type: &str,
    ) -> Result<MonitoringResult> {
        // Определяем тип устройства
        if let Ok(sys_object_id) = Self::get_sys_object_id(&mut client).await {
            let device_info = DeviceDetector::detect_device_type(&sys_object_id);
            // Устанавливаем глобальный тип устройства для правильного разрешения OID
            set_global_device_type(device_info.device_type);
        } else {
            set_global_device_type("generic".to_string());
        }

        // Собираем скаляры последовательно
        // TODO: Для реальной асинхронности нужно создать отдельные клиенты
        // или использовать клиент, поддерживающий множественные запросы
        let scalars = Self::collect_scalars(&mut client, config).await;

        // Собираем таблицы последовательно
        // TODO: Внутри каждого walk'а уже есть асинхронность I/O
        let tables = Self::collect_tables(&mut client, config).await;

        Ok(MonitoringResult {
            client_type: client_type.to_string(),
            scalars,
            tables,
        })
    }

    /// Получает sysObjectID устройства
    async fn get_sys_object_id(client: &mut SnmpClient) -> Result<String> {
        let sys_object_id_oid = parse_oid("1.3.6.1.2.1.1.2.0")?;
        let timeout_duration = Duration::from_secs(3);

        match timeout(timeout_duration, client.get(&sys_object_id_oid)).await {
            Ok(Ok(value)) => {
                let value_str = format!("{:?}", value);
                // Извлекаем OID из значения типа "OBJECT IDENTIFIER: 1.3.6.1.4.1.8072.3.2.10"
                if let Some(oid_part) = value_str.split(": ").nth(1) {
                    Ok(oid_part.to_string())
                } else {
                    Ok(value_str)
                }
            }

            Ok(Err(e)) => Err(anyhow::anyhow!("Ошибка получения sysObjectID: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Таймаут при получении sysObjectID")),
        }
    }
}
