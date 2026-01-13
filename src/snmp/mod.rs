use anyhow::Result;
use snmp2::{Oid, Value};

pub mod v2c;
pub mod v3;
pub mod oid;
pub mod clients_enum;

pub use clients_enum::SnmpClient;
pub use v2c::SnmpClientV2c;
pub use v3::SnmpClientV3;
pub use oid::parse_oid;


pub use snmp2::v3::{AuthProtocol, Cipher};



impl SnmpClient {
    pub async fn get(&mut self, oid: &Oid<'_>) -> Result<Value<'_>> {
        match self {
            SnmpClient::V2c(client) => client.get(oid).await,
            SnmpClient::V3(client) => client.get(oid).await,
        }
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
