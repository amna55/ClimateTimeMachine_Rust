// src/api/climate.rs

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Router,
    Json,
    debug_handler,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{DateTime, Utc};
use validator::Validate;
use std::sync::Arc;
use crate::services::earthengine::{EarthEngineService, LSTResponse};
use crate::services::data_fetcher; // Your get_temperature function

// -------------------- Query Structs --------------------
#[derive(Debug, Deserialize, Validate)]
pub struct LocationQuery {
    #[validate(range(min = -90.0, max = 90.0))]
    lat: f64,
    #[validate(range(min = -180.0, max = 180.0))]
    lng: f64,
}

// -------------------- Response Structs --------------------
#[derive(Debug, Serialize)]
pub struct TemperatureResponse {
    temperature: f64,
    unit: String,
    timestamp: DateTime<Utc>,
}

// -------------------- Handlers --------------------
#[debug_handler]
pub async fn get_current_temperature(
    Query(query): Query<LocationQuery>
) -> Result<Json<TemperatureResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Validate location coordinates
    if let Err(e) = query.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid coordinates", "details": e.to_string()}))
        ));
    }

    // Fetch temperature from your data service
    match data_fetcher::get_temperature(query.lat, query.lng).await {
        Ok(temp) => Ok(Json(TemperatureResponse {
            temperature: temp,
            unit: "C".to_string(),
            timestamp: Utc::now(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to fetch temperature", "details": e.to_string()}))
        )),
    }
}

#[debug_handler]
pub async fn get_lst_temperature(
    Query(query): Query<LocationQuery>,
    State(earth_engine): State<Arc<EarthEngineService>>,
) -> Result<Json<LSTResponse>, (StatusCode, Json<serde_json::Value>)> {
    let bbox = vec![
        query.lng - 0.1,
        query.lat - 0.1,
        query.lng + 0.1,
        query.lat + 0.1,
    ];

    let response = earth_engine
        .get_lst_data(bbox, 256, 256)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch LST data", "details": e.to_string()})),
            )
        })?;

    Ok(Json(response))
}

// -------------------- Routes --------------------
pub fn routes(earth_engine: Arc<EarthEngineService>) -> Router {
    Router::new()
        .route("/current", get(get_current_temperature))
        .route("/lst", get(get_lst_temperature))
        .with_state(earth_engine)
}
