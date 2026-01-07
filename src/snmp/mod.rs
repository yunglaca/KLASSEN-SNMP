pub mod snmp;

pub use snmp::{SnmpClientV2c, parse_oid};

// TODO: создать фабрику для поддержки SNMPv3 и выбора по конфигурации
pub async fn create_v2c_client(target: &str, community: &[u8]) -> anyhow::Result<SnmpClientV2c> {
    SnmpClientV2c::new(target, community).await
}
