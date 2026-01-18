use anyhow::Result;
use tokio::time::{Duration, timeout};

use crate::snmp::{DeviceDetector, SnmpClient, parse_oid, set_global_device_type};

/// Модуль для работы с информацией об устройстве
pub struct DeviceInfo;

impl DeviceInfo {
    /// Получает sysObjectID устройства
    pub async fn get_sys_object_id(client: &mut SnmpClient) -> Result<String> {
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

    /// Определяет тип устройства и устанавливает глобальный тип
    pub async fn detect_and_set_device_type(client: &mut SnmpClient) {
        // Определяем тип устройства для возможного использования в будущем
        if let Ok(sys_object_id) = Self::get_sys_object_id(client).await {
            let device_info = DeviceDetector::detect_device_type(&sys_object_id);
            // Устанавливаем глобальный тип устройства для правильного разрешения OID
            set_global_device_type(device_info.device_type);
        } else {
            set_global_device_type("generic".to_string());
        }
    }
}
