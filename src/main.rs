use crate::snmp::SnmpClientV2c;
use tokio::time::{Duration, timeout};

mod config;
mod snmp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let profile = config::Profile::load("./profiles/generic-endpoint.yaml")?;
    println!("Профиль загружен: {}", profile.name);

    let mut client = SnmpClientV2c::new("127.0.0.1:161", b"public").await?;
    println!("SNMP сессия создана");

    println!("Опрос scalars:");
    for (name, oid_str) in &profile.scalars {
        let oid = snmp::parse_oid(oid_str)?;

        let timeout_secs = 3;
        match timeout(Duration::from_secs(timeout_secs), client.get(&oid)).await {
            Ok(Ok(value)) => println!("  {} = {:?}", name, value),
            Ok(Err(e)) => println!("  {} = ERROR: {}", name, e),
            Err(_) => println!("  {} = TIMEOUT (нет ответа за {timeout_secs} сек)", name),
        }
    }
    Ok(())
}
