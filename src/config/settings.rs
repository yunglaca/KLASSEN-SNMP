use serde::{Deserialize, Serialize};
use snmp2::v3::{AuthProtocol, Cipher};

/// Базовые настройки приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Настройки подключения
    pub connection: ConnectionSettings,
    /// Настройки аутентификации
    pub auth: AuthSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    /// Таймаут для SNMP операций (секунды)
    pub timeout: u64,
    /// Количество повторов при ошибках
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    /// Настройки SNMPv2c
    pub v2c: SnmpV2cSettings,
    /// Настройки SNMPv3
    pub v3: SnmpV3Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpV2cSettings {
    /// Community string
    pub community: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpV3Settings {
    /// Имя пользователя
    pub username: String,
    /// Пароль аутентификации
    pub auth_password: String,
    /// Пароль шифрования
    pub privacy_password: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            connection: ConnectionSettings {
                timeout: 10,
                retries: 2,
            },
            auth: AuthSettings {
                v2c: SnmpV2cSettings {
                    community: "public".to_string(),
                },
                v3: SnmpV3Settings {
                    username: "myuser".to_string(),
                    auth_password: "myauthpass".to_string(),
                    privacy_password: "myprivpass".to_string(),
                },
            },
        }
    }
}

impl Settings {
    /// Получает протокол аутентификации (всегда SHA1)
    pub fn get_auth_protocol(&self) -> AuthProtocol {
        AuthProtocol::Sha1
    }

    /// Получает протокол шифрования (всегда AES128)
    pub fn get_privacy_protocol(&self) -> Cipher {
        Cipher::Aes128
    }
}
