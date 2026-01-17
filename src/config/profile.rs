use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String, // Название профиля, например "generic-endpoint" или "printer"
    pub scalars: HashMap<String, String>, // Хранит скалярные OID — одиночные значения, которые опрашиваются через SNMP
    pub tables: HashMap<String, String>, // Тоже словарь, но хранит корневые OID таблиц для SNMP WALK
}

impl Profile {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Не удалось прочитать файл: {}", path))?;

        let profile: Profile =
            serde_yml::from_str(&content).context("Не удалось распарсить YAML")?;

        if profile.scalars.is_empty() && profile.tables.is_empty() {
            anyhow::bail!("Профиль '{}' пустой", profile.name);
        }

        Ok(profile)
    }
}
