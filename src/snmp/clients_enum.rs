use super::v2c::SnmpClientV2c;
use super::v3::SnmpClientV3;

pub enum SnmpClient {
    V2c(SnmpClientV2c),
    V3(SnmpClientV3),
}