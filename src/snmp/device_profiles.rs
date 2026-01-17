use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use snmp2::Oid;
use std::collections::HashMap;
use std::path::Path;

// TODO: Расширение поддержки вендоров и устройств:
// - Добавить автоматическое обнаружение MIB файлов в директории
// - Добавить парсинг стандартных MIB файлов (.mib, .txt)
// - Создать базу данных известных sysObjectID → device mappings

// TODO: Улучшение детекции устройств:
// - Добавить multi-stage detection (sysObjectID + sysDescr + дополнительные OID)
// - Добавить fuzzy matching для неизвестных устройств (поиск похожих)

// TODO: Динамическая загрузка профилей:
// - Поддержка загрузки профилей каким-либо образом (например, через HTTP)
// - hot reload профилей без перезапуска

// !!!!!!!!!!!!!!!!!!!!!!!!!!! Дальше текст нейронки, я в целом верю ей :)
//
//
// ИНСТРУКЦИЯ: Как расширить поддержку OID и добавить новый вендор
//
// 1. ДОБАВЛЕНИЕ НОВОГО ВЕНДОРА (встроенный способ):
//    a) Найдите Enterprise ID вендора на http://www.iana.org/assignments/enterprise-numbers
//       Например: Cisco = 9, HP = 11, Juniper = 2636, Huawei = 2011
//
//    b) Добавьте детекцию в DeviceDetector::detect_device_type():
//       ```rust
//       } else if sys_object_id.starts_with("1.3.6.1.4.1.2011.") {
//           DeviceInfo {
//               device_type: "huawei".to_string(),
//               description: "Huawei Device".to_string(),
//           }
//       ```
//
//    c) Добавьте профиль в DeviceProfileManager::load_builtin_profiles():
//       ```rust
//       let huawei_profile = DeviceProfile {
//           device_type: "huawei".to_string(),
//           scalars: [
//               ("cpuUtilization".to_string(), "1.3.6.1.4.1.2011.5.25.31.1.1.1.1.5".to_string()),
//               // Huawei CPU utilization OID
//           ].into_iter().collect(),
//           tables: [
//               ("huaweiIfTable".to_string(), "1.3.6.1.4.1.2011.5.25.42.1.1.1".to_string()),
//               // Huawei interface table OID
//           ].into_iter().collect(),
//       };
//       self.profiles.insert("huawei".to_string(), huawei_profile);
//       ```
//
// 2. ДОБАВЛЕНИЕ ЧЕРЕЗ YAML ФАЙЛ (наверное будет использован этот способ, но не факт, из кода смешнее)):
//    Создайте файл profiles/vendors/huawei.yaml:
//    ```yaml
//    device_type: "huawei"
//    description: "Huawei Network Device"
//    detection:
//      sysObjectID_prefix: "1.3.6.1.4.1.2011."
//    scalars:
//      cpuUtilization: "1.3.6.1.4.1.2011.5.25.31.1.1.1.1.5"
//      memoryUtilization: "1.3.6.1.4.1.2011.5.25.31.1.1.1.1.7"
//    tables:
//      huaweiIfTable: "1.3.6.1.4.1.2011.5.25.42.1.1.1"
//    ```
//    Загрузите через: manager.load_profile_from_file("profiles/vendors/huawei.yaml")?
//
//
// 3. КАК НАЙТИ НУЖНЫЕ OID ДЛЯ ВЕНДОРА:
//    a) Скачайте MIB файлы с сайта вендора
//    b) Используйте snmpwalk для исследования устройства:
//       `snmpwalk -v2c -c public device_ip 1.3.6.1.4.1.VENDOR_ID`
//    c) Проверьте документацию вендора (SNMP MIB Reference Guide)
//    d) Используйте online MIB браузеры (например, oidref.com)
//
// 4. СТАНДАРТНЫЕ ПАТТЕРНЫ OID ПО КАТЕГОРИЯМ:
//    - CPU: обычно содержит "cpu", "processor" в названии MIB
//    - Memory: обычно содержит "memory", "mem" в названии MIB
//    - Interface: обычно расширение стандартного ifTable
//    - Temperature: обычно в environmental или sensor MIB
//    - Fan Status: также в environmental MIB
//    - Power Supply: в power или environmental MIB
//
// 5. ТЕСТИРОВАНИЕ НОВОГО ПРОФИЛЯ:
//    a) Используйте FORCE_DEVICE_TYPE=huawei для принудительного тестирования
//    b) Добавьте unit тесты в конец файла
//    c) Протестируйте на реальном устройстве если возможно

/// Информация о типе устройства
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: String,
    pub description: String,
}

/// Профиль устройства с OID маппингами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub device_type: String,
    pub description: String,
    pub detection: DetectionConfig,
    pub oid_mappings: Option<HashMap<String, String>>,
    pub scalars: HashMap<String, String>,
    pub tables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub sysObjectID_prefix: Option<String>,
    pub sysDescr_contains: Option<Vec<String>>,
}

/// Детектор типа устройства
pub struct DeviceDetector;

impl DeviceDetector {
    /// Определяет тип устройства по sysObjectID
    pub fn detect_device_type(sys_object_id: &str) -> DeviceInfo {
        // Простая детекция для MVP
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
        } else {
            DeviceInfo {
                device_type: "generic".to_string(),
                description: "Generic SNMP Device".to_string(),
            }
        }
    }
}

/// Менеджер профилей устройств
pub struct DeviceProfileManager {
    profiles: HashMap<String, DeviceProfile>,
}

impl DeviceProfileManager {
    /// Создает новый менеджер и загружает профили
    // TODO: Добавить конфигурируемые пути для профилей через config файл
    // TODO: Добавить lazy loading профилей - загружать только когда нужны
    // TODO: Добавить профиль caching с TTL
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            profiles: HashMap::new(),
        };

        // Загружаем встроенные профили
        manager.load_builtin_profiles()?;

        Ok(manager)
    }

    /// Загружает встроенные профили устройств
    // TODO: Заменить hardcoded профили на загрузку из YAML файлов
    // TODO: Добавить валидацию загружаемых профилей
    // TODO: Добавить поддержку профиль inheritance (base profile + overrides)
    fn load_builtin_profiles(&mut self) -> Result<()> {
        // Generic профиль загружается из generic-endpoint.yaml
        // через AppConfig::load(), поэтому здесь не дублируем

        // Linux Net-SNMP профиль
        let linux_profile = DeviceProfile {
            device_type: "linux".to_string(),
            description: "Linux Net-SNMP Agent".to_string(),
            detection: DetectionConfig {
                sysObjectID_prefix: Some("1.3.6.1.4.1.8072.".to_string()),
                sysDescr_contains: Some(vec!["Linux".to_string()]),
            },
            oid_mappings: Some(
                [
                    (
                        "loadAverage1min".to_string(),
                        "1.3.6.1.4.1.2021.10.1.3.1".to_string(),
                    ),
                    (
                        "loadAverage5min".to_string(),
                        "1.3.6.1.4.1.2021.10.1.3.2".to_string(),
                    ),
                    (
                        "loadAverage15min".to_string(),
                        "1.3.6.1.4.1.2021.10.1.3.3".to_string(),
                    ),
                    (
                        "memTotalReal".to_string(),
                        "1.3.6.1.4.1.2021.4.5.0".to_string(),
                    ),
                    (
                        "memAvailReal".to_string(),
                        "1.3.6.1.4.1.2021.4.6.0".to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            scalars: [
                ("sysDescr".to_string(), "1.3.6.1.2.1.1.1.0".to_string()),
                ("sysUpTime".to_string(), "1.3.6.1.2.1.1.3.0".to_string()),
                ("sysName".to_string(), "1.3.6.1.2.1.1.5.0".to_string()),
                ("sysLocation".to_string(), "1.3.6.1.2.1.1.6.0".to_string()),
                // Linux-специфичные скаляры
                (
                    "loadAverage1min".to_string(),
                    "1.3.6.1.4.1.2021.10.1.3.1".to_string(),
                ),
                (
                    "memTotalReal".to_string(),
                    "1.3.6.1.4.1.2021.4.5.0".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            tables: [
                ("ifTable".to_string(), "1.3.6.1.2.1.2.2".to_string()),
                (
                    "hrStorageTable".to_string(),
                    "1.3.6.1.2.1.25.2.3".to_string(),
                ),
                // Linux-специфичные таблицы
                ("processTable".to_string(), "1.3.6.1.2.1.25.4.2".to_string()),
                ("diskTable".to_string(), "1.3.6.1.4.1.2021.9.1".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        // Cisco профиль
        let cisco_profile = DeviceProfile {
            device_type: "cisco".to_string(),
            description: "Cisco Device".to_string(),
            detection: DetectionConfig {
                sysObjectID_prefix: Some("1.3.6.1.4.1.9.".to_string()),
                sysDescr_contains: Some(vec!["Cisco".to_string(), "IOS".to_string()]),
            },
            oid_mappings: Some(
                [
                    (
                        "cpuUsage".to_string(),
                        "1.3.6.1.4.1.9.9.109.1.1.1.1.7".to_string(),
                    ),
                    (
                        "memoryUsed".to_string(),
                        "1.3.6.1.4.1.9.9.48.1.1.1.5".to_string(),
                    ),
                    (
                        "memoryFree".to_string(),
                        "1.3.6.1.4.1.9.9.48.1.1.1.6".to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            scalars: [
                ("sysDescr".to_string(), "1.3.6.1.2.1.1.1.0".to_string()),
                ("sysUpTime".to_string(), "1.3.6.1.2.1.1.3.0".to_string()),
                ("sysName".to_string(), "1.3.6.1.2.1.1.5.0".to_string()),
                // Cisco-специфичные скаляры
                (
                    "cpuUsage".to_string(),
                    "1.3.6.1.4.1.9.9.109.1.1.1.1.7".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            tables: [
                ("ifTable".to_string(), "1.3.6.1.2.1.2.2".to_string()),
                // Cisco-специфичные таблицы
                (
                    "ciscoMemoryPoolTable".to_string(),
                    "1.3.6.1.4.1.9.9.48.1.1".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        };

        // Generic профиль не добавляем - он загружается из YAML
        self.profiles.insert("linux".to_string(), linux_profile);
        self.profiles.insert("cisco".to_string(), cisco_profile);

        Ok(())
    }

    /// Получает профиль устройства по типу
    // TODO: Добавить профиль matching по similarity score если точного совпадения нет
    // TODO: Добавить профиль composition - объединение нескольких профилей
    pub fn get_profile(&self, device_type: &str) -> Option<&DeviceProfile> {
        self.profiles.get(device_type).or_else(|| {
            // Fallback на generic профиль
            // TODO: Улучшить fallback логику - искать наиболее подходящий профиль
            self.profiles.get("generic")
        })
    }

    /// Загружает профиль из YAML файла (для будущего расширения)
    // TODO: Добавить schema validation для профилей
    // TODO: Добавить hot-reload профилей при изменении файлов
    pub fn load_profile_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content =
            std::fs::read_to_string(&path).context("Не удалось прочитать файл профиля")?;

        let profile: DeviceProfile =
            serde_yml::from_str(&content).context("Не удалось парсить YAML профиль")?;

        // TODO: Валидация профиля перед добавлением
        self.profiles.insert(profile.device_type.clone(), profile);

        Ok(())
    }

    /// Получает список доступных типов устройств
    // TODO: Добавить метаданные о профилях (версия, автор, описание, поддерживаемые модели)
    // TODO: Добавить статистику использования профилей
    pub fn get_available_device_types(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
}

/// Улучшенный OID resolver с поддержкой устройств
pub struct DeviceAwareOidResolver {
    profile_manager: DeviceProfileManager,
    current_device_type: Option<String>,
}

impl DeviceAwareOidResolver {
    /// Создает новый resolver
    // TODO: Добавить MIB parser integration для автоматического OID resolution
    // TODO: Добавить кеширование resolved OID names
    pub fn new() -> Result<Self> {
        Ok(Self {
            profile_manager: DeviceProfileManager::new()?,
            current_device_type: None,
        })
    }

    /// Устанавливает тип текущего устройства
    pub fn set_device_type(&mut self, device_type: String) {
        self.current_device_type = Some(device_type);
    }

    /// Парсит OID строку в объект Oid
    pub fn parse_oid(&self, s: &str) -> Result<Oid<'static>> {
        let parts: Result<Vec<u64>, _> = s
            .trim()
            .split('.')
            .filter(|p| !p.is_empty())
            .map(|p| p.parse::<u64>())
            .collect();

        let parts = parts.context("Не удалось распарсить OID")?;
        Oid::from(&parts).map_err(|e| anyhow::anyhow!("Не удалось создать Oid: {:?}", e))
    }

    /// Преобразует OID в человеко-читаемое имя с учетом типа устройства
    pub fn oid_to_name(&self, oid: &Oid) -> Option<String> {
        let device_type = self.current_device_type.as_deref().unwrap_or("generic");
        let profile = self.profile_manager.get_profile(device_type)?;

        let oid_str = oid.to_string();

        // Сначала проверяем в скалярах профиля
        for (name, profile_oid) in &profile.scalars {
            if oid_str == *profile_oid {
                return Some(name.clone());
            }
        }

        // Затем проверяем в дополнительных маппингах
        if let Some(ref mappings) = profile.oid_mappings {
            for (name, profile_oid) in mappings {
                if oid_str == *profile_oid {
                    return Some(name.clone());
                }
            }
        }

        // Fallback на встроенную логику для таблиц
        self.resolve_table_oid(&oid_str, profile)
    }

    /// Разрешает OID таблицы
    fn resolve_table_oid(&self, oid_str: &str, _profile: &DeviceProfile) -> Option<String> {
        let parts: Vec<&str> = oid_str.split('.').collect();

        // Для ifTable: 1.3.6.1.2.1.2.2.1.X.INDEX -> 1.3.6.1.2.1.2.2.1.X
        if parts.len() >= 10 && parts[0..9] == ["1", "3", "6", "1", "2", "1", "2", "2", "1"] {
            let column_oid = parts[0..10].join(".");
            let index = parts[10..].join(".");

            let column_name = match column_oid.as_str() {
                "1.3.6.1.2.1.2.2.1.1" => "ifIndex",
                "1.3.6.1.2.1.2.2.1.2" => "ifDescr",
                "1.3.6.1.2.1.2.2.1.3" => "ifType",
                "1.3.6.1.2.1.2.2.1.4" => "ifMtu",
                "1.3.6.1.2.1.2.2.1.5" => "ifSpeed",
                "1.3.6.1.2.1.2.2.1.7" => "ifAdminStatus",
                "1.3.6.1.2.1.2.2.1.8" => "ifOperStatus",
                "1.3.6.1.2.1.2.2.1.10" => "ifInOctets",
                "1.3.6.1.2.1.2.2.1.16" => "ifOutOctets",
                "1.3.6.1.2.1.2.2.1.19" => "ifOutDiscards",
                "1.3.6.1.2.1.2.2.1.21" => "ifOutQLen",
                _ => return None,
            };

            return Some(format!("{}.{}", column_name, index));
        }

        // Аналогично для других таблиц...
        None
    }

    /// Форматирует значение SNMP с учетом типа устройства
    pub fn format_snmp_value(&self, oid: &Oid, value: &str) -> String {
        let oid_str = oid.to_string();

        // ifType values
        if oid_str.contains("1.3.6.1.2.1.2.2.1.3.") {
            if let Ok(type_val) = value.replace("INTEGER: ", "").parse::<u32>() {
                let type_name = match type_val {
                    1 => "other",
                    6 => "ethernetCsmacd",
                    24 => "softwareLoopback",
                    131 => "tunnel",
                    _ => "unknown",
                };
                return format!("{} ({})", value, type_name);
            }
        }

        // ifOperStatus values
        if oid_str.contains("1.3.6.1.2.1.2.2.1.8.") {
            if let Ok(status_val) = value.replace("INTEGER: ", "").parse::<u32>() {
                let status_name = match status_val {
                    1 => "up",
                    2 => "down",
                    3 => "testing",
                    _ => "unknown",
                };
                return format!("{} ({})", value, status_name);
            }
        }

        value.to_string()
    }

    /// Получает информацию о текущем устройстве
    pub fn get_current_device_info(&self) -> Option<String> {
        self.current_device_type.clone()
    }

    /// Получает профиль текущего устройства
    pub fn get_current_profile(&self) -> Option<&DeviceProfile> {
        let device_type = self.current_device_type.as_deref().unwrap_or("generic");
        self.profile_manager.get_profile(device_type)
    }
}

// Глобальный статический resolver для обратной совместимости
use std::sync::LazyLock;
static GLOBAL_RESOLVER: LazyLock<std::sync::Mutex<DeviceAwareOidResolver>> =
    LazyLock::new(|| std::sync::Mutex::new(DeviceAwareOidResolver::new().unwrap()));

/// Парсит OID строку (обратная совместимость)
pub fn parse_oid(s: &str) -> Result<Oid<'static>> {
    let resolver = GLOBAL_RESOLVER.lock().unwrap();
    resolver.parse_oid(s)
}

/// Преобразует OID в имя (обратная совместимость)
pub fn oid_to_name(oid: &Oid) -> Option<String> {
    let resolver = GLOBAL_RESOLVER.lock().unwrap();
    resolver.oid_to_name(oid)
}

/// Форматирует SNMP значение (обратная совместимость)
pub fn format_snmp_value(oid: &Oid, value: &str) -> String {
    let resolver = GLOBAL_RESOLVER.lock().unwrap();
    resolver.format_snmp_value(oid, value)
}

/// Устанавливает тип устройства глобально
// TODO: Заменить глобальный state на context-based approach
// TODO: Добавить thread-safety improvements
pub fn set_global_device_type(device_type: String) {
    let mut resolver = GLOBAL_RESOLVER.lock().unwrap();
    resolver.set_device_type(device_type);
}

// TODO: Добавить юнит тесты
