use axum::{Json, http::StatusCode, response::IntoResponse};
use tokio::time::{timeout, Duration};

use crate::models::snmpv2c::Snmpv2c;
use crate::snmp::{create_v2c_client, parse_oid};

const SNMP_TIMEOUT_SECS: u64 = 10;

pub async fn handle_snmpv2c(Json(params): Json<Snmpv2c>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let target = format!("{}:161", params.ip);

    // TODO поменять хардкод оида 
    // sysobjectid 
    let oid = parse_oid("1.3.6.1.2.1.1.2.0")
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let work = async {
        let mut client = create_v2c_client(&target, &params.community.as_bytes()).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let value = client.get(&oid).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok::<_, (StatusCode, String)>(format!("{:?}", value))
    };

    let value_str = match timeout(Duration::from_secs(SNMP_TIMEOUT_SECS), work).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err((
            StatusCode::GATEWAY_TIMEOUT,
            "SNMP request timeout".to_string(),
        )),
    };

    Ok(Json(serde_json::json!({ "value": value_str })))
}
