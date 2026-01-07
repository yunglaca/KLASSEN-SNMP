use anyhow::{Context, Result};
use snmp2::{AsyncSession, Oid, Value};

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

pub struct SnmpClientV2c {
    session: AsyncSession,
}

impl SnmpClientV2c {
    pub async fn new(target: &str, community: &[u8]) -> Result<Self> {
        let session = AsyncSession::new_v2c(target, community, 2)
            .await
            .context("Не удалось создать SNMP сессию")?;

        Ok(Self { session })
    }

    pub async fn get(&mut self, oid: &Oid<'_>) -> Result<Value<'_>> {
        let resp = self
            .session
            .get(oid)
            .await
            .context("SNMP GET запрос не удался")?;

        let (_, value) = resp
            .varbinds
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("SNMP ответ пустой"))?;

        Ok(value)
    }
}
