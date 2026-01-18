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
    pub tables: Option<Vec<TableResult>>,
}
