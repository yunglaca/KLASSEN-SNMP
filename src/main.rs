use anyhow::Result;

mod collector;
mod config;
mod formatter;
mod snmp;

use collector::SnmpCollector;
use formatter::JsonFormatter;

// TODO: Добавить конфигурационный файл для самого приложения
// TODO: Добавить метрики и мониторинг:
// - Время выполнения операций
// - Количество успешных/неуспешных запросов
// - Статистика по типам устройств
// - Экспорт метрик maybeee


#[tokio::main]
async fn main() -> Result<()> {
    // Загружаем конфигурацию без лишних сообщений
    let config = config::AppConfig::load("./profiles/generic-endpoint.yaml")?;
    let target = config.get_target();

    // Результаты для JSON вывода
    let mut results = Vec::new();

    // Пробуем SNMPv3
    match create_snmpv3_client(&config, &target).await {
        Ok(client) => match SnmpCollector::collect_all(client, &config, "SNMPv3").await {
            Ok(result) => {
                results.push(result);
            }
            Err(e) => {
                eprintln!("SNMPv3 сбор данных не удался: {}", e);
            }
        },
        Err(_) => {
            eprintln!("SNMPv3 клиент недоступен");
        }
    }

    // Пробуем SNMPv2c (только если SNMPv3 не сработал)
    if results.is_empty() {
        match create_snmpv2c_client(&config, &target).await {
            Ok(client) => match SnmpCollector::collect_all(client, &config, "SNMPv2c").await {
                Ok(result) => {
                    results.push(result);
                }
                Err(_) => {
                    eprintln!("SNMPv2c также недоступен");
                }
            },
            Err(_) => {
                eprintln!("Все SNMP версии недоступны");
            }
        }
    }

    // Выводим результаты в JSON
    for result in results {
        match JsonFormatter::to_json_string(&result) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("❌ Ошибка JSON сериализации: {}", e),
        }
    }

    Ok(())
}

/// Создает SNMPv3 клиент
async fn create_snmpv3_client(
    config: &config::AppConfig,
    target: &str,
) -> Result<snmp::SnmpClient> {
    snmp::create_v3_client_auth_priv(
        target,
        &config.get_username(),
        &config.get_auth_password(),
        config.settings.get_auth_protocol(),
        config.settings.get_privacy_protocol(),
        &config.get_privacy_password(),
    )
    .await
}

/// Создает SNMPv2c клиент
async fn create_snmpv2c_client(
    config: &config::AppConfig,
    target: &str,
) -> Result<snmp::SnmpClient> {
    snmp::create_v2c_client(target, &config.get_community()).await
}
