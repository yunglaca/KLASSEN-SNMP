use serde::Deserialize;
use std::{collections::HashMap};
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub name: String,                         // Название профиля, например "generic-endpoint" или "printer"
    pub scalars: HashMap<String, String>,     // Хранит скалярные OID — одиночные значения, которые опрашиваются через SNMP
    pub tables: HashMap<String, String>,      // Тоже словарь, но хранит корневые OID таблиц для SNMP WALK
}

impl Profile {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Не удалось прочитать файл: {}", path))?;
        
        let profile: Profile = serde_yml::from_str(&content).context("Не удалось распарсить YAML")?;
        
        if profile.scalars.is_empty() && profile.tables.is_empty() {
            anyhow::bail!("Профиль '{}' пустой", profile.name);
        }

        Ok(profile)
    }

    // TODO Найти имя scalar по OID (для маппинга результатов)
    pub fn find_scalar_name(&self, oid_str: &str) -> Option<&String> {
        self.scalars.iter()
            .find(|(_, v)| v.as_str() == oid_str)
            .map(|(k,_)| k)
    }

    pub fn item_count(&self) -> usize {
        self.scalars.len() + self.tables.len()
    }
}