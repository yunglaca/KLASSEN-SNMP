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

    pub async fn walk(&mut self, start_oid: &Oid<'_>) -> Result<Vec<(Oid<'static>, String)>> {
        self.walk_bulk(start_oid, 10).await
    }

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
                .context("SNMP GETBULK запрос не удался")?;

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
