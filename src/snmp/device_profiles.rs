use anyhow::Result;
use snmp2::Oid;
use std::sync::LazyLock;

/// Информация об устройстве
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: String,
    pub description: String,
}

/// Детектор типа устройства
pub struct DeviceDetector;

impl DeviceDetector {
    /// Определяет тип устройства по sysObjectID
    pub fn detect_device_type(sys_object_id: &str) -> DeviceInfo {
        if sys_object_id.starts_with("1.3.6.1.4.1.8072.") {
            DeviceInfo {
                device_type: "linux".to_string(),
                description: "Linux Net-SNMP Agent".to_string(),
            }
        } else if sys_object_id.starts_with("1.3.6.1.4.1.9.") {
            DeviceInfo {
                device_type: "cisco".to_string(),
                description: "Cisco Device".to_string(),
            }
        } else if sys_object_id.starts_with("1.3.6.1.4.1.11.") {
            DeviceInfo {
                device_type: "hp".to_string(),
                description: "HP Device".to_string(),
            }
        } else if sys_object_id.starts_with("1.3.6.1.4.1.2636.") {
            DeviceInfo {
                device_type: "juniper".to_string(),
                description: "Juniper Device".to_string(),
            }
        } else if sys_object_id.starts_with("1.3.6.1.4.1.2011.") {
            DeviceInfo {
                device_type: "huawei".to_string(),
                description: "Huawei Device".to_string(),
            }
        } else {
            DeviceInfo {
                device_type: "generic".to_string(),
                description: "Unknown Device".to_string(),
            }
        }
    }
}

/// Простой OID resolver для текущего устройства
struct DeviceTypeHolder {
    current_device_type: Option<String>,
}

impl DeviceTypeHolder {
    fn new() -> Self {
        Self {
            current_device_type: None,
        }
    }

    fn set_device_type(&mut self, device_type: String) {
        self.current_device_type = Some(device_type);
    }
}

/// Глобальный держатель типа устройства
static GLOBAL_DEVICE_TYPE: LazyLock<std::sync::Mutex<DeviceTypeHolder>> =
    LazyLock::new(|| std::sync::Mutex::new(DeviceTypeHolder::new()));

/// Парсит строку OID в объект Oid
pub fn parse_oid(oid_str: &str) -> Result<Oid> {
    let parts: Result<Vec<u64>, _> = oid_str
        .trim()
        .split('.')
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<u64>())
        .collect();

    let parts = parts
        .map_err(|e| anyhow::anyhow!("Не удалось распарсить числа в OID '{}': {}", oid_str, e))?;
    Oid::from(&parts)
        .map_err(|e| anyhow::anyhow!("Не удалось создать Oid из '{}': {:?}", oid_str, e))
}

/// Устанавливает глобальный тип устройства
pub fn set_global_device_type(device_type: String) {
    let mut holder = GLOBAL_DEVICE_TYPE.lock().unwrap();
    holder.set_device_type(device_type);
}