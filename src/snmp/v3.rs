use anyhow::Result;
use snmp2::{Oid, Value};

pub struct SnmpClientV3 {
    // TODO: сохранять SNMPv3 сессию или состояние, связанное с безопасностью и тд, здесь
}

impl SnmpClientV3 {
    // TODO: сделать конструкторы для различных уровней безопасности (noAuthNoPriv, authNoPriv, authPriv)
    pub async fn new(_target: &str) -> Result<Self> {
        // TODO: внедрить создание сессии SNMPv3
        unimplemented!("SNMPv3 client is not implemented yet");
    }

    pub async fn get(&mut self, _oid: &Oid<'_>) -> Result<Value<'_>> {
        // TODO: внедрить SNMPv3 GET
        unimplemented!("SNMPv3 GET is not implemented yet");
    }
}
