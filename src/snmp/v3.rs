use anyhow::{Context, Result};
use snmp2::{AsyncSession, Oid, Value, v3};

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
    // тупа копипаст из v2c но пока так =)
    pub async fn walk(&mut self, start_oid: &Oid<'_>) -> Result<Vec<(Oid<'static>, String)>> {
        self.walk_bulk(start_oid, 10).await
    }
    // тупа копипаст из v2c но пока так =)
    pub async fn walk_bulk(
        &mut self,
        start_oid: &Oid<'_>,
        max_repetitions: u32,
    ) -> Result<Vec<(Oid<'static>, String)>> {
        let mut results: Vec<(Oid<'static>, String)> = Vec::new();
        let mut current_oid = start_oid.to_owned();

        loop {
            // Выполняем SNMP GETBULK запрос
            let resp = self
                .session
                .getbulk(&[&current_oid], 0, max_repetitions)
                .await
                .context("SNMPv3 GETBULK запрос не удался")?;

            let mut items = Vec::new();
            let mut found_any = false;

            // Обрабатываем каждый элемент из ответа и конвертируем в строку
            for (oid, value) in resp.varbinds {
                if !oid.starts_with(start_oid) {
                    // Добавляем собранные элементы перед возвратом
                    results.extend(items);
                    return Ok(results);
                }

                // Конвертируем Value в строку для возможности клонирования
                let value_str = format!("{:?}", value);
                items.push((oid.to_owned(), value_str));
                current_oid = oid.to_owned();
                found_any = true;
            }

            if !found_any {
                break;
            }

            // Добавляем все собранные элементы
            results.extend(items);
        }

        Ok(results)
    }
}
