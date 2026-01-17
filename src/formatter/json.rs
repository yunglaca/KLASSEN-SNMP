use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::collector::{MonitoringResult, ScalarResult, TableResult};

// TODO: Расширение JSON форматирования для интеграции:
// - Добавить streaming JSON для очень больших результатов

// TODO: Улучшение метаданных в JSON:
// - Добавить информацию о производительности (время сбора, количество запросов)
// - Добавить версию профиля устройства использованного для сбора

// TODO: Оптимизация JSON структуры:
// - Добавить опции для compact/verbose режимов
// - Добавить фильтрацию полей (включить только нужные поля)
// - Добавить группировку связанных метрик для уменьшения размера
// - Добавить поддержка incremental updates (только изменившиеся значения)

/// JSON структура для отдачи монолиту
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringResultJson {
    pub device_type: String,
    pub client_type: String,
    pub timestamp: String,
    pub summary: ResultSummary,
    pub scalars: Vec<ScalarResultJson>,
    pub tables: Vec<TableResultJson>,
    pub errors: Vec<ErrorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSummary {
    pub total_scalars: usize,
    pub successful_scalars: usize,
    pub total_tables: usize,
    pub successful_tables: usize,
    pub total_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalarResultJson {
    pub name: String,
    pub oid: String,
    pub value: Option<String>,
    pub status: String, // "success" | "error" | "timeout"
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResultJson {
    pub name: String,
    pub oid: String,
    pub status: String, // "success" | "error" | "timeout"
    pub row_count: usize,
    pub limited_to: Option<usize>,
    pub columns: HashMap<String, ColumnInfo>,
    pub rows: Vec<RowData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub oid_pattern: String,
    pub value_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    pub oid: String,
    pub value: String,
    pub parsed_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub item_type: String, // "scalar" | "table"
    pub item_name: String,
    pub error_message: String,
}

/// JSON форматтер для результатов мониторинга
pub struct JsonFormatter;

// Срань конечно какая-то, но хз как по другому..
impl JsonFormatter {
    /// Конвертирует результат мониторинга в JSON
    pub fn format_monitoring_result(result: &MonitoringResult) -> MonitoringResultJson {
        let timestamp = chrono::Utc::now().to_rfc3339();

        let successful_scalars = result.scalars.iter().filter(|s| s.error.is_none()).count();
        let successful_tables = result.tables.iter().filter(|t| t.error.is_none()).count();
        let total_rows: usize = result.tables.iter().map(|t| t.rows.len()).sum();

        let summary = ResultSummary {
            total_scalars: result.scalars.len(),
            successful_scalars,
            total_tables: result.tables.len(),
            successful_tables,
            total_rows,
        };

        let scalars = result
            .scalars
            .iter()
            .map(|s| Self::format_scalar(s))
            .collect();

        let tables = result
            .tables
            .iter()
            .map(|t| Self::format_table(t))
            .collect();

        let errors = Self::extract_errors(result);

        MonitoringResultJson {
            device_type: "auto_detected".to_string(), // Будет заполняться из device detection
            client_type: result.client_type.clone(),
            timestamp,
            summary,
            scalars,
            tables,
            errors,
        }
    }

    /// Форматирует скалярное значение для JSON
    fn format_scalar(scalar: &ScalarResult) -> ScalarResultJson {
        let status = match (&scalar.value, &scalar.error) {
            (Some(_), None) => "success",
            (None, Some(error)) if error.contains("TIMEOUT") => "timeout",
            (None, Some(_)) => "error",
            _ => "unknown",
        };

        ScalarResultJson {
            name: scalar.name.clone(),
            oid: scalar.oid.clone(),
            value: scalar.value.clone(),
            status: status.to_string(),
            error: scalar.error.clone(),
        }
    }

    /// Форматирует таблицу для JSON
    fn format_table(table: &TableResult) -> TableResultJson {
        let status = match &table.error {
            None => "success",
            Some(error) if error.contains("TIMEOUT") => "timeout",
            Some(_) => "error",
        };

        let (columns, rows) = Self::analyze_table_structure(&table.rows);

        TableResultJson {
            name: table.name.clone(),
            oid: table.oid.clone(),
            status: status.to_string(),
            row_count: table.rows.len(),
            limited_to: table.limited_to,
            columns,
            rows,
            error: table.error.clone(),
        }
    }

    /// Анализирует структуру таблицы для JSON
    fn analyze_table_structure(
        rows: &[(String, String)],
    ) -> (HashMap<String, ColumnInfo>, Vec<RowData>) {
        let mut columns: HashMap<String, ColumnInfo> = HashMap::new();
        let mut formatted_rows = Vec::new();

        for (oid_str, value) in rows {
            // Определяем колонку из OID
            let column_oid = Self::extract_column_oid(oid_str);
            let column_name = Self::get_column_name(&column_oid);

            // Обновляем информацию о колонке
            let column_info = columns.entry(column_oid.clone()).or_insert(ColumnInfo {
                name: column_name,
                oid_pattern: format!("{}.*", column_oid),
                value_count: 0,
            });
            column_info.value_count += 1;

            // Добавляем строку данных
            formatted_rows.push(RowData {
                oid: oid_str.clone(),
                value: value.clone(),
                parsed_name: Self::parse_oid_name(oid_str),
            });
        }

        (columns, formatted_rows)
    }

    /// Извлекает OID колонки из полного OID
    fn extract_column_oid(oid_str: &str) -> String {
        let parts: Vec<&str> = oid_str.split('.').collect();

        // Для ifTable: 1.3.6.1.2.1.2.2.1.X.INDEX -> 1.3.6.1.2.1.2.2.1.X
        if parts.len() >= 10 && parts[0..9] == ["1", "3", "6", "1", "2", "1", "2", "2", "1"] {
            return parts[0..10].join(".");
        }

        // Для hrStorageTable: 1.3.6.1.2.1.25.2.3.1.X.INDEX -> 1.3.6.1.2.1.25.2.3.1.X
        if parts.len() >= 11 && parts[0..10] == ["1", "3", "6", "1", "2", "1", "25", "2", "3", "1"]
        {
            return parts[0..11].join(".");
        }

        // Для hrDeviceTable: 1.3.6.1.2.1.25.3.2.1.X.INDEX -> 1.3.6.1.2.1.25.3.2.1.X
        if parts.len() >= 11 && parts[0..10] == ["1", "3", "6", "1", "2", "1", "25", "3", "2", "1"]
        {
            return parts[0..11].join(".");
        }

        // Fallback - убираем последний компонент (индекс)
        if parts.len() > 1 {
            parts[0..parts.len() - 1].join(".")
        } else {
            oid_str.to_string()
        }
    }

    /// Получает имя колонки по OID
    fn get_column_name(column_oid: &str) -> String {
        match column_oid {
            // Interface Table columns
            "1.3.6.1.2.1.2.2.1.1" => "ifIndex".to_string(),
            "1.3.6.1.2.1.2.2.1.2" => "ifDescr".to_string(),
            "1.3.6.1.2.1.2.2.1.3" => "ifType".to_string(),
            "1.3.6.1.2.1.2.2.1.4" => "ifMtu".to_string(),
            "1.3.6.1.2.1.2.2.1.5" => "ifSpeed".to_string(),
            "1.3.6.1.2.1.2.2.1.7" => "ifAdminStatus".to_string(),
            "1.3.6.1.2.1.2.2.1.8" => "ifOperStatus".to_string(),
            "1.3.6.1.2.1.2.2.1.10" => "ifInOctets".to_string(),
            "1.3.6.1.2.1.2.2.1.16" => "ifOutOctets".to_string(),
            "1.3.6.1.2.1.2.2.1.13" => "ifInDiscards".to_string(),
            "1.3.6.1.2.1.2.2.1.19" => "ifOutDiscards".to_string(),
            "1.3.6.1.2.1.2.2.1.21" => "ifOutQLen".to_string(),

            // Host Resources Storage Table
            "1.3.6.1.2.1.25.2.3.1.1" => "hrStorageIndex".to_string(),
            "1.3.6.1.2.1.25.2.3.1.2" => "hrStorageType".to_string(),
            "1.3.6.1.2.1.25.2.3.1.3" => "hrStorageDescr".to_string(),
            "1.3.6.1.2.1.25.2.3.1.5" => "hrStorageSize".to_string(),
            "1.3.6.1.2.1.25.2.3.1.6" => "hrStorageUsed".to_string(),

            // Host Resources Device Table
            "1.3.6.1.2.1.25.3.2.1.1" => "hrDeviceIndex".to_string(),
            "1.3.6.1.2.1.25.3.2.1.2" => "hrDeviceType".to_string(),
            "1.3.6.1.2.1.25.3.2.1.3" => "hrDeviceDescr".to_string(),

            _ => format!("column_{}", column_oid.replace(".", "_")),
        }
    }

    /// Парсит OID в читаемое имя с индексом
    fn parse_oid_name(oid_str: &str) -> Option<String> {
        let column_oid = Self::extract_column_oid(oid_str);
        let column_name = Self::get_column_name(&column_oid);

        // Извлекаем индекс
        let index = oid_str.strip_prefix(&format!("{}.", column_oid))?;

        Some(format!("{}.{}", column_name, index))
    }

    /// Извлекает ошибки из результата
    fn extract_errors(result: &MonitoringResult) -> Vec<ErrorInfo> {
        let mut errors = Vec::new();

        // Ошибки скаляров
        for scalar in &result.scalars {
            if let Some(ref error) = scalar.error {
                errors.push(ErrorInfo {
                    item_type: "scalar".to_string(),
                    item_name: scalar.name.clone(),
                    error_message: error.clone(),
                });
            }
        }

        // Ошибки таблиц
        for table in &result.tables {
            if let Some(ref error) = table.error {
                errors.push(ErrorInfo {
                    item_type: "table".to_string(),
                    item_name: table.name.clone(),
                    error_message: error.clone(),
                });
            }
        }

        errors
    }

    /// Сериализует результат в JSON строку
    pub fn to_json_string(result: &MonitoringResult) -> anyhow::Result<String> {
        let json_result = Self::format_monitoring_result(result);
        serde_json::to_string_pretty(&json_result)
            .map_err(|e| anyhow::anyhow!("Ошибка сериализации в JSON: {}", e))
    }

    /// Сериализует результат в компактный JSON
    pub fn to_json_compact(result: &MonitoringResult) -> anyhow::Result<String> {
        let json_result = Self::format_monitoring_result(result);
        serde_json::to_string(&json_result)
            .map_err(|e| anyhow::anyhow!("Ошибка сериализации в JSON: {}", e))
    }
}
