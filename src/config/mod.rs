use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

pub mod profile;
pub mod settings;

pub use profile::Profile;
pub use settings::Settings;

/// Главная конфигурация приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// SNMP профиль устройства
    pub profile: Profile,
    /// Базовые настройки
    pub settings: Settings,
}

impl AppConfig {
    /// Загружает конфигурацию из YAML файла
    pub fn load(profile_path: impl AsRef<Path>) -> Result<Self> {
        let profile = Profile::load(profile_path.as_ref().to_str().unwrap())?;
        let settings = Settings::default();

        Ok(Self { profile, settings })
    }

    /// Получает target из переменной окружения или использует по умолчанию
    pub fn get_target(&self) -> String {
        env::var("SNMP_TARGET").unwrap_or_else(|_| "127.0.0.1:161".to_string())
    }

    /// Получает timeout из переменной окружения или из настроек
    pub fn get_timeout(&self) -> u64 {
        env::var("SNMP_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.settings.connection.timeout)
    }

    /// Получает community для SNMPv2c
    pub fn get_community(&self) -> Vec<u8> {
        env::var("SNMP_COMMUNITY")
            .unwrap_or_else(|_| self.settings.auth.v2c.community.clone())
            .into_bytes()
    }

    /// Получает username для SNMPv3
    pub fn get_username(&self) -> Vec<u8> {
        env::var("SNMP_USERNAME")
            .unwrap_or_else(|_| self.settings.auth.v3.username.clone())
            .into_bytes()
    }

    /// Получает auth password для SNMPv3
    pub fn get_auth_password(&self) -> Vec<u8> {
        env::var("SNMP_AUTH_PASSWORD")
            .unwrap_or_else(|_| self.settings.auth.v3.auth_password.clone())
            .into_bytes()
    }

    /// Получает privacy password для SNMPv3
    pub fn get_privacy_password(&self) -> Vec<u8> {
        env::var("SNMP_PRIVACY_PASSWORD")
            .unwrap_or_else(|_| self.settings.auth.v3.privacy_password.clone())
            .into_bytes()
    }

    pub fn debug_config(&self) {
        println!("=== Конфигурация SNMP ===");
        println!("Профиль: {}", self.profile.name);
        println!("Цель: {}", self.get_target());
        println!("Таймаут: {}с", self.get_timeout());
        println!("Скаляров: {}", self.profile.scalars.len());
        println!("Таблиц: {}", self.profile.tables.len());
        println!("========================");
    }
}
