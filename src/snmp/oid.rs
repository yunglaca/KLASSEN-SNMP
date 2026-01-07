use anyhow::{Context, Result};
use snmp2::Oid;

pub fn parse_oid(s: &str) -> Result<Oid<'static>> {
    let parts: Result<Vec<u64>, _> = s
        .trim()
        .split('.')
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<u64>())
        .collect();

    let parts = parts.context(format!("Невалидный OID: {}", s))?;
    Oid::from(&parts)
        .map_err(|e| anyhow::anyhow!("Не удалось создать Oid: {:?}", e))
}
