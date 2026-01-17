use anyhow::Result;
use snmp2::{Oid, Value};

pub mod clients_enum;
pub mod device_profiles;
pub mod v2c;
pub mod v3;

pub use clients_enum::SnmpClient;
pub use device_profiles::{DeviceDetector, parse_oid, set_global_device_type};
pub use v2c::SnmpClientV2c;
pub use v3::SnmpClientV3;

pub use snmp2::v3::{AuthProtocol, Cipher};

impl SnmpClient {
    pub async fn get(&mut self, oid: &Oid<'_>) -> Result<Value<'_>> {
        match self {
            SnmpClient::V2c(client) => client.get(oid).await,
            SnmpClient::V3(client) => client.get(oid).await,
        }
    }

    pub async fn walk(&mut self, root_oid: &Oid<'_>) -> Result<Vec<(Oid<'static>, String)>> {
        match self {
            SnmpClient::V2c(client) => client.walk(root_oid).await,
            SnmpClient::V3(client) => client.walk(root_oid).await,
        }
    }

    pub async fn walk_limited(
        &mut self,
        root_oid: &Oid<'_>,
        max_items: usize,
    ) -> Result<Vec<(Oid<'static>, String)>> {
        let all_items = self.walk(root_oid).await?;
        Ok(all_items.into_iter().take(max_items).collect())
    }
}

// TODO: создать фабрику для поддержки выбора версии (v2c/v3) по конфигурации
pub async fn create_v2c_client(target: &str, community: &[u8]) -> anyhow::Result<SnmpClient> {
    let client = SnmpClientV2c::new(target, community).await?;
    Ok(SnmpClient::V2c(client))
}

// TODO: расширить фабрику
pub async fn create_v3_client_auth_priv(
    target: &str,
    username: &[u8],
    auth_password: &[u8],
    auth_protocol: AuthProtocol,
    cipher: Cipher,
    privacy_password: &[u8],
) -> anyhow::Result<SnmpClient> {
    let client = SnmpClientV3::new_auth_priv(
        target,
        username,
        auth_password,
        auth_protocol,
        cipher,
        privacy_password,
    )
    .await?;
    Ok(SnmpClient::V3(client))
}
