use anyhow::{Context, Result};
use snmp2::{AsyncSession, Oid, Value};

pub struct SnmpClientV2c {
    pub(crate) session: AsyncSession,
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
