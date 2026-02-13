mod collector;
mod config;
mod formatter;
mod snmp;
mod routes;
mod handlers;
mod models;

use routes::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use collector::SnmpCollector;
use formatter::JsonFormatter;

fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // TODO поменять/убрать лимит
    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(4 * 1024 * 1024) 
        .enable_all()
        .build()
        .expect("Не удалось создать runtime");

    rt.block_on(async {
        let app = create_router();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.expect("Не удалось сделать bind");
        axum::serve(listener, app).await.expect("Не удалось создать сервер");
    });
}
    
    // let config = config::AppConfig::load("./profiles/generic-endpoint.yaml")?;
    // let target = config.get_target();

    // // Результаты для JSON вывода
    // let mut results = Vec::new();

    // match create_snmpv3_client(&config, &target).await {
    //     Ok(client) => match SnmpCollector::collect_all(client, &config, "SNMPv3").await {
    //         Ok(result) => {
    //             results.push(result);
    //         }
    //         Err(e) => {
    //             eprintln!("SNMPv3 сбор данных не удался: {}", e);
    //         }
    //     },
    //     Err(_) => {
    //         eprintln!("SNMPv3 клиент недоступен");
    //     }
    // }

//     match create_snmpv2c_client(&config, &target).await {
//         Ok(client) => match SnmpCollector::collect_all(client, &config, "SNMPv2c").await {
//             Ok(result) => {
//                 results.push(result);
//             }
//             Err(_) => {
//                 eprintln!("SNMPv2c недоступен");
//             }
//         },
//         Err(_) => {
//             eprintln!("snmp недоступен");
//         }
//     }

//     // Выводим результаты в JSON
//     for result in results {
//         match JsonFormatter::to_json_string(&result) {
//             Ok(json) => println!("{}", json),
//             Err(e) => eprintln!("❌ Ошибка JSON сериализации: {}", e),
//         }
//     }

//     Ok(())
// }

// /// Создает SNMPv3 клиент
// async fn create_snmpv3_client(
//     config: &config::AppConfig,
//     target: &str,
// ) -> Result<snmp::SnmpClient> {
//     snmp::create_v3_client_auth_priv(
//         target,
//         &config.get_username(),
//         &config.get_auth_password(),
//         config.settings.get_auth_protocol(),
//         config.settings.get_privacy_protocol(),
//         &config.get_privacy_password(),
//     )
//     .await
// }

// /// Создает SNMPv2c клиент
// async fn create_snmpv2c_client(
//     config: &config::AppConfig,
//     target: &str,
// ) -> Result<snmp::SnmpClient> {
//     snmp::create_v2c_client(target, &config.get_community()).await
// }