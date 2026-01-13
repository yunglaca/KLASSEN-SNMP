use anyhow::{Context, Result};
use snmp2::{v3, AsyncSession, Oid, Value};

pub struct SnmpClientV3 {
    session: AsyncSession,
}

impl SnmpClientV3 {
    // TODO ПРОТЕСТИТЬ!
    /// Конструктор для noAuthNoPriv (без аутентификации и шифрования)
    pub async fn new_no_auth_no_priv(target: &str, username: &[u8]) -> Result<Self> {
        let security = v3::Security::new(username, b"");

        let mut session = AsyncSession::new_v3(target, 0, security)
            .await
            .context("Failed to create SNMPv3 session")?;

        // Инициализация для получения engine_id от агента
        session
            .init()
            .await
            .context("Failed to initialize session")?;

        Ok(Self { session })
    }

    // TODO ПРОТЕСТИТЬ!
    /// Конструктор для authNoPriv (с аутентификацией, без шифрования)
    pub async fn new_auth_no_priv(
        target: &str,
        username: &[u8],
        auth_password: &[u8],
        auth_protocol: v3::AuthProtocol,
    ) -> Result<Self> {
        let security = v3::Security::new(username, auth_password)
            .with_auth_protocol(auth_protocol)
            .with_auth(v3::Auth::AuthNoPriv);

        let mut session = AsyncSession::new_v3(target, 0, security)
            .await
            .context("Failed to create SNMPv3 session")?;

        session
            .init()
            .await
            .context("Failed to initialize session")?;

        Ok(Self { session })
    }
    

    // тестировался!
    /// Конструктор для authPriv (с аутентификацией и шифрованием)
    pub async fn new_auth_priv(
        target: &str,
        username: &[u8],
        auth_password: &[u8],
        auth_protocol: v3::AuthProtocol,
        cipher: v3::Cipher,
        privacy_password: &[u8],
    ) -> Result<Self> {
        let security = v3::Security::new(username, auth_password)
            .with_auth_protocol(auth_protocol)
            .with_auth(v3::Auth::AuthPriv {
                cipher,
                privacy_password: privacy_password.to_vec(),
            });

        let mut session = AsyncSession::new_v3(target, 0, security)
            .await
            .context("Failed to create SNMPv3 session")?;

        session
            .init()
            .await
            .context("Failed to initialize session")?;

        Ok(Self { session })
    }

    pub async fn get(&mut self, oid: &Oid<'_>) -> Result<Value<'_>> {
        let resp = self
            .session
            .get(oid)
            .await
            .context("SNMPv3 GET запрос не удался")?;

        let (_, value) = resp
            .varbinds
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("SNMPv3 ответ пустой"))?;

        Ok(value)
    }
}
