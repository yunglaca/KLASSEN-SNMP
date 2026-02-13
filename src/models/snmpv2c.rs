use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Snmpv2c {
    pub ip: String,
    pub community: String,
}